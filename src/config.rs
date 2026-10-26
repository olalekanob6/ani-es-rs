use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub app: AppConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub color: bool,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_timeout() -> u64 {
    15
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("ani-es-rs").join("config.toml"))
    }

    pub fn load() -> Self {
        let path = match Self::path() {
            Some(p) => p,
            None => return Config::default(),
        };

        let data = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(_) => return Config::default(),
        };

        match toml::from_str(&data) {
            Ok(cfg) => cfg,
            Err(_) => Config::default(),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path().expect("config dir not available");
        fs::create_dir_all(path.parent().expect("config parent"))?;
        let data = toml::to_string_pretty(self).expect("serialize config");
        fs::write(path, data)
    }
}
