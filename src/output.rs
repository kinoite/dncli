// src/output.rs

pub struct FileInfo {
    pub url: String,
    pub file_name: String,
    pub total_size: u64,
}

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub hide_progress: bool,
    pub progress_template: Option<String>,
    pub show_download_speed: bool,
}

