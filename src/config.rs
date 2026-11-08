use crate::storage::atomic_write;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub app: AppConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_color")]
    pub color: bool,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            color: true,
            timeout_seconds: default_timeout(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_color() -> bool {
    true
}

fn default_timeout() -> u64 {
    15
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("ani-es-rs").join("config.toml"))
    }

    pub fn load() -> Self {
        Self::load_checked().unwrap_or_default()
    }

    pub fn load_checked() -> Result<Self> {
        let path = match Self::path() {
            Some(p) => p,
            None => return Ok(Config::default()),
        };

        let data = match fs::read_to_string(&path) {
            Ok(data) => data,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Config::default())
            }
            Err(error) => {
                return Err(error).with_context(|| format!("No se pudo leer {}", path.display()))
            }
        };

        toml::from_str(&data)
            .with_context(|| format!("Configuración inválida en {}", path.display()))
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No se encontró directorio de configuración",
            )
        })?;
        fs::create_dir_all(path.parent().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Ruta de configuración inválida",
            )
        })?)?;
        let data = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        atomic_write(&path, data.as_bytes())
    }
}
