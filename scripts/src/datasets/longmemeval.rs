use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use serde_json::Value;

use crate::sample::{calculate_difficulty, Dataset, Metadata, Sample};
use crate::widgets::{DownloadProgress, Widget};

const BASE_URL: &str =
    "https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned/resolve/main";
// Only download the smaller split to reduce download size (~265 MB vs ~2.6 GB)
const SPLITS: [&str; 1] = ["longmemeval_s_cleaned"];

pub async fn download(client: &Client) -> Result<Dataset> {
    let mut dataset = Dataset::new(
        "https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned",
        "LongMemEval: Memory evaluation benchmark with cleaned history sessions",
    );

    for split in SPLITS {
        println!("  Downloading LongMemEval {} split...", split);

        let url = format!("{}/{}.json", BASE_URL, split);

        let response = client.get(&url).send().await?.error_for_status()?;
        let total_size = response.content_length();

        let mut downloaded: u64 = 0;
        let mut data = Vec::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            data.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;
            DownloadProgress::new()
                .downloaded(downloaded)
                .total(total_size)
                .message(format!("Downloading {}.json...", split))
                .render()
                .write();
        }

        DownloadProgress::clear();

        // Parse JSON array
        let text = String::from_utf8(data)?;
        let records: Vec<Value> = serde_json::from_str(&text)?;

        println!("  Processing {} {} records...", records.len(), split);

        for (idx, record) in records.iter().enumerate() {
            // Extract fields flexibly
            let question = record
                .get("question")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if question.is_empty() {
                continue;
            }

            // Build context from haystack/history
            let context = build_context(record);

            // Extract question type if available
            let question_type = record
                .get("question_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mut labels = vec!["memory_eval".to_string()];
            if let Some(qt) = &question_type {
                labels.push(qt.clone());
            }

            let difficulty = calculate_difficulty(question);

            let sample = Sample {
                id: format!("longmemeval-{}-{}", split, idx),
                text: question.to_string(),
                context,
                expected_decision: "accept".to_string(),
                expected_labels: labels,
                primary_category: "memory".to_string(),
                difficulty: difficulty.to_string(),
                metadata: Metadata {
                    source: "xiaowu0162/longmemeval-cleaned".to_string(),
                    split: Some(split.to_string()),
                    conversation_id: idx,
                    turn_id: 0,
                    speaker: None,
                    session_id: None,
                },
            };

            dataset.samples.push(sample);
        }
    }

    Ok(dataset)
}

fn build_context(record: &Value) -> Option<String> {
    // Try different field names for the history/haystack
    let haystack = record
        .get("haystack")
        .or_else(|| record.get("history"))
        .or_else(|| record.get("sessions"))
        .or_else(|| record.get("haystack_sessions"));

    if let Some(h) = haystack {
        if let Some(arr) = h.as_array() {
            // If it's an array of sessions, concatenate them
            let sessions: Vec<String> = arr
                .iter()
                .filter_map(|session| {
                    if let Some(s) = session.as_str() {
                        Some(s.to_string())
                    } else if let Some(turns) = session.as_array() {
                        // Array of turns in a session
                        let turn_strings: Vec<String> = turns
                            .iter()
                            .filter_map(|turn| {
                                if let Some(t) = turn.as_str() {
                                    Some(t.to_string())
                                } else if let Some(obj) = turn.as_object() {
                                    // Turn is an object with role/content
                                    let role = obj
                                        .get("role")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown");
                                    let content = obj
                                        .get("content")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    Some(format!("{}: {}", role, content))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if !turn_strings.is_empty() {
                            Some(turn_strings.join("\n"))
                        } else {
                            None
                        }
                    } else {
                        // Try to serialize as JSON
                        serde_json::to_string(session).ok()
                    }
                })
                .collect();

            if !sessions.is_empty() {
                return Some(sessions.join("\n\n---\n\n"));
            }
        } else if let Some(s) = h.as_str() {
            return Some(s.to_string());
        }
    }

    None
}
