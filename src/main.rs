// src/main.rs

mod config;
mod dncli;
mod output;

use std::path::PathBuf;
use clap::Parser;
use url::Url;
use futures_util::future;
use futures::FutureExt;
use std::pin::Pin;
use crate::output::{FileInfo, DownloadOptions};
use mime_guess;
use mime;
use tokio::fs; 
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    urls: Vec<String>,

    #[arg(short, long)]
    output: Option<PathBuf>, 

    #[arg(short, long, default_value_t = 4)]
    connections: usize,

    #[arg(long)]
    oneshot: bool,

    #[arg(long)]
    hide_progress: bool,

    #[arg(long)]
    progress_template: Option<String>,

    #[arg(long)]
    no_speed: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let app_config = config::load_config().unwrap_or_else(|e| {
        eprintln!("Warning: Could not load configuration: {}. Using default settings.", e);
        config::DncliConfig::default()
    });

    let hide_progress = args.hide_progress || app_config.hide_progress_bar;
    let progress_template = app_config.progress_template.or(args.progress_template);
    let show_download_speed = app_config.show_download_speed && !args.no_speed;

    let download_options = DownloadOptions {
        hide_progress,
        progress_template,
        show_download_speed,
    };

    type DownloadFuture = Pin<Box<dyn futures::Future<Output = Result<FileInfo, dncli::DncliError>>>>;

    let total_urls = args.urls.len();
    let mut download_tasks: Vec<DownloadFuture> = Vec::new();

    let shared_client = Arc::new(reqwest::Client::new()); 

    for (i, url_str_owned) in args.urls.into_iter().enumerate() {
        let mut determined_filename: PathBuf;

        let parsed_url = Url::parse(&url_str_owned)?;
        determined_filename = parsed_url
            .path_segments()
            .and_then(|segments| segments.last())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("download_{}", i)));

        let head_response = shared_client.head(&url_str_owned).send().await;
        if let Ok(response) = head_response {
            if let Some(content_disposition) = response.headers().get("content-disposition") {
                if let Ok(disposition_str) = content_disposition.to_str() {
                    if let Some(filename_match) = disposition_str.find("filename=\"").and_then(|idx| {
                        let start = idx + "filename=\"".len();
                        disposition_str[start..].find('"').map(|end| &disposition_str[start..start + end])
                    }) {
                        determined_filename = PathBuf::from(filename_match);
                    } else if let Some(filename_match) = disposition_str.find("filename=").and_then(|idx| {
                        let start = idx + "filename=".len();
                        disposition_str[start..].split(';').next()
                    }) {
                        determined_filename = PathBuf::from(filename_match.trim());
                    }
                }
            } else if determined_filename.extension().is_none() {
                if let Some(content_type) = response.headers().get("content-type") {
                    if let Ok(content_type_str) = content_type.to_str() {
                        if let Ok(mime_type) = content_type_str.parse::<mime::Mime>() { 
                            if let Some(exts_slice) = mime_guess::get_mime_extensions(&mime_type) { 
                                if let Some(ext_os_str) = exts_slice.first() {
                                    let ext = ext_os_str.to_string(); 
                                    if !ext.is_empty() {
                                        let original_file_stem = determined_filename.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                                        determined_filename = PathBuf::from(format!("{}.{}", original_file_stem, &ext)); // borrow `ext` here
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        let final_output_path_for_task: PathBuf = if let Some(explicit_output_base) = &args.output {
            if let Ok(metadata) = fs::metadata(explicit_output_base).await {
                if metadata.is_dir() {
                    explicit_output_base.join(determined_filename)
                } else {
                    explicit_output_base.clone()
                }
            } else if total_urls > 1 {
                explicit_output_base.with_file_name(
                    format!("{}_{}", explicit_output_base.file_name().unwrap_or_default().to_str().unwrap_or_default(), 
                            determined_filename.to_str().unwrap_or_default())
                )
            }
            else {
                explicit_output_base.clone()
            }
        } else {
            determined_filename
        };

        let final_path_with_fallback = if final_output_path_for_task.file_name().is_none() || final_output_path_for_task.extension().is_none() {
            let mut new_path = final_output_path_for_task.clone();
            new_path.set_extension("bin");
            new_path
        } else {
            final_output_path_for_task
        };

        let client_clone = Arc::clone(&shared_client); 
        let task: DownloadFuture = dncli::download_file(
            client_clone,
            url_str_owned,
            final_path_with_fallback,
            args.connections,
            download_options.clone(),
        ).boxed();

        download_tasks.push(task);
    }

    if args.oneshot {
        println!("Downloading {} files simultaneously...", download_tasks.len());
        future::join_all(download_tasks).await
            .into_iter()
            .for_each(|res| {
                match res {
                    Ok(file_info) => println!("Successfully downloaded: {}", file_info.file_name),
                    Err(e) => eprintln!("Download failed: {}", e),
                }
            });
    } else {
        println!("Downloading {} files sequentially...", download_tasks.len());
        for task in download_tasks {
            match task.await {
                Ok(file_info) => println!("Successfully downloaded: {}", file_info.file_name),
                Err(e) => eprintln!("Download failed: {}", e),
            }
        }
    }

    Ok(())
}

