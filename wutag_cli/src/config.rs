use crate::{Error, Result};

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs, io};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum ConfigError {
    #[error("failed to load configuration - {0}")]
    Load(io::Error),
    #[error("failed to deserialize configuration - {0}")]
    Deserialize(serde_yaml::Error),
    #[error("failed to determine user config directory")]
    FindUserDir,
}

const CONFIG_FILE: &str = "wutag.yml";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub max_depth: Option<usize>,
    pub colors: Option<Vec<String>>,
    #[serde(default)]
    pub pretty_output: bool,
}

impl Config {
    /// Loads Config from provided `path` by appending [CONFIG_FILE](CONFIG_FILE) name to it and
    /// reading the file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().join(CONFIG_FILE);
        serde_yaml::from_slice(&fs::read(path).map_err(ConfigError::Load)?)
            .map_err(ConfigError::Deserialize)
            .map_err(Error::from)
    }

    /// Loads config file from config directory of user executing the program
    pub fn load_default_location() -> Result<Self> {
        Self::load(dirs::config_dir().ok_or_else(|| ConfigError::FindUserDir)?)
    }
}
