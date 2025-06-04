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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    urls: Vec<String>,

    #[arg(short, long, default_value = "output.bin")]
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
    let progress_template = args.progress_template.or(app_config.progress_template);
    let show_download_speed = app_config.show_download_speed && !args.no_speed;

    let download_options = DownloadOptions {
        hide_progress,
        progress_template,
        show_download_speed,
    };

    type DownloadFuture = Pin<Box<dyn futures::Future<Output = Result<FileInfo, dncli::DncliError>>>>;

    let total_urls = args.urls.len();
    let mut download_tasks: Vec<DownloadFuture> = Vec::new();

    for (i, url_str_owned) in args.urls.into_iter().enumerate() {
        let current_output_path_owned = if let Some(output_base) = &args.output {
            if total_urls > 1 && output_base == &PathBuf::from("output.bin") {
                 let parsed_url = Url::parse(&url_str_owned)?;
                 let filename_from_url = parsed_url
                     .path_segments()
                     .and_then(|segments| segments.last())
                     .filter(|s| !s.is_empty())
                     .map(PathBuf::from);

                 if let Some(mut file_name) = filename_from_url {
                     if total_urls > 1 {
                         file_name.set_extension(format!("{}.{}", i, file_name.extension().unwrap_or_default().to_str().unwrap_or_default()));
                     }
                     PathBuf::from(file_name)
                 } else {
                     PathBuf::from(format!("output_{}.bin", i))
                 }
            } else {
                 output_base.clone()
            }
        } else {
            let parsed_url = Url::parse(&url_str_owned)?;
            parsed_url
                .path_segments()
                .and_then(|segments| segments.last())
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("output.bin"))
        };

        // Simplified logic: always call dncli::download_file
        let task: DownloadFuture = dncli::download_file(
            url_str_owned, // Pass owned String
            current_output_path_owned, // Pass owned PathBuf
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

