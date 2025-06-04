// config.rs

use serde::{Serialize, Deserialize};
use confy::{ConfyError, load, store};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DncliConfig {
    pub throttle_limit_kbps: Option<u64>,
}

impl Default for DncliConfig {
    fn default() -> Self {
        DncliConfig {
            throttle_limit_kbps: None,
        }
    }
}

pub fn load_config() -> Result<DncliConfig, ConfyError> {
    load("dncli", None)
}

pub fn save_config(config: &DncliConfig) -> Result<(), ConfyError> {
    store("dncli", None, config)
}

