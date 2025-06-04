// main.rs

mod dncli;
mod output;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about = "A fast and reliable download client written in Rust", long_about = None)]
struct Args {
    /// URL to download
    #[clap(short, long, value_parser)]
    url: String,

    /// Output file name
    #[clap(short, long, value_parser)]
    output: Option<PathBuf>,

    /// Number of concurrent connections/parts for downloading
    #[clap(short, long, value_parser, default_value_t = 4)]
    connections: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    output::print_welcome();

    let output_path = args.output.unwrap_or_else(|| {
        let url_path = url::Url::parse(&args.url)
            .ok()
            .and_then(|u| u.path_segments().map(|s| s.last().map(|seg| PathBuf::from(seg))))
            .flatten()
            .unwrap_or_else(|| "downloaded_file".into());
        url_path
    });

    match dncli::download_file(&args.url, &output_path, args.connections).await {
        Ok(file_info) => {
            output::print_download_complete(&file_info);
        }
        Err(e) => {
            output::print_error(&format!("Download failed: {}", e));
            std::process::exit(1);
        }
    }

    Ok(())
}

