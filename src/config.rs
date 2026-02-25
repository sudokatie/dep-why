use serde::Deserialize;
use std::path::PathBuf;

use crate::cli::OutputFormat;
use crate::error::Result;

/// Configuration per spec Section 8
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    /// Default output format
    #[serde(default)]
    pub format: Option<String>,
    
    /// Default max paths (0 = unlimited with --all)
    #[serde(default)]
    pub max_paths: Option<usize>,
    
    /// Include dev dependencies by default
    #[serde(default)]
    pub include_dev: bool,
    
    /// Color output mode (auto, always, never)
    #[serde(default)]
    pub color: Option<String>,
    
    /// Custom lock file locations
    #[serde(default)]
    pub lock_files: LockFileConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LockFileConfig {
    pub npm: Option<String>,
    pub cargo: Option<String>,
    pub pip: Option<String>,
}

impl Config {
    /// Load config from file or defaults
    pub fn load(path: Option<&PathBuf>) -> Result<Self> {
        // Try explicit path first
        if let Some(p) = path {
            if p.exists() {
                let content = std::fs::read_to_string(p)?;
                let config: Config = toml::from_str(&content)
                    .map_err(|e| crate::error::Error::ConfigError(e.to_string()))?;
                return Ok(config);
            }
        }
        
        // Try .dep-why.toml in current directory
        let local_config = PathBuf::from(".dep-why.toml");
        if local_config.exists() {
            let content = std::fs::read_to_string(&local_config)?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| crate::error::Error::ConfigError(e.to_string()))?;
            return Ok(config);
        }
        
        // Try ~/.config/dep-why/config.toml
        if let Some(config_dir) = dirs_next::config_dir() {
            let global_config = config_dir.join("dep-why").join("config.toml");
            if global_config.exists() {
                let content = std::fs::read_to_string(&global_config)?;
                let config: Config = toml::from_str(&content)
                    .map_err(|e| crate::error::Error::ConfigError(e.to_string()))?;
                return Ok(config);
            }
        }
        
        // Apply environment variable overrides to default config
        let mut config = Config::default();
        config.apply_env_overrides();
        
        Ok(config)
    }
    
    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        if let Ok(format) = std::env::var("DEP_WHY_FORMAT") {
            self.format = Some(format);
        }
        
        if let Ok(max_paths) = std::env::var("DEP_WHY_MAX_PATHS") {
            if let Ok(n) = max_paths.parse() {
                self.max_paths = Some(n);
            }
        }
        
        // Per spec Section 8.2: DEP_WHY_COLOR (auto, always, never)
        if let Ok(color) = std::env::var("DEP_WHY_COLOR") {
            self.color = Some(color);
        }
    }
    
    /// Get color mode (auto, always, never)
    pub fn color_mode(&self) -> ColorMode {
        match self.color.as_deref() {
            Some("always") => ColorMode::Always,
            Some("never") => ColorMode::Never,
            _ => ColorMode::Auto,
        }
    }
    
    /// Get the output format, respecting config and env
    pub fn output_format(&self) -> Option<OutputFormat> {
        self.format.as_ref().and_then(|s| match s.to_lowercase().as_str() {
            "tree" => Some(OutputFormat::Tree),
            "json" => Some(OutputFormat::Json),
            "mermaid" => Some(OutputFormat::Mermaid),
            _ => None,
        })
    }
    
    /// Get max paths (default 5)
    pub fn max_paths(&self) -> usize {
        self.max_paths.unwrap_or(5)
    }
}

/// Color output mode per spec Section 8.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.include_dev);
        assert!(config.format.is_none());
        assert!(config.max_paths.is_none());
    }

    #[test]
    fn test_load_local_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".dep-why.toml");
        
        fs::write(&config_path, r#"
format = "json"
max_paths = 10
include_dev = true
"#).unwrap();
        
        // Change to temp dir
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        
        let config = Config::load(None).unwrap();
        
        // Restore dir
        std::env::set_current_dir(original_dir).unwrap();
        
        assert_eq!(config.format, Some("json".to_string()));
        assert_eq!(config.max_paths, Some(10));
        assert!(config.include_dev);
    }

    #[test]
    fn test_load_explicit_path() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("custom.toml");
        
        fs::write(&config_path, r#"
format = "mermaid"
"#).unwrap();
        
        let config = Config::load(Some(&config_path)).unwrap();
        
        assert_eq!(config.format, Some("mermaid".to_string()));
    }

    #[test]
    fn test_output_format_parsing() {
        let mut config = Config::default();
        
        config.format = Some("json".to_string());
        assert!(matches!(config.output_format(), Some(OutputFormat::Json)));
        
        config.format = Some("tree".to_string());
        assert!(matches!(config.output_format(), Some(OutputFormat::Tree)));
        
        config.format = Some("mermaid".to_string());
        assert!(matches!(config.output_format(), Some(OutputFormat::Mermaid)));
        
        config.format = Some("invalid".to_string());
        assert!(config.output_format().is_none());
    }

    #[test]
    fn test_max_paths_default() {
        let config = Config::default();
        assert_eq!(config.max_paths(), 5);
    }

    #[test]
    fn test_env_override() {
        // Save and clear existing env
        let old_format = std::env::var("DEP_WHY_FORMAT").ok();
        let old_max = std::env::var("DEP_WHY_MAX_PATHS").ok();
        
        std::env::set_var("DEP_WHY_FORMAT", "json");
        std::env::set_var("DEP_WHY_MAX_PATHS", "20");
        
        let mut config = Config::default();
        config.apply_env_overrides();
        
        assert_eq!(config.format, Some("json".to_string()));
        assert_eq!(config.max_paths, Some(20));
        
        // Restore env
        if let Some(v) = old_format {
            std::env::set_var("DEP_WHY_FORMAT", v);
        } else {
            std::env::remove_var("DEP_WHY_FORMAT");
        }
        if let Some(v) = old_max {
            std::env::set_var("DEP_WHY_MAX_PATHS", v);
        } else {
            std::env::remove_var("DEP_WHY_MAX_PATHS");
        }
    }
    
    #[test]
    fn test_color_mode_default() {
        let config = Config::default();
        assert_eq!(config.color_mode(), ColorMode::Auto);
    }
    
    #[test]
    fn test_color_mode_always() {
        let mut config = Config::default();
        config.color = Some("always".to_string());
        assert_eq!(config.color_mode(), ColorMode::Always);
    }
    
    #[test]
    fn test_color_mode_never() {
        let mut config = Config::default();
        config.color = Some("never".to_string());
        assert_eq!(config.color_mode(), ColorMode::Never);
    }
    
    #[test]
    fn test_color_env_override() {
        let old_color = std::env::var("DEP_WHY_COLOR").ok();
        
        std::env::set_var("DEP_WHY_COLOR", "never");
        
        let mut config = Config::default();
        config.apply_env_overrides();
        
        assert_eq!(config.color, Some("never".to_string()));
        assert_eq!(config.color_mode(), ColorMode::Never);
        
        // Restore env
        if let Some(v) = old_color {
            std::env::set_var("DEP_WHY_COLOR", v);
        } else {
            std::env::remove_var("DEP_WHY_COLOR");
        }
    }
}
