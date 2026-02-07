mod config;
mod context;
pub mod eval;
mod layer;
mod result;

pub use config::*;
pub use context::*;
pub use eval::score::ScoreConfig;
pub use layer::*;
pub use result::*;

use std::sync::{Arc, Mutex};

use loom_codec::{CodecRegistry, CodecRegistryBuilder};
use loom_config::Config;
use loom_core::{Format, MediaType, decode, encode, ident_path};
use loom_error::Result;
use loom_io::{DataSourceRegistry, DataSourceRegistryBuilder, path::Path};

// Re-export config types
pub use loom_config::{Config as RConfig, ConfigError};
pub use loom_pipe::{
    Layer, LayerContext, LayerResult, Pipeline, PipelineBuilder,
    operators::{Await, FanOut, Filter, Fork, Parallel, Router, TryMap},
};
use serde::{Serialize, de::DeserializeOwned};

// Re-export commonly used types for convenience
#[cfg(feature = "toml")]
pub use loom_codec::TomlCodec;
#[cfg(feature = "yaml")]
pub use loom_codec::YamlCodec;
pub use loom_codec::{JsonCodec, TextCodec};
pub use loom_io::Record;
pub use loom_io::sources::FileSystemSource;

// Re-export signal types for convenience
pub use loom_signal::{
    Emitter, Level, NoopEmitter, Signal, SignalBroadcaster, Span, Type as SignalType,
    consumers::{FileEmitter, MemoryEmitter, StdoutEmitter},
};

/// Wrapper that bridges Arc<Mutex<ScoreLayer>> to the Layer trait.
/// This allows the scorer to be used via runtime.eval().
struct ScorerLayerWrapper(Arc<Mutex<eval::score::ScoreLayer>>);

impl Layer for ScorerLayerWrapper {
    type Input = Context<()>;
    type Output = eval::score::ScoreResult;

    fn process(&self, input: Self::Input) -> Result<LayerResult<Self::Output>> {
        let scorer = self.0.lock().expect("scorer lock poisoned");
        scorer.process(input)
    }

    fn name(&self) -> &'static str {
        "score"
    }
}

pub struct Runtime {
    codecs: CodecRegistry,
    sources: DataSourceRegistry,
    layers: LayerRegistry,
    rconfig: Config,
    scorer: Arc<Mutex<eval::score::ScoreLayer>>,
    signals: Arc<dyn Emitter + Send + Sync>,
}

impl Runtime {
    pub fn new() -> Builder {
        Builder::new()
    }

    pub fn codecs(&self) -> &CodecRegistry {
        &self.codecs
    }

    pub fn sources(&self) -> &DataSourceRegistry {
        &self.sources
    }

    pub fn layers(&self) -> &LayerRegistry {
        &self.layers
    }

    /// Get the LoomConfig (runtime settings like batch_size, strict, etc.)
    pub fn config(&self) -> LoomConfig {
        self.rconfig.root_section().bind().unwrap_or_default()
    }

    /// Get the raw Config object for section access.
    pub fn rconfig(&self) -> &Config {
        &self.rconfig
    }

    /// Get a reference to the signal emitter.
    pub fn emitter(&self) -> &dyn Emitter {
        self.signals.as_ref()
    }

    /// Emit a signal through the runtime's emitter.
    pub fn emit(&self, signal: Signal) {
        self.signals.emit(signal);
    }

    /// Get access to the scorer for direct batch operations.
    pub fn scorer(&self) -> &Arc<Mutex<eval::score::ScoreLayer>> {
        &self.scorer
    }

    /// Score a single text using the registered score layer.
    ///
    /// This uses `runtime.eval()` internally for type-checked layer invocation.
    ///
    /// # Example
    /// ```ignore
    /// let result = runtime.score("Some text to classify")?;
    /// println!("Score: {}", result.score);
    /// ```
    pub fn score(&self, text: &str) -> Result<eval::score::ScoreResult> {
        let ctx = Context::new(text, ());
        self.eval::<Context<()>, eval::score::ScoreResult>("score", ctx)
    }

    /// Score multiple texts in a batch using the registered score layer.
    ///
    /// More efficient than calling `score()` repeatedly for ML inference.
    ///
    /// # Example
    /// ```ignore
    /// let texts = &["first text", "second text"];
    /// let outputs = runtime.score_batch(texts)?;
    /// for output in outputs {
    ///     println!("Score: {}", output.score());
    /// }
    /// ```
    pub fn score_batch(&self, texts: &[&str]) -> Result<Vec<eval::score::ScoreLayerOutput>> {
        let scorer = self.scorer.lock().expect("scorer lock poisoned");
        scorer.score_batch(texts)
    }

    pub fn pipeline<Input: Send + 'static>(&self) -> PipelineBuilder<Input, Input> {
        PipelineBuilder::new()
    }

    /// Evaluate input using a named layer.
    ///
    /// Returns the layer's output, performing runtime type checks.
    ///
    /// # Example
    /// ```ignore
    /// let result: MyOutput = runtime.eval("my_layer", my_input)?;
    /// ```
    pub fn eval<I, O>(&self, layer_name: &str, input: I) -> Result<O>
    where
        I: Send + 'static,
        O: Send + 'static,
    {
        use std::any::TypeId;

        let layer = self
            .layers
            .get_checked(layer_name, TypeId::of::<I>(), TypeId::of::<O>())?;

        let boxed_input: Box<dyn std::any::Any + Send> = Box::new(input);
        let boxed_output = layer.process_any(boxed_input)?;

        boxed_output.downcast::<O>().map(|b| *b).map_err(|_| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::Unknown)
                .message("Output type mismatch after layer execution")
                .build()
        })
    }

    /// Evaluate a dataset using the registered scorer.
    ///
    /// Progress is emitted through the runtime's signal system.
    ///
    /// # Example
    /// ```ignore
    /// let result = runtime.eval_scoring(&dataset, 16).await?;
    /// ```
    pub async fn eval_scoring(
        &self,
        dataset: &eval::SampleDataset,
        batch_size: usize,
    ) -> Result<eval::EvalResult> {
        use loom_cortex::bench::Decision;
        use std::collections::HashSet;

        let scorer = self.scorer.clone();
        let eval_start = std::time::Instant::now();
        let total = dataset.samples.len();

        // Emit start signal
        self.emit(
            Signal::new()
                .otype(SignalType::Event)
                .name("eval.start")
                .attr("total", total as i64)
                .build(),
        );

        // Collect all samples with their original indices
        let indexed_samples: Vec<(usize, eval::Sample)> =
            dataset.samples.iter().cloned().enumerate().collect();

        // Process samples in batches
        let mut all_results: Vec<(eval::Sample, eval::SampleResult)> = Vec::with_capacity(total);
        let mut processed = 0;

        for chunk in indexed_samples.chunks(batch_size) {
            let batch_samples: Vec<(usize, eval::Sample)> = chunk.to_vec();
            let texts: Vec<String> = batch_samples.iter().map(|(_, s)| s.text.clone()).collect();
            let scorer = scorer.clone();

            // Process batch in spawn_blocking
            let batch_outputs = tokio::task::spawn_blocking(move || {
                let scorer = scorer.lock().expect("scorer lock poisoned");
                let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                scorer.score_batch(&text_refs)
            })
            .await
            .expect("spawn_blocking failed");

            // Evaluate each sample in the batch
            match batch_outputs {
                Ok(outputs) => {
                    for ((_idx, sample), output) in
                        batch_samples.into_iter().zip(outputs.into_iter())
                    {
                        let detected_labels = output.detected_labels();
                        let actual_decision = output.decision();
                        let score = output.score();

                        let sample_result = eval::SampleResult {
                            id: sample.id.clone(),
                            expected_decision: sample.expected_decision,
                            actual_decision,
                            correct: actual_decision == sample.expected_decision,
                            score,
                            expected_labels: sample.expected_labels.clone(),
                            detected_labels,
                            elapsed_ms: None,
                        };

                        processed += 1;

                        // Emit progress signal
                        self.emit(
                            Signal::new()
                                .otype(SignalType::Event)
                                .name("eval.progress")
                                .attr("current", processed as i64)
                                .attr("total", total as i64)
                                .attr("sample_id", sample.id.clone())
                                .attr("correct", sample_result.correct)
                                .build(),
                        );

                        all_results.push((sample, sample_result));
                    }
                }
                Err(e) => {
                    // Emit error signal
                    self.emit(
                        Signal::new()
                            .otype(SignalType::Event)
                            .level(Level::Error)
                            .name("eval.batch_error")
                            .attr("error", e.to_string())
                            .build(),
                    );

                    // On batch error, mark all samples as rejected
                    for (_idx, sample) in batch_samples {
                        let sample_result = eval::SampleResult {
                            id: sample.id.clone(),
                            expected_decision: sample.expected_decision,
                            actual_decision: Decision::Reject,
                            correct: sample.expected_decision == Decision::Reject,
                            score: 0.0,
                            expected_labels: sample.expected_labels.clone(),
                            detected_labels: vec![],
                            elapsed_ms: None,
                        };

                        processed += 1;

                        // Emit progress signal
                        self.emit(
                            Signal::new()
                                .otype(SignalType::Event)
                                .name("eval.progress")
                                .attr("current", processed as i64)
                                .attr("total", total as i64)
                                .attr("sample_id", sample.id.clone())
                                .attr("correct", sample_result.correct)
                                .build(),
                        );

                        all_results.push((sample, sample_result));
                    }
                }
            }
        }

        // Calculate timing metrics
        let elapsed = eval_start.elapsed();
        let elapsed_ms = elapsed.as_millis() as i64;
        let throughput = if elapsed.as_secs_f32() > 0.0 {
            total as f32 / elapsed.as_secs_f32()
        } else {
            0.0
        };

        // Emit completion signal
        self.emit(
            Signal::new()
                .otype(SignalType::Event)
                .name("eval.complete")
                .attr("elapsed_ms", elapsed_ms)
                .attr("throughput", throughput as f64)
                .attr("total", total as i64)
                .attr(
                    "correct",
                    all_results.iter().filter(|(_, r)| r.correct).count() as i64,
                )
                .build(),
        );

        // Build result
        let mut result = eval::EvalResult::new();
        result.total = all_results.len();
        result.elapsed_ms = elapsed_ms;
        result.throughput = throughput;

        for (sample, sample_result) in all_results {
            if sample_result.correct {
                result.correct += 1;
            }

            let cat_result = result
                .per_category
                .entry(sample.primary_category.clone())
                .or_default();
            cat_result.total += 1;
            if sample_result.correct {
                cat_result.correct += 1;
            }

            // Update per-label metrics
            let expected_set: HashSet<_> = sample.expected_labels.iter().collect();
            let detected_set: HashSet<_> = sample_result.detected_labels.iter().collect();

            for label in &sample.expected_labels {
                let entry = result.per_label.entry(label.clone()).or_default();
                entry.expected_count += 1;
            }

            for label in &sample_result.detected_labels {
                let entry = result.per_label.entry(label.clone()).or_default();
                entry.detected_count += 1;

                if expected_set.contains(label) {
                    entry.true_positives += 1;
                } else {
                    entry.false_positives += 1;
                }
            }

            for label in &sample.expected_labels {
                if !detected_set.contains(label) {
                    let entry = result.per_label.entry(label.clone()).or_default();
                    entry.false_negatives += 1;
                }
            }

            result.sample_results.push(sample_result);
        }

        Ok(result)
    }

    /// Evaluate a dataset and return both results and raw scores.
    ///
    /// Combines eval_scoring with raw score extraction for Platt calibration training.
    ///
    /// # Example
    /// ```ignore
    /// let (result, raw_scores) = runtime.eval_scoring_with_scores(&dataset, 16).await?;
    /// let export = ScoreExport::from_results(&dataset, &result, raw_scores);
    /// ```
    pub async fn eval_scoring_with_scores(
        &self,
        dataset: &eval::SampleDataset,
        batch_size: usize,
    ) -> Result<(
        eval::EvalResult,
        std::collections::HashMap<String, std::collections::HashMap<String, f32>>,
    )> {
        use loom_cortex::bench::Decision;
        use std::collections::{HashMap, HashSet};

        let scorer = self.scorer.clone();
        let eval_start = std::time::Instant::now();
        let total = dataset.samples.len();

        // Emit start signal
        self.emit(
            Signal::new()
                .otype(SignalType::Event)
                .name("eval.start")
                .attr("total", total as i64)
                .build(),
        );

        // Collect all samples with their original indices
        let indexed_samples: Vec<(usize, eval::Sample)> =
            dataset.samples.iter().cloned().enumerate().collect();

        // Process samples in batches
        let mut all_results: Vec<(eval::Sample, eval::SampleResult, HashMap<String, f32>)> =
            Vec::with_capacity(total);
        let mut processed = 0;

        for chunk in indexed_samples.chunks(batch_size) {
            let batch_samples: Vec<(usize, eval::Sample)> = chunk.to_vec();
            let texts: Vec<String> = batch_samples.iter().map(|(_, s)| s.text.clone()).collect();
            let scorer = scorer.clone();

            // Process batch in spawn_blocking
            let batch_outputs = tokio::task::spawn_blocking(move || {
                let scorer = scorer.lock().expect("scorer lock poisoned");
                let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                scorer.score_batch(&text_refs)
            })
            .await
            .expect("spawn_blocking failed");

            // Evaluate each sample in the batch
            match batch_outputs {
                Ok(outputs) => {
                    for ((_idx, sample), output) in
                        batch_samples.into_iter().zip(outputs.into_iter())
                    {
                        let detected_labels = output.detected_labels();
                        let actual_decision = output.decision();
                        let score = output.score();
                        let raw_scores: HashMap<String, f32> =
                            output.labels().into_iter().collect();

                        let sample_result = eval::SampleResult {
                            id: sample.id.clone(),
                            expected_decision: sample.expected_decision,
                            actual_decision,
                            correct: actual_decision == sample.expected_decision,
                            score,
                            expected_labels: sample.expected_labels.clone(),
                            detected_labels,
                            elapsed_ms: None,
                        };

                        processed += 1;

                        // Emit progress signal
                        self.emit(
                            Signal::new()
                                .otype(SignalType::Event)
                                .name("eval.progress")
                                .attr("current", processed as i64)
                                .attr("total", total as i64)
                                .attr("sample_id", sample.id.clone())
                                .attr("correct", sample_result.correct)
                                .build(),
                        );

                        all_results.push((sample, sample_result, raw_scores));
                    }
                }
                Err(e) => {
                    // Emit error signal
                    self.emit(
                        Signal::new()
                            .otype(SignalType::Event)
                            .level(Level::Error)
                            .name("eval.batch_error")
                            .attr("error", e.to_string())
                            .build(),
                    );

                    // On batch error, mark all samples as rejected with empty scores
                    for (_idx, sample) in batch_samples {
                        let sample_result = eval::SampleResult {
                            id: sample.id.clone(),
                            expected_decision: sample.expected_decision,
                            actual_decision: Decision::Reject,
                            correct: sample.expected_decision == Decision::Reject,
                            score: 0.0,
                            expected_labels: sample.expected_labels.clone(),
                            detected_labels: vec![],
                            elapsed_ms: None,
                        };

                        processed += 1;

                        // Emit progress signal
                        self.emit(
                            Signal::new()
                                .otype(SignalType::Event)
                                .name("eval.progress")
                                .attr("current", processed as i64)
                                .attr("total", total as i64)
                                .attr("sample_id", sample.id.clone())
                                .attr("correct", sample_result.correct)
                                .build(),
                        );

                        all_results.push((sample, sample_result, HashMap::new()));
                    }
                }
            }
        }

        // Calculate timing metrics
        let elapsed = eval_start.elapsed();
        let elapsed_ms = elapsed.as_millis() as i64;
        let throughput = if elapsed.as_secs_f32() > 0.0 {
            total as f32 / elapsed.as_secs_f32()
        } else {
            0.0
        };

        // Emit completion signal
        self.emit(
            Signal::new()
                .otype(SignalType::Event)
                .name("eval.complete")
                .attr("elapsed_ms", elapsed_ms)
                .attr("throughput", throughput as f64)
                .attr("total", total as i64)
                .attr(
                    "correct",
                    all_results.iter().filter(|(_, r, _)| r.correct).count() as i64,
                )
                .build(),
        );

        // Build result and raw_scores map
        let mut result = eval::EvalResult::new();
        let mut raw_scores_map: HashMap<String, HashMap<String, f32>> = HashMap::new();
        result.total = all_results.len();
        result.elapsed_ms = elapsed_ms;
        result.throughput = throughput;

        for (sample, sample_result, raw_scores) in all_results {
            if sample_result.correct {
                result.correct += 1;
            }

            let cat_result = result
                .per_category
                .entry(sample.primary_category.clone())
                .or_default();
            cat_result.total += 1;
            if sample_result.correct {
                cat_result.correct += 1;
            }

            // Update per-label metrics
            let expected_set: HashSet<_> = sample.expected_labels.iter().collect();
            let detected_set: HashSet<_> = sample_result.detected_labels.iter().collect();

            for label in &sample.expected_labels {
                let entry = result.per_label.entry(label.clone()).or_default();
                entry.expected_count += 1;
            }

            for label in &sample_result.detected_labels {
                let entry = result.per_label.entry(label.clone()).or_default();
                entry.detected_count += 1;

                if expected_set.contains(label) {
                    entry.true_positives += 1;
                } else {
                    entry.false_positives += 1;
                }
            }

            for label in &sample.expected_labels {
                if !detected_set.contains(label) {
                    let entry = result.per_label.entry(label.clone()).or_default();
                    entry.false_negatives += 1;
                }
            }

            // Store raw scores by sample ID
            raw_scores_map.insert(sample_result.id.clone(), raw_scores);

            result.sample_results.push(sample_result);
        }

        Ok((result, raw_scores_map))
    }

    /// Load and deserialize data from a DataSource.
    ///
    /// # Arguments
    /// * `source` - The name of the registered DataSource (e.g., "file_system")
    /// * `path` - The path to load from
    ///
    /// # Example
    /// ```ignore
    /// let dataset: SampleDataset = runtime.load("file_system", &path).await?;
    /// ```
    pub async fn load<T: DeserializeOwned>(&self, source: &str, path: &Path) -> Result<T> {
        let source = self.sources.get(source).ok_or_else(|| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::NotFound)
                .message(format!("DataSource '{}' not found", source))
                .build()
        })?;

        let record = source.find_one(path).await.map_err(|e| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::Unknown)
                .message(format!("Failed to load from path '{}': {}", path, e))
                .build()
        })?;

        let content = record.content_str().map_err(|e| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::Unknown)
                .message(format!("Invalid UTF-8 content: {}", e))
                .build()
        })?;

        decode!(content, record.media_type.format()).map_err(|e| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::Unknown)
                .message(format!("Deserialization failed: {}", e))
                .build()
        })
    }

    /// Save and serialize data to a DataSource.
    ///
    /// # Arguments
    /// * `source` - The name of the registered DataSource (e.g., "file_system")
    /// * `path` - The path to save to
    /// * `data` - The data to serialize and save
    /// * `format` - The format to serialize as
    ///
    /// # Example
    /// ```ignore
    /// runtime.save("file_system", &path, &export, Format::Json).await?;
    /// ```
    pub async fn save<T: Serialize>(
        &self,
        source: &str,
        path: &Path,
        data: &T,
        format: Format,
    ) -> Result<()> {
        let source = self.sources.get(source).ok_or_else(|| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::NotFound)
                .message(format!("DataSource '{}' not found", source))
                .build()
        })?;

        let content = encode!(data, format).map_err(|e| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::Unknown)
                .message(format!("Serialization failed: {}", e))
                .build()
        })?;

        let media_type = match format {
            Format::Json => MediaType::TextJson,
            Format::Yaml => MediaType::TextYaml,
            Format::Toml => MediaType::TextToml,
            _ => MediaType::TextPlain,
        };

        let record = loom_io::Record::from_str(path.clone(), media_type, &content);

        source.upsert(record).await.map_err(|e| {
            loom_error::Error::builder()
                .code(loom_error::ErrorCode::Unknown)
                .message(format!("Failed to save to path '{}': {}", path, e))
                .build()
        })?;

        Ok(())
    }
}

pub struct Builder {
    codecs: CodecRegistryBuilder,
    sources: DataSourceRegistryBuilder,
    layers: LayerRegistry,
    rconfig: Config,
    scorer: Option<eval::score::ScoreLayer>,
    signals: SignalBroadcaster,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            codecs: CodecRegistryBuilder::default(),
            sources: DataSourceRegistryBuilder::default(),
            layers: LayerRegistry::default(),
            rconfig: Config::new().build().unwrap(),
            scorer: None,
            signals: SignalBroadcaster::default(),
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn codec<T: loom_codec::Codec + 'static>(mut self, codec: T) -> Self {
        self.codecs = self.codecs.codec(codec);
        self
    }

    pub fn source<T: loom_io::DataSource + 'static>(mut self, source: T) -> Self {
        self.sources = self.sources.source(source);
        self
    }

    /// Register a layer with the runtime.
    /// The layer's name() method is used as the lookup key.
    pub fn layer<L>(mut self, layer: L) -> Self
    where
        L: Layer + Sync + 'static,
        L::Input: 'static,
        L::Output: 'static,
    {
        self.layers.register(layer);
        self
    }

    /// Set the configuration for the runtime.
    /// Auto-builds the scorer from `layers.score` section if present.
    pub fn config(mut self, config: Config) -> Self {
        let score_path = ident_path!("layers.score");
        let score_section = config.get_section(&score_path);

        if let Ok(score_config) = score_section.bind::<eval::score::ScoreConfig>() {
            if let Ok(scorer) = score_config.build() {
                self.scorer = Some(scorer);
            }
        }

        self.rconfig = config;
        self
    }

    /// Add a signal emitter to the runtime.
    /// Multiple emitters can be added and signals will be broadcast to all of them.
    pub fn emitter<E: Emitter + Send + Sync + 'static>(mut self, emitter: E) -> Self {
        self.signals = self.signals.add(emitter);
        self
    }

    pub fn build(self) -> Runtime {
        let signals: Arc<dyn Emitter + Send + Sync> = if self.signals.is_empty() {
            Arc::new(NoopEmitter)
        } else {
            Arc::new(self.signals)
        };

        // Build scorer from config or use default
        let scorer = self.scorer.unwrap_or_else(|| {
            eval::score::ScoreConfig::default()
                .build()
                .expect("default ScoreConfig should build")
        });

        // Wrap scorer in Arc<Mutex<>> for shared access
        let scorer = Arc::new(Mutex::new(scorer));

        // Register the scorer layer wrapper for runtime.eval() access
        let mut layers = self.layers;
        layers.register(ScorerLayerWrapper(scorer.clone()));

        Runtime {
            codecs: self.codecs.build(),
            sources: self.sources.build(),
            layers,
            rconfig: self.rconfig,
            scorer,
            signals,
        }
    }
}
