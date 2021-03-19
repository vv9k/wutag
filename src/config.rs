use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use wutag_core::{Error, Result};

const CONFIG_FILE: &str = ".wutag.yml";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub max_depth: Option<usize>,
    pub colors: Option<Vec<String>>,
}

impl Config {
    /// Loads Config from provided `path` by appending [CONFIG_FILE](CONFIG_FILE) name to it and
    /// reading the file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().join(CONFIG_FILE);
        Ok(serde_yaml::from_slice(&fs::read(path)?)?)
    }

    /// Loads config file from home directory of user executing the program
    pub fn load_default_location() -> Result<Self> {
        Self::load(dirs::home_dir().ok_or(Error::FileNotFound)?)
    }
}
