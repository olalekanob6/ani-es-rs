use anyhow::{anyhow, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reqwest::blocking::Client;
use reqwest::cookie::Jar;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use crate::models::{AnimeDetail, AnimeSearchResult, VideoSource};

const BASE_URL: &str = "https://jkanime.net";
const EXCEPTIONS_URL: &str = "https://zhuchii.github.io/ani-es/excepciones.json";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_RETRIES: u32 = 3;

pub fn build_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
        ),
    );

    let jar = Jar::default();

    Ok(Client::builder()
        .default_headers(headers)
        .cookie_provider(jar.into())
        .timeout(REQUEST_TIMEOUT)
        .build()?)
}

fn get_text(client: &Client, url: &str) -> Result<String> {
    let mut last_err = None;
    for attempt in 0..MAX_RETRIES {
        match client.get(url).send().and_then(|r| r.text()) {
            Ok(text) => return Ok(text),
            Err(e) => {
                last_err = Some(e);
                if attempt + 1 < MAX_RETRIES {
                    let sleep = Duration::from_secs(2u64.pow(attempt as u32));
                    std::thread::sleep(sleep);
                }
            }
        }
    }
    Err(last_err.unwrap().into())
}

fn post_form(client: &Client, url: &str, form: &[(&str, &str)]) -> Result<String> {
    let url = url.to_string();
    let mut last_err = None;
    for attempt in 0..MAX_RETRIES {
        match client
            .post(&url)
            .header("X-Requested-With", "XMLHttpRequest")
            .header("X-CSRF-TOKEN", "")
            .form(&form.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>())
            .send()
            .and_then(|r| r.text())
        {
            Ok(text) => return Ok(text),
            Err(e) => {
                last_err = Some(e);
                if attempt + 1 < MAX_RETRIES {
                    let sleep = Duration::from_secs(2u64.pow(attempt as u32));
                    std::thread::sleep(sleep);
                }
            }
        }
    }
    Err(last_err.unwrap().into())
}

pub fn search_anime(client: &Client, query: &str) -> Result<Vec<AnimeSearchResult>> {
    let url = format!("{}/buscar/{}/", BASE_URL, query);
    let body = get_text(client, &url)?;
    let document = Html::parse_document(&body);

    let h5_selector = Selector::parse("h5 a").map_err(|e| anyhow!("{:?}", e))?;
    let mut results = Vec::new();

    for element in document.select(&h5_selector) {
        if let Some(href) = element.value().attr("href") {
            let title = element.text().collect::<String>();
            if !title.is_empty() {
                results.push(AnimeSearchResult {
                    title: title.trim().to_string(),
                    slug: href.trim().to_string(),
                });
            }
        }
    }

    Ok(results)
}

pub fn fetch_anime_detail(client: &Client, slug: &str) -> Result<AnimeDetail> {
    let mut url = slug.to_string();
    if !url.ends_with('/') {
        url.push('/');
    }

    if let Ok(exceptions) = fetch_exceptions(client) {
        if let Some(corrected) = exceptions.get(&url) {
            url = corrected.to_string();
        }
    }

    let body = get_text(client, &url)?;
    let document = Html::parse_document(&body);

    let csrf_token = document
        .select(&Selector::parse("meta[name=csrf-token]").unwrap())
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string());

    let episode_api_base = document
        .select(&Selector::parse("script").unwrap())
        .find_map(|el| {
            let text = el.text().collect::<String>();
            let marker = "url: 'https://jkanime.net/ajax/episodes/";
            if let Some(pos) = text.find(marker) {
                let rest = &text[pos + marker.len()..];
                if let Some(end) = rest.find('\'') {
                    let candidate = "https://jkanime.net/ajax/episodes/".to_string() + &rest[..end];
                    return Some(candidate);
                }
            }
            None
        });
    let episode_api_base = episode_api_base.or_else(|| {
        document
            .select(&Selector::parse("script").unwrap())
            .find_map(|el| {
                let text = el.text().collect::<String>();
                let marker = "url: 'https://jkanime.net/ajax/search_episode/";
                if let Some(pos) = text.find(marker) {
                    let rest = &text[pos + marker.len()..];
                    if let Some(end) = rest.find('\'') {
                        let candidate =
                            "https://jkanime.net/ajax/search_episode/".to_string() + &rest[..end];
                        return Some(candidate);
                    }
                }
                None
            })
    });

    let total_episodes = if let Some(ref api_base) = episode_api_base {
        let token = csrf_token.as_deref().unwrap_or("");
        let form = vec![("_token", token)];
        let json = post_form(client, api_base, &form)?;
        let v: Value = serde_json::from_str(&json)?;
        v.get("total").and_then(|t| t.as_u64()).unwrap_or(0) as u32
    } else {
        0
    };

    let title_selector = Selector::parse(".anime_info h3").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| url.trim_end_matches('/').to_string());

    Ok(AnimeDetail {
        title: title.trim().to_string(),
        slug: url,
        total_episodes: total_episodes.max(1),
    })
}

pub fn fetch_video_source(
    client: &Client,
    anime_url: &str,
    episode_number: u32,
) -> Result<VideoSource> {
    let cap_url = format!("{}{}", anime_url, episode_number);
    let body = get_text(client, &cap_url)?;

    if let Some(source) = parse_jkplayer_iframe(client, &body)? {
        return Ok(source);
    }

    if let Some(source) = parse_jkphp(client, &body)? {
        return Ok(source);
    }

    if let Some(source) = parse_mediafire(client, &body)? {
        return Ok(source);
    }

    Err(anyhow!(
        "No se pudo obtener la fuente de video para el episodio {}",
        episode_number
    ))
}

fn parse_jkplayer(body: &str) -> Option<VideoSource> {
    let mut url = None;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("url:") {
            if let Some((_, val)) = trimmed.split_once("url: '") {
                if let Some(end) = val.find('\'') {
                    url = Some(val[..end].to_string());
                }
            }
        }
    }

    url.map(|u| VideoSource { url: u })
}

fn parse_jkplayer_iframe(client: &Client, body: &str) -> Result<Option<VideoSource>> {
    let href = body.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.contains("<iframe") && trimmed.contains("/jkplayer/") {
            if let Some(start) = trimmed.find("src=\"") {
                let rest = &trimmed[start + 5..];
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].to_string());
                }
            }
        }
        None
    });

    if let Some(href) = href {
        let page = get_text(client, &href)?;
        if let Some(source) = parse_jkplayer(&page) {
            return Ok(Some(source));
        }
    }

    Ok(None)
}

fn parse_jkphp(client: &Client, body: &str) -> Result<Option<VideoSource>> {
    let href = body.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.contains("<iframe") && trimmed.contains("/jk.php") {
            if let Some(start) = trimmed.find("src=\"") {
                let rest = &trimmed[start + 5..];
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].to_string());
                }
            }
        }
        None
    });

    if let Some(href) = href {
        let page = get_text(client, &href)?;
        if let Some(source) = parse_jkplayer(&page) {
            return Ok(Some(source));
        }
    }

    Ok(None)
}

fn parse_mediafire(client: &Client, body: &str) -> Result<Option<VideoSource>> {
    let line = body
        .lines()
        .find(|l| l.trim().starts_with("var servers ="))
        .ok_or_else(|| anyhow!("Array servers no encontrada"))?;
    let json_start = line
        .find('[')
        .ok_or_else(|| anyhow!("Array servers no encontrada"))?;
    let json_str = &line[json_start..];
    let end = json_str.find(';').unwrap_or(json_str.len());
    let json_str = &json_str[..end];

    let servers: Vec<HashMap<String, Value>> = serde_json::from_str(json_str)?;
    if let Some(server) = servers
        .iter()
        .find(|s| s.get("server").and_then(|v| v.as_str()) == Some("Mediafire"))
    {
        if let Some(remote) = server.get("remote").and_then(|v| v.as_str()) {
            if let Ok(decoded) = STANDARD.decode(remote) {
                let decoded_url = String::from_utf8(decoded)?;
                let page = get_text(client, &decoded_url)?;
                let download = page.lines().find_map(|l| {
                    let trimmed = l.trim();
                    if trimmed.starts_with("href=\"") && trimmed.contains("https://download") {
                        let start = trimmed.find("href=\"").unwrap() + 6;
                        let rest = &trimmed[start..];
                        if let Some(end) = rest.find('"') {
                            return Some(rest[..end].to_string());
                        }
                    }
                    None
                });
                if let Some(download) = download {
                    return Ok(Some(VideoSource { url: download }));
                }
            }
        }
    }

    Ok(None)
}

fn fetch_exceptions(client: &Client) -> Result<HashMap<String, String>> {
    let body = get_text(client, EXCEPTIONS_URL)?;
    let map: HashMap<String, String> = serde_json::from_str(&body)?;
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_jkplayer_basic() {
        let html = r#"video: {
    url: 'https://example.com/video.m3u8',
    type: 'customHls'
}"#;
        let result = parse_jkplayer(html);
        assert!(result.is_some());
        let source = result.unwrap();
        assert_eq!(source.url, "https://example.com/video.m3u8");
    }

    #[test]
    fn test_parse_jkplayer_no_url() {
        let html = "some random text without video info";
        let result = parse_jkplayer(html);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_jkplayer_only_url() {
        let html = r#"url: 'https://example.com/video.mp4'"#;
        let result = parse_jkplayer(html);
        assert!(result.is_some());
        let source = result.unwrap();
        assert_eq!(source.url, "https://example.com/video.mp4");
    }
}

