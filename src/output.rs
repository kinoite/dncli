// output.rs

use indicatif::{ProgressBar, ProgressStyle};

pub struct FileInfo {
    pub url: String,
    pub file_name: String,
    pub total_size: u64,
}

pub fn print_welcome() {
    println!("--- dncli: A Fast and Reliable Download Client ---");
}

pub fn print_download_start(file_info: &FileInfo) {
    println!("Downloading: {}", file_info.url);
    println!("Saving to: {}", file_info.file_name);
    println!("Total size: {}", format_bytes(file_info.total_size));
}

pub fn print_download_complete(file_info: &FileInfo) {
    println!("\nDownload of '{}' complete!", file_info.file_name);
    println!("Total size downloaded: {}", format_bytes(file_info.total_size));
}

pub fn print_error(message: &str) {
    eprintln!("\nError: {}", message);
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1_000 {
        format!("{} B", bytes)
    } else if bytes < 1_000_000 {
        format!("{:.2} KB", bytes as f64 / 1_024.0)
    } else if bytes < 1_000_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    }
}

