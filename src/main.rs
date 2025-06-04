// src/main.rs

mod config;
mod dncli;
mod output;

use std::path::PathBuf;
use clap::Parser;
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    url: String,

    #[arg(short, long, default_value = "output.bin")]
    output: PathBuf,

    #[arg(short, long, default_value_t = 4)]
    connections: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let app_config = config::load_config().unwrap_or_else(|e| {
        eprintln!("Warning: Could not load configuration: {}. Using default settings.", e);
        config::DncliConfig::default()
    });

    let final_output_path = if args.output == PathBuf::from("output.bin") {
        let parsed_url = Url::parse(&args.url)?;
        let filename_from_url = parsed_url
            .path_segments()
            .and_then(|segments| segments.last())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);

        filename_from_url.unwrap_or(args.output)
    } else 
        args.output
    };

    match dncli::download_file(
        &args.url,
        &final_output_path,
        args.connections,
    ).await {
        Ok(file_info) => {
            println!("Successfully downloaded: {}", file_info.file_name);
            println!("Total size: {} bytes", file_info.total_size);
        }
        Err(e) => {
            eprintln!("Download failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
