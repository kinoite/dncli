// src/dncli.rs

use crate::output::{FileInfo, DownloadOptions};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, StatusCode};
use tokio::fs::File;
use tokio::io::{self, AsyncSeekExt, AsyncWriteExt};
use std::path::{Path, PathBuf}; 
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::task;
use futures_util::stream::StreamExt;

#[derive(Debug)]
pub enum DncliError {
    HttpRequest(reqwest::Error),
    Io(io::Error),
    UrlParse(url::ParseError),
    Network(String),
    Other(String),
    Join(tokio::task::JoinError),
    ChannelSendError(String),
}

impl From<reqwest::Error> for DncliError {
    fn from(err: reqwest::Error) -> Self {
        DncliError::HttpRequest(err)
    }
}

impl From<io::Error> for DncliError {
    fn from(err: io::Error) -> Self {
        DncliError::Io(err)
    }
}

impl From<url::ParseError> for DncliError {
    fn from(err: url::ParseError) -> Self {
        DncliError::UrlParse(err)
    }
}

impl From<tokio::task::JoinError> for DncliError {
    fn from(err: tokio::task::JoinError) -> Self {
        DncliError::Join(err)
    }
}

struct ChunkData {
    offset: u64,
    bytes: bytes::Bytes,
}

impl std::fmt::Display for DncliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DncliError::HttpRequest(e) => write!(f, "HTTP request error: {}", e),
            DncliError::Io(e) => write!(f, "I/O error: {}", e),
            DncliError::UrlParse(e) => write!(f, "URL parse error: {}", e),
            DncliError::Network(msg) => write!(f, "Network error: {}", msg),
            DncliError::Other(msg) => write!(f, "Error: {}", msg),
            DncliError::Join(e) => write!(f, "Task join error: {}", e),
            DncliError::ChannelSendError(msg) => write!(f, "Channel send error: {}", msg),
        }
    }
}

impl std::error::Error for DncliError {}

pub async fn download_file(
    url: String,
    output_path: PathBuf,
    connections: usize,
    options: DownloadOptions,
) -> Result<FileInfo, DncliError> {
    let client = Client::new();
    let _parsed_url = url::Url::parse(&url)?;

    let response = client.head(&url).send().await?.error_for_status()?;
    let total_size = response
        .headers()
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let accepts_ranges = response.headers().contains_key("accept-ranges");

    let file_info = FileInfo {
        url: url.to_string(),
        file_name: output_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string(),
        total_size,
    };

    if !accepts_ranges || total_size == 0 || connections == 1 {
        println!("Server does not support byte-range requests or single connection requested. Falling back to single-threaded download.");
        download_single_thread(url, output_path, &file_info, options).await?;
    } else {
        download_multi_thread(url, output_path, total_size, connections, &file_info, options).await?;
    }

    Ok(file_info)
}

async fn download_single_thread(
    url: String,
    output_path: PathBuf,
    _file_info: &FileInfo,
    options: DownloadOptions,
) -> Result<(), DncliError> {
    let client = Client::new();
    let mut response = client.get(&url).send().await?.error_for_status()?;

    let file = Arc::new(Mutex::new(File::create(&output_path).await?));

    let pb = if options.hide_progress {
        ProgressBar::hidden()
    } else {
        let bar = ProgressBar::new(_file_info.total_size);
        let template_str = if let Some(custom_template) = options.progress_template {
            custom_template.to_string()
        } else if options.show_download_speed {
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}) {speed_bytes}/s".to_string()
        } else {
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})".to_string()
        };
        bar.set_style(ProgressStyle::default_bar().template(&template_str).unwrap().progress_chars("#>-"));
        bar
    };

    let mut downloaded_bytes = 0;
    while let Some(chunk) = response.chunk().await? {
        let mut file_guard = file.lock().await;
        file_guard.write_all(&chunk).await?;
        downloaded_bytes += chunk.len() as u64;
        pb.set_position(downloaded_bytes);
    }
    pb.finish_with_message("Download complete!");

    Ok(())
}

async fn download_multi_thread(
    url: String,
    output_path: PathBuf,
    total_size: u64,
    connections: usize,
    _file_info: &FileInfo,
    options: DownloadOptions,
) -> Result<(), DncliError> {
    let client = Arc::new(Client::new());
    let output_file = Arc::new(Mutex::new(File::create(&output_path).await?)); 
    
    let pb = if options.hide_progress {
        ProgressBar::hidden()
    } else {
        let bar = ProgressBar::new(total_size);
        let template_str = if let Some(custom_template) = options.progress_template {
            custom_template.to_string()
        } else if options.show_download_speed {
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}) {speed_bytes}/s".to_string()
        } else {
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})".to_string()
        };
        bar.set_style(ProgressStyle::default_bar().template(&template_str).unwrap().progress_chars("#>-"));
        bar
    };
        
    let (sender, mut receiver) = mpsc::unbounded_channel::<ChunkData>();

    let writer_file_handle = Arc::clone(&output_file);
    let writer_pb = pb.clone();
    let writer_task = tokio::task::spawn(async move {
        let mut file_guard = writer_file_handle.lock().await;
        while let Some(chunk_data) = receiver.recv().await {
            file_guard.seek(io::SeekFrom::Start(chunk_data.offset)).await
                .map_err(|e| DncliError::Io(e))?;
            file_guard.write_all(&chunk_data.bytes).await
                .map_err(|e| DncliError::Io(e))?;
            writer_pb.inc(chunk_data.bytes.len() as u64);
        }
        Ok::<(), DncliError>(())
    });

    let chunk_size = total_size / connections as u64;
    let mut handles = vec![];

    for i in 0..connections {
        let start = i as u64 * chunk_size;
        let end = if i == connections - 1 {
            total_size.saturating_sub(1)
        } else {
            start + chunk_size.saturating_sub(1)
        };

        let client = Arc::clone(&client);
        let url_clone = url.clone(); // Clone the owned String for each task
        let sender_clone = sender.clone();

        let handle = tokio::task::spawn(async move {
            let mut current_start = start;
            let max_retries = 5;
            let mut retries = 0;

            loop {
                // Start building the request
                let mut request_builder = client.get(&url_clone); 
                
                if end > 0 {
                    request_builder = request_builder.header("Range", format!("bytes={}-{}", current_start, end));
                }

                // Build the request and send it
                match request_builder.send().await {
                    Ok(response) => {
                        if response.status() == StatusCode::PARTIAL_CONTENT || response.status() == StatusCode::OK {
                            let mut stream = response.bytes_stream();
                            let mut downloaded_in_chunk = 0;

                            while let Some(chunk_result) = stream.next().await {
                                match chunk_result {
                                    Ok(chunk) => {
                                        let chunk_len = chunk.len() as u64;
                                        sender_clone.send(ChunkData {
                                            offset: current_start + downloaded_in_chunk,
                                            bytes: chunk,
                                        }).map_err(|e| DncliError::ChannelSendError(e.to_string()))?;

                                        downloaded_in_chunk += chunk_len;
                                    }
                                    Err(e) => {
                                        eprintln!("Error downloading chunk in segment {}-{}: {}", start, end, e);
                                        return Err(DncliError::HttpRequest(e));
                                    }
                                }
                            }

                            if current_start + downloaded_in_chunk >= end + 1 || (end == 0 && downloaded_in_chunk > 0) {
                                return Ok(());
                            } else {
                                current_start += downloaded_in_chunk;
                                retries += 1;
                                if retries > max_retries {
                                    return Err(DncliError::Network(format!("Max retries reached for segment {}-{}", start, end)));
                                }
                                tokio::time::sleep(tokio::time::Duration::from_secs(2_u64.pow(retries))).await;
                                eprintln!("Retrying download for segment {}-{}. Attempt {}", start, end, retries);
                            }
                        } else {
                            return Err(DncliError::Network(format!("Unexpected status code for segment {}-{}: {}", start, end, response.status())));
                        }
                    }
                    Err(e) => {
                        retries += 1;
                        if retries > max_retries {
                            return Err(DncliError::Network(format!("Max retries reached for segment {}-{}: {}", start, end, e)));
                        }
                        tokio::time::sleep(tokio::time::Duration::from_secs(2_u64.pow(retries))).await;
                        eprintln!("Retrying connection for segment {}-{}: {}. Attempt {}", start, end, e, retries);
                    }
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await??;
    }

    drop(sender);
    writer_task.await??;

    pb.finish_with_message("Download complete!");
    Ok(())
}

