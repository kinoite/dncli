// src/config.rs

use serde::{Serialize, Deserialize};
use confy::{ConfyError, load, store};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DncliConfig {
    pub hide_progress_bar: bool,
    pub progress_template: Option<String>,
    pub show_download_speed: bool,
}

impl Default for DncliConfig {
    fn default() -> Self {
        DncliConfig {
            hide_progress_bar: false,
            progress_template: None,
            show_download_speed: true,
        }
    }
}

pub fn load_config() -> Result<DncliConfig, ConfyError> {
    load("dncli", None)
}

pub fn save_config(config: &DncliConfig) -> Result<(), ConfyError> {
    store("dncli", None, config)
}

