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
