use rust_bert::RustBertError;
use rust_bert::pipelines::*;
use serde::{Deserialize, Serialize};

use super::{
    CortexConversationConfig, CortexMaskedLanguageConfig, CortexNerConfig, CortexPosTaggingConfig,
    CortexQuestionAnsweringConfig, CortexSentenceEmbeddingsConfig,
    CortexSentenceEmbeddingsModelType, CortexSentimentConfig, CortexSequenceClassificationConfig,
    CortexSummarizationConfig, CortexTextGenerationConfig, CortexTokenClassificationConfig,
    CortexTranslationConfig, CortexZeroShotConfig,
};
use crate::model::CortexModel;
use crate::{CortexDevice, CortexModelSource, CortexModelType};

/// Serializable configuration for all pipeline types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CortexModelConfig {
    Conversation(CortexConversationConfig),
    MaskedLanguage(CortexMaskedLanguageConfig),
    Ner(CortexNerConfig),
    PosTagging(CortexPosTaggingConfig),
    QuestionAnswering(CortexQuestionAnsweringConfig),
    SentenceEmbeddings(CortexSentenceEmbeddingsConfig),
    Sentiment(CortexSentimentConfig),
    SequenceClassification(CortexSequenceClassificationConfig),
    Summarization(CortexSummarizationConfig),
    TextGeneration(CortexTextGenerationConfig),
    TokenClassification(CortexTokenClassificationConfig),
    Translation(CortexTranslationConfig),
    ZeroShotClassification(CortexZeroShotConfig),
}

impl CortexModelConfig {
    pub fn build(self) -> Result<CortexModel, RustBertError> {
        Ok(match self {
            Self::Conversation(c) => {
                let model_type = c.model.clone();
                CortexModel::Conversation {
                    model: conversation::ConversationModel::new(c.into())?,
                    model_type,
                }
            }
            Self::MaskedLanguage(c) => {
                let model_type = c.model.clone();
                CortexModel::MaskedLanguage {
                    model: masked_language::MaskedLanguageModel::new(c.into())?,
                    model_type,
                }
            }
            Self::Ner(c) => {
                let model_type = c.model.clone();
                CortexModel::Ner {
                    model: ner::NERModel::new(c.into())?,
                    model_type,
                }
            }
            Self::PosTagging(c) => {
                let model_type = c.model.clone();
                CortexModel::PosTagging {
                    model: pos_tagging::POSModel::new(c.into())?,
                    model_type,
                }
            }
            Self::QuestionAnswering(c) => {
                let model_type = c.model.clone();
                CortexModel::QuestionAnswering {
                    model: question_answering::QuestionAnsweringModel::new(c.into())?,
                    model_type,
                }
            }
            Self::SentenceEmbeddings(c) => {
                let model_type = c.model.clone();
                CortexModel::SentenceEmbeddings {
                    model: sentence_embeddings::SentenceEmbeddingsModel::new(c.into())?,
                    model_type,
                }
            }
            Self::Sentiment(c) => {
                let model_type = c.model.clone();
                CortexModel::Sentiment {
                    model: sentiment::SentimentModel::new(c.into())?,
                    model_type,
                }
            }
            Self::SequenceClassification(c) => {
                let model_type = c.model.clone();
                CortexModel::SequenceClassification {
                    model: sequence_classification::SequenceClassificationModel::new(c.into())?,
                    model_type,
                }
            }
            Self::Summarization(c) => {
                let model_type = c.model.clone();
                CortexModel::Summarization {
                    model: summarization::SummarizationModel::new(c.into())?,
                    model_type,
                }
            }
            Self::TextGeneration(c) => {
                let model_type = c.model.clone();
                CortexModel::TextGeneration {
                    model: text_generation::TextGenerationModel::new(c.into())?,
                    model_type,
                }
            }
            Self::TokenClassification(c) => {
                let model_type = c.model.clone();
                CortexModel::TokenClassification {
                    model: token_classification::TokenClassificationModel::new(c.into())?,
                    model_type,
                }
            }
            Self::Translation(c) => {
                let model_type = c.model.clone();
                let model = translation::TranslationModelBuilder::new()
                    .with_device(c.device.into())
                    .create_model()?;
                CortexModel::Translation { model, model_type }
            }
            Self::ZeroShotClassification(c) => {
                let model_type = c.model.clone();
                CortexModel::ZeroShotClassification {
                    model: zero_shot_classification::ZeroShotClassificationModel::new(c.into())?,
                    model_type,
                }
            }
        })
    }

    /// Returns a reference to the device configuration.
    /// All config variants have this field.
    pub fn device(&self) -> &CortexDevice {
        match self {
            Self::Conversation(c) => &c.device,
            Self::MaskedLanguage(c) => &c.device,
            Self::Ner(c) => &c.device,
            Self::PosTagging(c) => &c.device,
            Self::QuestionAnswering(c) => &c.device,
            Self::SentenceEmbeddings(c) => &c.device,
            Self::Sentiment(c) => &c.device,
            Self::SequenceClassification(c) => &c.device,
            Self::Summarization(c) => &c.device,
            Self::TextGeneration(c) => &c.device,
            Self::TokenClassification(c) => &c.device,
            Self::Translation(c) => &c.device,
            Self::ZeroShotClassification(c) => &c.device,
        }
    }

    /// Returns a reference to the model type.
    /// Returns `None` for SentenceEmbeddings which uses a different model type.
    pub fn model(&self) -> Option<&CortexModelType> {
        match self {
            Self::Conversation(c) => Some(&c.model),
            Self::MaskedLanguage(c) => Some(&c.model),
            Self::Ner(c) => Some(&c.model),
            Self::PosTagging(c) => Some(&c.model),
            Self::QuestionAnswering(c) => Some(&c.model),
            Self::SentenceEmbeddings(_) => None,
            Self::Sentiment(c) => Some(&c.model),
            Self::SequenceClassification(c) => Some(&c.model),
            Self::Summarization(c) => Some(&c.model),
            Self::TextGeneration(c) => Some(&c.model),
            Self::TokenClassification(c) => Some(&c.model),
            Self::Translation(c) => Some(&c.model),
            Self::ZeroShotClassification(c) => Some(&c.model),
        }
    }

    /// Returns a reference to the sentence embeddings model type.
    /// Returns `Some` only for the SentenceEmbeddings variant.
    pub fn sentence_embeddings_model(&self) -> Option<&CortexSentenceEmbeddingsModelType> {
        match self {
            Self::SentenceEmbeddings(c) => Some(&c.model),
            _ => None,
        }
    }

    /// Returns a reference to the model source.
    /// Returns `None` for SentenceEmbeddings which doesn't have a source field.
    pub fn source(&self) -> Option<&CortexModelSource> {
        match self {
            Self::Conversation(c) => Some(&c.source),
            Self::MaskedLanguage(c) => Some(&c.source),
            Self::Ner(c) => Some(&c.source),
            Self::PosTagging(c) => Some(&c.source),
            Self::QuestionAnswering(c) => Some(&c.source),
            Self::SentenceEmbeddings(_) => None,
            Self::Sentiment(c) => Some(&c.source),
            Self::SequenceClassification(c) => Some(&c.source),
            Self::Summarization(c) => Some(&c.source),
            Self::TextGeneration(c) => Some(&c.source),
            Self::TokenClassification(c) => Some(&c.source),
            Self::Translation(c) => Some(&c.source),
            Self::ZeroShotClassification(c) => Some(&c.source),
        }
    }

    pub fn is_conversation(&self) -> bool {
        matches!(self, Self::Conversation(_))
    }

    pub fn is_masked_language(&self) -> bool {
        matches!(self, Self::MaskedLanguage(_))
    }

    pub fn is_ner(&self) -> bool {
        matches!(self, Self::Ner(_))
    }

    pub fn is_pos_tagging(&self) -> bool {
        matches!(self, Self::PosTagging(_))
    }

    pub fn is_question_answering(&self) -> bool {
        matches!(self, Self::QuestionAnswering(_))
    }

    pub fn is_sentence_embeddings(&self) -> bool {
        matches!(self, Self::SentenceEmbeddings(_))
    }

    pub fn is_sentiment(&self) -> bool {
        matches!(self, Self::Sentiment(_))
    }

    pub fn is_sequence_classification(&self) -> bool {
        matches!(self, Self::SequenceClassification(_))
    }

    pub fn is_summarization(&self) -> bool {
        matches!(self, Self::Summarization(_))
    }

    pub fn is_text_generation(&self) -> bool {
        matches!(self, Self::TextGeneration(_))
    }

    pub fn is_token_classification(&self) -> bool {
        matches!(self, Self::TokenClassification(_))
    }

    pub fn is_translation(&self) -> bool {
        matches!(self, Self::Translation(_))
    }

    pub fn is_zero_shot_classification(&self) -> bool {
        matches!(self, Self::ZeroShotClassification(_))
    }
}

impl Default for CortexModelConfig {
    fn default() -> Self {
        Self::Conversation(CortexConversationConfig::default())
    }
}

impl From<CortexConversationConfig> for CortexModelConfig {
    fn from(config: CortexConversationConfig) -> Self {
        Self::Conversation(config)
    }
}

impl From<CortexZeroShotConfig> for CortexModelConfig {
    fn from(config: CortexZeroShotConfig) -> Self {
        Self::ZeroShotClassification(config)
    }
}

impl From<CortexMaskedLanguageConfig> for CortexModelConfig {
    fn from(config: CortexMaskedLanguageConfig) -> Self {
        Self::MaskedLanguage(config)
    }
}

impl From<CortexQuestionAnsweringConfig> for CortexModelConfig {
    fn from(config: CortexQuestionAnsweringConfig) -> Self {
        Self::QuestionAnswering(config)
    }
}

impl From<CortexSentenceEmbeddingsConfig> for CortexModelConfig {
    fn from(config: CortexSentenceEmbeddingsConfig) -> Self {
        Self::SentenceEmbeddings(config)
    }
}

impl From<CortexSentimentConfig> for CortexModelConfig {
    fn from(config: CortexSentimentConfig) -> Self {
        Self::Sentiment(config)
    }
}

impl From<CortexSequenceClassificationConfig> for CortexModelConfig {
    fn from(config: CortexSequenceClassificationConfig) -> Self {
        Self::SequenceClassification(config)
    }
}

impl From<CortexSummarizationConfig> for CortexModelConfig {
    fn from(config: CortexSummarizationConfig) -> Self {
        Self::Summarization(config)
    }
}

impl From<CortexTextGenerationConfig> for CortexModelConfig {
    fn from(config: CortexTextGenerationConfig) -> Self {
        Self::TextGeneration(config)
    }
}

impl From<CortexTokenClassificationConfig> for CortexModelConfig {
    fn from(config: CortexTokenClassificationConfig) -> Self {
        Self::TokenClassification(config)
    }
}

impl From<CortexTranslationConfig> for CortexModelConfig {
    fn from(config: CortexTranslationConfig) -> Self {
        Self::Translation(config)
    }
}

impl From<CortexNerConfig> for CortexModelConfig {
    fn from(config: CortexNerConfig) -> Self {
        Self::Ner(config)
    }
}

impl From<CortexPosTaggingConfig> for CortexModelConfig {
    fn from(config: CortexPosTaggingConfig) -> Self {
        Self::PosTagging(config)
    }
}
