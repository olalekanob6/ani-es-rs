use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AnimeSearchResult {
    pub title: String,
    pub slug: String,
}

#[derive(Debug, Clone)]
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
    pub progress: String,
    pub slug: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

pub type HistoryMap = std::collections::HashMap<String, HistoryEntry>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WatchlistStatus {
    Planned,
    Watching,
    Completed,
}

impl Default for WatchlistStatus {
    fn default() -> Self {
        WatchlistStatus::Planned
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistEntry {
    pub slug: String,
    pub status: WatchlistStatus,
    pub added_at: String,
    pub updated_at: String,
}

pub type WatchlistMap = std::collections::HashMap<String, WatchlistEntry>;
