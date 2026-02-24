use serde::Deserialize;
use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default_format: Option<String>,
    #[serde(default)]
    pub show_versions: bool,
    #[serde(default)]
    pub max_depth: Option<usize>,
}

impl Config {
    pub fn load(_path: Option<&PathBuf>) -> Result<Self> {
        // TODO: implement config loading
        Ok(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.show_versions);
        assert!(config.default_format.is_none());
    }
}
