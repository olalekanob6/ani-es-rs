use anyhow::Result;
use reqwest::blocking::Client;

use crate::models::{AnimeDetail, AnimeSearchResult, VideoSource};
use crate::providers::{AnimeProvider, JkanimeProvider};

pub fn build_client() -> Result<Client> {
    JkanimeProvider::build_client()
}

pub fn search_anime(client: &Client, query: &str) -> Result<Vec<AnimeSearchResult>> {
    JkanimeProvider::new().search(client, query)
}

pub fn search_anime_cached(
    client: &Client,
    query: &str,
    cache: &mut crate::ScraperCache,
) -> Result<Vec<AnimeSearchResult>> {
    let key = format!("search:{query}");
    if let Some(value) = cache.get(&key) {
        if let Ok(results) = serde_json::from_str(value) {
            return Ok(results);
        }
        cache.invalidate(&key);
    }

    let results = search_anime(client, query)?;
    if let Ok(value) = serde_json::to_string(&results) {
        cache.set(key, value, 300);
    }
    Ok(results)
}

pub fn fetch_anime_detail(
    client: &Client,
    slug: &str,
    cache: &mut crate::ScraperCache,
) -> Result<AnimeDetail> {
    let key = format!("detail:{slug}");
    if let Some(value) = cache.get(&key) {
        if let Ok(detail) = serde_json::from_str(value) {
            return Ok(detail);
        }
        cache.invalidate(&key);
    }

    let detail = JkanimeProvider::new().detail(client, slug)?;
    if let Ok(value) = serde_json::to_string(&detail) {
        cache.set(key, value, 900);
    }
    Ok(detail)
}

pub fn fetch_video_source(
    client: &Client,
    anime_url: &str,
    episode_number: u32,
    _cache: &mut crate::ScraperCache,
) -> Result<VideoSource> {
    JkanimeProvider::new().video_source(client, anime_url, episode_number)
}
