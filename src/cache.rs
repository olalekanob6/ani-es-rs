use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub value: String,
    pub expires_at: u64,
}

impl CacheEntry {
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.expires_at > now
    }
}

pub struct ScraperCache {
    path: PathBuf,
    data: HashMap<String, CacheEntry>,
}

impl ScraperCache {
    pub fn new() -> Self {
        let path = cache_dir();
        let data = load(&path);
        Self { path, data }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|entry| {
            if entry.is_valid() {
                Some(entry.value.as_str())
            } else {
                None
            }
        })
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>, ttl_secs: u64) {
        let key = key.into();
        let value = value.into();
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs().saturating_add(ttl_secs))
            .unwrap_or(0);

        self.data
            .insert(key.clone(), CacheEntry { value, expires_at });
        self.persist();
    }

    pub fn invalidate(&mut self, key: &str) {
        self.data.remove(key);
        self.persist();
    }

    pub fn clean_expired(&mut self) {
        let before = self.data.len();
        self.data.retain(|_, entry| entry.is_valid());
        if self.data.len() != before {
            self.persist();
        }
    }

    fn persist(&self) {
        if let Ok(data) = serde_json::to_string(&self.data) {
            if let Err(error) = crate::storage::atomic_write(&self.path, data.as_bytes()) {
                crate::logger::warn(&format!("No se pudo guardar la caché: {error}"));
            }
        }
    }
}

impl Default for ScraperCache {
    fn default() -> Self {
        Self::new()
    }
}

fn cache_dir() -> PathBuf {
    crate::storage::app_data_dir()
        .unwrap_or_else(|_| PathBuf::from(".").join("ani-es-rs"))
        .join("cache")
        .join("scraper.json")
}

fn load(path: &PathBuf) -> HashMap<String, CacheEntry> {
    match fs::read_to_string(path) {
        Ok(data) => match serde_json::from_str(&data) {
            Ok(map) => map,
            Err(error) => {
                crate::logger::warn(&format!("La caché está dañada y se ignorará: {error}"));
                HashMap::new()
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
        Err(error) => {
            crate::logger::warn(&format!("No se pudo leer la caché: {error}"));
            HashMap::new()
        }
    }
}
