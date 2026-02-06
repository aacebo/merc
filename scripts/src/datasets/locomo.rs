use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::sample::{calculate_difficulty, Dataset, Metadata, Sample};
use crate::widgets::{DownloadProgress, Widget};

const DATA_URL: &str =
    "https://huggingface.co/datasets/Percena/locomo-mc10/resolve/main/data/locomo_mc10.json";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LocomoRecord {
    question_id: String,
    question: String,
    question_type: Option<String>,
    answer: Option<Value>,
    #[serde(default)]
    choices: Vec<String>,
    #[serde(default)]
    haystack_sessions: Vec<Value>,
    #[serde(default)]
    haystack_session_summaries: Vec<String>,
}

pub async fn download(client: &Client) -> Result<Dataset> {
    let mut dataset = Dataset::new(
        "https://huggingface.co/datasets/Percena/locomo-mc10",
        "LoCoMo: Long conversation memory multiple-choice benchmark",
    );

    println!("  Downloading LoCoMo dataset...");

    let response = client.get(DATA_URL).send().await?.error_for_status()?;
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
            .message("Downloading locomo_mc10.json...")
            .render()
            .write();
    }

    DownloadProgress::clear();

    // Parse JSON - could be array, JSONL, or object
    let text = String::from_utf8(data)?;

    // Try parsing as JSON array first
    let records: Vec<LocomoRecord> = if let Ok(arr) = serde_json::from_str::<Vec<LocomoRecord>>(&text) {
        arr
    } else if let Ok(json) = serde_json::from_str::<Value>(&text) {
        // Try parsing as single object or object with data field
        if let Some(arr) = json.get("data").and_then(|v| v.as_array()) {
            serde_json::from_value(Value::Array(arr.clone()))?
        } else if let Some(arr) = json.get("train").and_then(|v| v.as_array()) {
            serde_json::from_value(Value::Array(arr.clone()))?
        } else if let Some(obj) = json.as_object() {
            // Map of id -> record
            let arr: Vec<Value> = obj.values().cloned().collect();
            serde_json::from_value(Value::Array(arr))?
        } else {
            anyhow::bail!("Unexpected JSON structure");
        }
    } else {
        // Try JSONL format (newline-delimited JSON)
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str::<LocomoRecord>(line).ok())
            .collect()
    };

    println!("  Processing {} LoCoMo records...", records.len());

    for (idx, record) in records.iter().enumerate() {
        if record.question.is_empty() {
            continue;
        }

        // Build context from haystack sessions
        let context = build_context(&record.haystack_sessions, &record.haystack_session_summaries);

        // Build labels
        let mut labels = vec!["memory_eval".to_string(), "multiple_choice".to_string()];
        if let Some(qt) = &record.question_type {
            labels.push(qt.clone());
        }

        let difficulty = calculate_difficulty(&record.question);

        let sample = Sample {
            id: format!("locomo-{}", record.question_id),
            text: record.question.clone(),
            context,
            expected_decision: "accept".to_string(),
            expected_labels: labels,
            primary_category: "memory".to_string(),
            difficulty: difficulty.to_string(),
            metadata: Metadata {
                source: "Percena/locomo-mc10".to_string(),
                split: Some("train".to_string()),
                conversation_id: idx,
                turn_id: 0,
                speaker: None,
                session_id: None,
            },
        };

        dataset.samples.push(sample);
    }

    Ok(dataset)
}

fn build_context(sessions: &[Value], summaries: &[String]) -> Option<String> {
    let mut parts = Vec::new();

    // Add session summaries if available
    if !summaries.is_empty() {
        let summary_text = summaries
            .iter()
            .enumerate()
            .map(|(i, s)| format!("Session {}: {}", i + 1, s))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(format!("## Session Summaries\n{}", summary_text));
    }

    // Add conversation sessions
    if !sessions.is_empty() {
        let sessions_text: Vec<String> = sessions
            .iter()
            .enumerate()
            .filter_map(|(i, session)| {
                if let Some(turns) = session.as_array() {
                    let turn_strings: Vec<String> = turns
                        .iter()
                        .filter_map(|turn| {
                            if let Some(obj) = turn.as_object() {
                                let role = obj
                                    .get("role")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown");
                                let content =
                                    obj.get("content").and_then(|v| v.as_str()).unwrap_or("");
                                Some(format!("{}: {}", role, content))
                            } else if let Some(s) = turn.as_str() {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();

                    if !turn_strings.is_empty() {
                        Some(format!("### Session {}\n{}", i + 1, turn_strings.join("\n")))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if !sessions_text.is_empty() {
            parts.push(format!("## Conversations\n{}", sessions_text.join("\n\n")));
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n\n"))
    }
}
