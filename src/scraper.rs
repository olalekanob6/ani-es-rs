use anyhow::Result;
use reqwest::blocking::Client;

use crate::models::{AnimeDetail, AnimeSearchResult, VideoSource};
use crate::providers::{AnimeProvider, JkanimeProvider};

pub fn build_client() -> Result<Client> {
    JkanimeProvider::build_client()
}

pub fn search_anime(client: &Client, query: &str) -> Result<Vec<AnimeSearchResult>> {
    JkanimeProvider.search(client, query)
}

pub fn fetch_anime_detail(client: &Client, slug: &str, _cache: &mut crate::ScraperCache) -> Result<AnimeDetail> {
    JkanimeProvider.detail(client, slug)
}

pub fn fetch_video_source(client: &Client, anime_url: &str, episode_number: u32, _cache: &mut crate::ScraperCache) -> Result<VideoSource> {
    JkanimeProvider.video_source(client, anime_url, episode_number)
}
