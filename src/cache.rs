use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
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
            .map(|d| d.as_secs() + ttl_secs)
            .unwrap_or(0);

        self.data.insert(
            key.clone(),
            CacheEntry {
                value,
                expires_at,
            },
        );
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
            if let Some(parent) = self.path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&self.path)
                .and_then(|mut f| f.write_all(data.as_bytes()));
        }
    }
}

impl Default for ScraperCache {
    fn default() -> Self {
        Self::new()
    }
}

fn cache_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ani-es-rs")
        .join("cache")
        .join("scraper.json")
}

fn load(path: &PathBuf) -> HashMap<String, CacheEntry> {
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(map) = serde_json::from_str(&data) {
            return map;
        }
    }
    HashMap::new()
}
