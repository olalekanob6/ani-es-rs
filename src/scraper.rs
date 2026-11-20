use anyhow::Result;
use reqwest::blocking::Client;

use crate::models::{AnimeDetail, AnimeSearchResult, VideoSource};
use crate::providers::{AnimeProvider, JkanimeProvider};

pub fn build_client() -> Result<Client> {
    JkanimeProvider::build_client()
}

pub fn search_anime(client: &Client, query: &str) -> Result<Vec<AnimeSearchResult>> {
    search_anime_with_provider(client, &JkanimeProvider::new(), query)
}

pub fn search_anime_with_provider(
    client: &Client,
    provider: &dyn AnimeProvider,
    query: &str,
) -> Result<Vec<AnimeSearchResult>> {
    provider.search(client, query)
}

pub fn search_anime_cached(
    client: &Client,
    query: &str,
    cache: &mut crate::ScraperCache,
) -> Result<Vec<AnimeSearchResult>> {
    search_anime_cached_with_provider(client, &JkanimeProvider::new(), query, cache)
}

pub fn search_anime_cached_with_provider(
    client: &Client,
    provider: &dyn AnimeProvider,
    query: &str,
    cache: &mut crate::ScraperCache,
) -> Result<Vec<AnimeSearchResult>> {
    let key = format!("search:{}:{}", provider.name(), query.trim().to_lowercase());
    if let Some(value) = cache.get(&key) {
        if let Ok(results) = serde_json::from_str(value) {
            return Ok(results);
        }
        cache.invalidate(&key);
    }

    let results = provider.search(client, query)?;
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
    fetch_anime_detail_with_provider(client, &JkanimeProvider::new(), slug, cache)
}

pub fn fetch_anime_detail_with_provider(
    client: &Client,
    provider: &dyn AnimeProvider,
    slug: &str,
    cache: &mut crate::ScraperCache,
) -> Result<AnimeDetail> {
    let key = format!("detail:{}:{slug}", provider.name());
    if let Some(value) = cache.get(&key) {
        if let Ok(detail) = serde_json::from_str(value) {
            return Ok(detail);
        }
        cache.invalidate(&key);
    }

    let detail = provider.detail(client, slug)?;
    // No cachear el fallback/error de conteo desconocido (actualmente 1).
    if detail.total_episodes > 1 {
        if let Ok(value) = serde_json::to_string(&detail) {
            cache.set(key, value, 900);
        }
    }
    Ok(detail)
}

pub fn fetch_video_source(
    client: &Client,
    anime_url: &str,
    episode_number: u32,
    _cache: &mut crate::ScraperCache,
) -> Result<VideoSource> {
    fetch_video_source_with_provider(client, &JkanimeProvider::new(), anime_url, episode_number)
}

pub fn fetch_video_source_with_provider(
    client: &Client,
    provider: &dyn AnimeProvider,
    anime_url: &str,
    episode_number: u32,
) -> Result<VideoSource> {
    // Intencionadamente sin caché: las URLs de reproducción pueden expirar.
    provider.video_source(client, anime_url, episode_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubProvider;

    impl AnimeProvider for StubProvider {
        fn name(&self) -> &str {
            "stub"
        }

        fn search(&self, _client: &Client, _query: &str) -> Result<Vec<AnimeSearchResult>> {
            Ok(vec![AnimeSearchResult {
                title: "AnimeAV1 result".into(),
                slug: "https://animeav1.com/media/example".into(),
            }])
        }

        fn detail(&self, _client: &Client, slug: &str) -> Result<AnimeDetail> {
            Ok(AnimeDetail {
                title: "Stub".into(),
                slug: slug.into(),
                total_episodes: 1,
            })
        }

        fn video_source(&self, _client: &Client, slug: &str, episode: u32) -> Result<VideoSource> {
            Ok(VideoSource {
                url: format!("{slug}/{episode}"),
            })
        }
    }

    #[test]
    fn search_with_provider_dispatches_to_selected_provider() {
        let results = search_anime_with_provider(&Client::new(), &StubProvider, "query").unwrap();
        assert_eq!(results[0].title, "AnimeAV1 result");
    }

    #[test]
    fn video_source_with_provider_dispatches_to_selected_provider() {
        let source = fetch_video_source_with_provider(
            &Client::new(),
            &StubProvider,
            "https://animeav1.com/media/example",
            4,
        )
        .unwrap();
        assert_eq!(source.url, "https://animeav1.com/media/example/4");
    }
}
