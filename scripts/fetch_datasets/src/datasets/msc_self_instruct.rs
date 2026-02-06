use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

use crate::sample::{Dataset, Metadata, Sample, calculate_difficulty};
use super::render_download_progress;

const JSONL_URL: &str = "https://huggingface.co/datasets/MemGPT/MSC-Self-Instruct/resolve/main/msc_self_instruct.jsonl";

#[derive(Debug, Deserialize)]
struct MscConversation {
    personas: Option<Vec<Vec<String>>>,
    dialog: Vec<DialogTurn>,
}

#[derive(Debug, Deserialize)]
struct DialogTurn {
    text: Option<String>,
    utterance: Option<String>,
    id: Option<String>,
}

impl DialogTurn {
    fn get_text(&self) -> String {
        self.text
            .clone()
            .or_else(|| self.utterance.clone())
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    fn get_speaker(&self, turn_idx: usize) -> String {
        self.id
            .clone()
            .unwrap_or_else(|| format!("Speaker {}", (turn_idx % 2) + 1))
    }
}

pub async fn download(client: &Client) -> Result<Dataset> {
    let mut dataset = Dataset::new(
        "https://huggingface.co/datasets/MemGPT/MSC-Self-Instruct",
        "MSC-Self-Instruct: Multi-session conversations with persona information",
    );

    // Download JSONL with progress
    let response = client.get(JSONL_URL).send().await?.error_for_status()?;
    let total_size = response.content_length();

    let mut downloaded: u64 = 0;
    let mut data = Vec::new();
    let mut stream = futures::StreamExt::boxed(response.bytes_stream());

    use futures::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        data.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;
        render_download_progress(downloaded, total_size, "Downloading MSC-Self-Instruct...");
    }

    super::DownloadProgress::clear();
    println!("  Processing MSC-Self-Instruct conversations...");

    let text = String::from_utf8(data)?;
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();

    for (conv_idx, line) in lines.iter().enumerate() {
        let conversation: MscConversation = match serde_json::from_str(line) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if conversation.dialog.is_empty() {
            continue;
        }

        // Build persona context
        let persona_context = conversation.personas.as_ref().and_then(|personas| {
            if personas.is_empty() {
                None
            } else {
                Some(
                    personas
                        .iter()
                        .enumerate()
                        .map(|(i, p)| format!("Speaker {}: {}", i + 1, p.join(". ")))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            }
        });

        let mut context = persona_context.clone();

        for (turn_idx, turn) in conversation.dialog.iter().enumerate() {
            let utterance = turn.get_text();
            if utterance.is_empty() {
                continue;
            }

            let speaker = turn.get_speaker(turn_idx);
            let difficulty = calculate_difficulty(&utterance);

            let sample = Sample {
                id: format!("msc_instruct-{}-{}", conv_idx, turn_idx),
                text: utterance.clone(),
                context: context.clone(),
                expected_decision: "accept".to_string(),
                expected_labels: vec!["multi_session".to_string(), "persona".to_string()],
                primary_category: "conversational".to_string(),
                difficulty: difficulty.to_string(),
                metadata: Metadata {
                    source: "MemGPT/MSC-Self-Instruct".to_string(),
                    split: None,
                    conversation_id: conv_idx,
                    turn_id: turn_idx,
                    speaker: Some(speaker.clone()),
                    session_id: None,
                },
            };

            dataset.samples.push(sample);

            // Update context
            let speaker_utterance = format!("{}: {}", speaker, utterance);
            context = match context {
                Some(ctx) => Some(format!("{}\n{}", ctx, speaker_utterance)),
                None => Some(speaker_utterance),
            };
        }
    }

    Ok(dataset)
}
