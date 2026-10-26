use anyhow::{anyhow, Result};
use reqwest::blocking::Client;

use crate::{AnimeDetail, AnimeSearchResult, VideoSource};

pub struct JkanimeProvider;

impl JkanimeProvider {
    pub fn new() -> Self {
        Self
    }
}

impl AnimeProvider for JkanimeProvider {
    fn name(&self) -> &str {
        "jkanime"
    }

    fn search(&self, client: &Client, query: &str) -> Result<Vec<AnimeSearchResult>> {
        // Lógica de búsqueda movida desde scraper.rs
        let url = format!("https://jkanime.net/buscar/{}/", query);
        let body = client.get(&url).send()?.text()?;
        let document = scraper::Html::parse_document(&body);

        let h5_selector = scraper::Selector::parse("h5 a").map_err(|e| anyhow!("{:?}", e))?;
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

    fn detail(&self, client: &Client, slug: &str) -> Result<AnimeDetail> {
        // Lógica de detalle movida desde scraper.rs
        let mut url = slug.to_string();
        if !url.ends_with('/') {
            url.push('/');
        }

        let body = client.get(&url).send()?.text()?;
        let document = scraper::Html::parse_document(&body);

        let csrf_token = document
            .select(&scraper::Selector::parse("meta[name=csrf-token]").unwrap())
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_string());

        let episode_api_base = document
            .select(&scraper::Selector::parse("script").unwrap())
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
                .select(&scraper::Selector::parse("script").unwrap())
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
            let json = client
                .post(api_base)
                .header("X-Requested-With", "XMLHttpRequest")
                .header("X-CSRF-TOKEN", token)
                .form(&form)
                .send()?
                .text()?;
            let v: serde_json::Value = serde_json::from_str(&json)?;
            v.get("total").and_then(|t| t.as_u64()).unwrap_or(0) as u32
        } else {
            0
        };

        let title_selector = scraper::Selector::parse(".anime_info h3").unwrap();
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

    fn video_source(&self, client: &Client, slug: &str, episode_number: u32) -> Result<VideoSource> {
        // Lógica de fuente movida desde scraper.rs
        let cap_url = format!("{}{}", slug, episode_number);
        let body = client.get(&cap_url).send()?.text()?;

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
        let page = client.get(&href).send()?.text()?;
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
        let page = client.get(&href).send()?.text()?;
        if let Some(source) = parse_jkplayer(&page) {
            return Ok(Some(source));
        }
    }
    Ok(None)
}

fn parse_mediafire(client: &Client, body: &str) -> Result<Option<VideoSource>> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

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

    let servers: Vec<std::collections::HashMap<String, serde_json::Value>> =
        serde_json::from_str(json_str)?;
    if let Some(server) = servers
        .iter()
        .find(|s| s.get("server").and_then(|v| v.as_str()) == Some("Mediafire"))
    {
        if let Some(remote) = server.get("remote").and_then(|v| v.as_str()) {
            if let Ok(decoded) = STANDARD.decode(remote) {
                let decoded_url = String::from_utf8(decoded)?;
                let page = client.get(&decoded_url).send()?.text()?;
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
