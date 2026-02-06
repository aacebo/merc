use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

use crate::sample::{Dataset, Metadata, Sample, calculate_difficulty};
use crate::widgets::{Spinner, Widget};

const SPLITS: [&str; 3] = ["train", "validation", "test"];

#[derive(Debug, Deserialize)]
struct HuggingFaceResponse {
    rows: Option<Vec<RowWrapper>>,
}

#[derive(Debug, Deserialize)]
struct RowWrapper {
    row: Row,
}

#[derive(Debug, Deserialize)]
struct Row {
    dialogue: Option<Vec<String>>,
    dialog: Option<Vec<String>>,
    speaker: Option<Vec<String>>,
    session_id: Option<i64>,
}

impl Row {
    fn get_dialog(&self) -> Vec<String> {
        self.dialogue
            .clone()
            .or_else(|| self.dialog.clone())
            .unwrap_or_default()
    }
}

pub async fn download(client: &Client) -> Result<Dataset> {
    let mut dataset = Dataset::new(
        "https://huggingface.co/datasets/nayohan/multi_session_chat",
        "Multi-Session Chat: Conversations across multiple sessions with persona information",
    );

    // Try first_rows endpoint first
    for split in SPLITS {
        let spinner_frame = 0usize;
        Spinner::new()
            .message(format!("Fetching Multi-Session Chat {} split...", split))
            .frame(spinner_frame)
            .render()
            .write();

        let url = format!(
            "https://datasets-server.huggingface.co/first-rows?dataset=nayohan/multi_session_chat&config=default&split={}",
            split
        );

        let response = match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            _ => {
                Spinner::clear();
                println!("  Failed to fetch {} split", split);
                continue;
            }
        };

        let data: HuggingFaceResponse = match response.json().await {
            Ok(d) => d,
            Err(_) => continue,
        };

        Spinner::clear();

        let rows = data.rows.unwrap_or_default();
        println!("  Processing {} {} conversations...", rows.len(), split);

        for (idx, row_wrapper) in rows.iter().enumerate() {
            let row = &row_wrapper.row;
            let dialog = row.get_dialog();
            let speakers = row.speaker.clone().unwrap_or_default();
            let session_id = row.session_id.unwrap_or(1);

            let mut context: Option<String> = None;

            for (turn_idx, utterance) in dialog.iter().enumerate() {
                let utterance = utterance.trim();
                if utterance.is_empty() {
                    continue;
                }

                let speaker = speakers
                    .get(turn_idx)
                    .cloned()
                    .unwrap_or_else(|| if turn_idx % 2 == 0 { "A".to_string() } else { "B".to_string() });

                let difficulty = calculate_difficulty(utterance);

                let sample = Sample {
                    id: format!("msc-{}-{}-{}", split, idx, turn_idx),
                    text: utterance.to_string(),
                    context: context.clone(),
                    expected_decision: "accept".to_string(),
                    expected_labels: vec!["multi_session".to_string()],
                    primary_category: "conversational".to_string(),
                    difficulty: difficulty.to_string(),
                    metadata: Metadata {
                        source: "nayohan/multi_session_chat".to_string(),
                        split: Some(split.to_string()),
                        conversation_id: idx,
                        turn_id: turn_idx,
                        speaker: Some(speaker),
                        session_id: Some(session_id.to_string()),
                    },
                };

                dataset.samples.push(sample);

                // Update context
                context = match context {
                    Some(ctx) => Some(format!("{}\n{}", ctx, utterance)),
                    None => Some(utterance.to_string()),
                };
            }
        }
    }

    // If we got few samples, try paginated endpoint
    if dataset.samples.len() < 100 {
        println!("  Trying paginated rows endpoint...");

        for split in SPLITS {
            let mut offset = 0;
            let length = 100;

            while offset < 1000 {
                Spinner::new()
                    .message(format!(
                        "Fetching {} split (offset {})...",
                        split, offset
                    ))
                    .frame((offset / 100) % 10)
                    .render()
                    .write();

                let url = format!(
                    "https://datasets-server.huggingface.co/rows?dataset=nayohan/multi_session_chat&config=default&split={}&offset={}&length={}",
                    split, offset, length
                );

                let response = match client.get(&url).send().await {
                    Ok(r) if r.status().is_success() => r,
                    _ => break,
                };

                let data: HuggingFaceResponse = match response.json().await {
                    Ok(d) => d,
                    Err(_) => break,
                };

                Spinner::clear();

                let rows = data.rows.unwrap_or_default();
                if rows.is_empty() {
                    break;
                }

                for (idx, row_wrapper) in rows.iter().enumerate() {
                    let row = &row_wrapper.row;
                    let dialog = row.get_dialog();
                    let speakers = row.speaker.clone().unwrap_or_default();
                    let session_id = row.session_id.unwrap_or(1);

                    let mut context: Option<String> = None;

                    for (turn_idx, utterance) in dialog.iter().enumerate() {
                        let utterance = utterance.trim();
                        if utterance.is_empty() {
                            continue;
                        }

                        let speaker = speakers
                            .get(turn_idx)
                            .cloned()
                            .unwrap_or_else(|| {
                                if turn_idx % 2 == 0 {
                                    "A".to_string()
                                } else {
                                    "B".to_string()
                                }
                            });

                        let difficulty = calculate_difficulty(utterance);

                        let sample = Sample {
                            id: format!("msc-{}-{}-{}", split, offset + idx, turn_idx),
                            text: utterance.to_string(),
                            context: context.clone(),
                            expected_decision: "accept".to_string(),
                            expected_labels: vec!["multi_session".to_string()],
                            primary_category: "conversational".to_string(),
                            difficulty: difficulty.to_string(),
                            metadata: Metadata {
                                source: "nayohan/multi_session_chat".to_string(),
                                split: Some(split.to_string()),
                                conversation_id: offset + idx,
                                turn_id: turn_idx,
                                speaker: Some(speaker),
                                session_id: Some(session_id.to_string()),
                            },
                        };

                        dataset.samples.push(sample);

                        context = match context {
                            Some(ctx) => Some(format!("{}\n{}", ctx, utterance)),
                            None => Some(utterance.to_string()),
                        };
                    }
                }

                offset += length;
                println!("  Fetched {} samples so far...", dataset.samples.len());
            }
        }
    }

    Ok(dataset)
}
