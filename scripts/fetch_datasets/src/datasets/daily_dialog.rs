use std::collections::HashMap;
use std::io::{BufReader, Read};

use anyhow::Result;
use reqwest::Client;
use tempfile::TempDir;
use zip::ZipArchive;

use crate::sample::{Dataset, Metadata, Sample, calculate_difficulty};
use crate::widgets::{DownloadProgress, Widget};

const BASE_URL: &str = "https://huggingface.co/datasets/roskoN/dailydialog/resolve/main";
const SPLITS: [&str; 3] = ["train", "validation", "test"];

// Emotion mapping
fn emotion_label(code: &str) -> Option<&'static str> {
    match code {
        "0" => None,          // no emotion
        "1" => Some("anger"),
        "2" => Some("disgust"),
        "3" => Some("fear"),
        "4" => Some("joy"),
        "5" => Some("sad"),
        "6" => Some("surprise"),
        _ => None,
    }
}

// Dialog act mapping
fn act_label(code: &str) -> Option<&'static str> {
    match code {
        "1" => Some("inform"),
        "2" => Some("question"),
        "3" => Some("directive"),
        "4" => Some("commissive"),
        _ => None,
    }
}

// Map dialog acts to primary categories
fn act_to_category(act: Option<&str>) -> &'static str {
    match act {
        Some("inform") | Some("question") => "factual",
        Some("directive") => "task",
        Some("commissive") => "decision",
        _ => "factual",
    }
}

// Check if emotion is negative
fn is_negative_emotion(emotion: Option<&str>) -> bool {
    matches!(emotion, Some("anger") | Some("disgust") | Some("fear") | Some("sad"))
}

// Check if emotion is positive
fn is_positive_emotion(emotion: Option<&str>) -> bool {
    matches!(emotion, Some("joy") | Some("surprise"))
}

pub async fn download(client: &Client) -> Result<Dataset> {
    let mut dataset = Dataset::new(
        "https://huggingface.co/datasets/roskoN/dailydialog",
        "DailyDialog: Multi-turn dialogues with emotion and dialog act labels",
    );

    let temp_dir = TempDir::new()?;

    for split in SPLITS {
        println!("  Downloading DailyDialog {} split...", split);

        // Download ZIP file
        let zip_url = format!("{}/{}.zip", BASE_URL, split);
        let zip_path = temp_dir.path().join(format!("{}.zip", split));

        let response = client.get(&zip_url).send().await?.error_for_status()?;
        let total_size = response.content_length();

        let mut downloaded: u64 = 0;
        let mut data = Vec::new();

        use futures::StreamExt;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            data.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;
            DownloadProgress::new()
                .downloaded(downloaded)
                .total(total_size)
                .message(format!("Downloading {}.zip...", split))
                .render()
                .write();
        }

        DownloadProgress::clear();

        // Write ZIP to temp file
        std::fs::write(&zip_path, &data)?;

        // Extract ZIP
        let file = std::fs::File::open(&zip_path)?;
        let mut archive = ZipArchive::new(BufReader::new(file))?;

        // Find and read files from archive
        let mut dialog_text = None;
        let mut act_text = None;
        let mut emotion_text = None;

        // First, collect file names and data
        let mut files: HashMap<String, String> = HashMap::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if file.is_file() {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                files.insert(name, content);
            }
        }

        // Find the right files
        for (name, content) in &files {
            let lower_name = name.to_lowercase();

            if lower_name.contains("dialogues") && !lower_name.contains("act") && !lower_name.contains("emotion") {
                dialog_text = Some(content.clone());
            } else if lower_name.contains("act") {
                act_text = Some(content.clone());
            } else if lower_name.contains("emotion") {
                emotion_text = Some(content.clone());
            }
        }

        // Fallback: try other naming patterns
        if dialog_text.is_none() {
            for (name, content) in &files {
                if name.ends_with(".txt") && !name.contains("act") && !name.contains("emotion") {
                    dialog_text = Some(content.clone());
                    break;
                }
            }
        }

        let dialog_text = match dialog_text {
            Some(t) => t,
            None => {
                println!("  Warning: No dialog file found in {} split", split);
                continue;
            }
        };

        // Parse dialogs
        let dialogs: Vec<&str> = dialog_text.lines().filter(|l| !l.is_empty()).collect();
        let acts: Vec<&str> = act_text
            .as_ref()
            .map(|t| t.lines().filter(|l| !l.is_empty()).collect())
            .unwrap_or_default();
        let emotions: Vec<&str> = emotion_text
            .as_ref()
            .map(|t| t.lines().filter(|l| !l.is_empty()).collect())
            .unwrap_or_default();

        println!("  Processing {} {} conversations...", dialogs.len(), split);

        for (conv_idx, dialog_line) in dialogs.iter().enumerate() {
            let utterances: Vec<&str> = dialog_line
                .split("__eou__")
                .map(|u| u.trim())
                .filter(|u| !u.is_empty())
                .collect();

            let conv_acts: Vec<&str> = acts
                .get(conv_idx)
                .map(|line| line.split_whitespace().collect())
                .unwrap_or_default();

            let conv_emotions: Vec<&str> = emotions
                .get(conv_idx)
                .map(|line| line.split_whitespace().collect())
                .unwrap_or_default();

            let mut context: Option<String> = None;

            for (turn_idx, utterance) in utterances.iter().enumerate() {
                let mut labels = Vec::new();

                // Map emotion
                let emotion = conv_emotions
                    .get(turn_idx)
                    .and_then(|&e| emotion_label(e));

                if let Some(e) = emotion {
                    labels.push(e.to_string());
                    if is_negative_emotion(Some(e)) {
                        labels.push("negative".to_string());
                    } else if is_positive_emotion(Some(e)) {
                        labels.push("positive".to_string());
                    }
                }

                // Map dialog act
                let act = conv_acts.get(turn_idx).and_then(|&a| act_label(a));
                if let Some(a) = act {
                    labels.push(a.to_string());
                }

                // Determine primary category
                let primary_category = if emotion.is_some() {
                    "emotional"
                } else {
                    act_to_category(act)
                };

                // Default to neutral if no labels
                if labels.is_empty() {
                    labels.push("neutral".to_string());
                }

                let difficulty = calculate_difficulty(utterance);

                let sample = Sample {
                    id: format!("daily_dialog-{}-{}-{}", split, conv_idx, turn_idx),
                    text: utterance.to_string(),
                    context: context.clone(),
                    expected_decision: "accept".to_string(),
                    expected_labels: labels,
                    primary_category: primary_category.to_string(),
                    difficulty: difficulty.to_string(),
                    metadata: Metadata {
                        source: "daily_dialog".to_string(),
                        split: Some(split.to_string()),
                        conversation_id: conv_idx,
                        turn_id: turn_idx,
                        speaker: None,
                        session_id: None,
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

    Ok(dataset)
}
