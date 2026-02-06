pub mod daily_dialog;
pub mod msc_self_instruct;
pub mod multi_session_chat;

use std::path::Path;

use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use tokio::io::AsyncWriteExt;

use crate::widgets::{DownloadProgress, Widget};

/// Build a shared HTTP client with appropriate headers
pub fn build_client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent("fetch_datasets/1.0 (Rust; +https://github.com/aacebo/loom)")
        .build()?)
}

/// Download a file with progress reporting
#[allow(dead_code)]
pub async fn download_file<F>(
    client: &Client,
    url: &str,
    dest: &Path,
    on_progress: F,
) -> Result<()>
where
    F: Fn(u64, Option<u64>),
{
    let response = client.get(url).send().await?.error_for_status()?;
    let total_size = response.content_length();

    let mut file = tokio::fs::File::create(dest).await?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total_size);
    }

    file.flush().await?;
    Ok(())
}

/// Download text content with progress reporting
#[allow(dead_code)]
pub async fn download_text<F>(client: &Client, url: &str, on_progress: F) -> Result<String>
where
    F: Fn(u64, Option<u64>),
{
    let response = client.get(url).send().await?.error_for_status()?;
    let total_size = response.content_length();

    let mut downloaded: u64 = 0;
    let mut data = Vec::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        data.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total_size);
    }

    Ok(String::from_utf8(data)?)
}

/// Download JSON content
#[allow(dead_code)]
pub async fn download_json<T, F>(client: &Client, url: &str, on_progress: F) -> Result<T>
where
    T: serde::de::DeserializeOwned,
    F: Fn(u64, Option<u64>),
{
    let text = download_text(client, url, on_progress).await?;
    Ok(serde_json::from_str(&text)?)
}

/// Helper to render download progress
pub fn render_download_progress(downloaded: u64, total: Option<u64>, message: &str) {
    DownloadProgress::new()
        .downloaded(downloaded)
        .total(total)
        .message(message)
        .render()
        .write();
}
