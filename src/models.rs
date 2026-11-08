use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeSearchResult {
    pub title: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeDetail {
    pub title: String,
    pub slug: String,
    pub total_episodes: u32,
}

#[derive(Debug, Clone)]
pub struct VideoSource {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryEntry {
    pub last_cap: u32,
    #[serde(default = "default_progress")]
    pub progress: String,
    pub slug: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

fn default_progress() -> String {
    "00:00:00".to_string()
}

pub type HistoryMap = std::collections::HashMap<String, HistoryEntry>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WatchlistStatus {
    #[default]
    Planned,
    Watching,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistEntry {
    pub slug: String,
    #[serde(default)]
    pub status: WatchlistStatus,
    #[serde(default)]
    pub added_at: String,
    #[serde(default)]
    pub updated_at: String,
}

pub type WatchlistMap = std::collections::HashMap<String, WatchlistEntry>;
