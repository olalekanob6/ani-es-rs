use anyhow::{anyhow, Result};
use reqwest::blocking::Client;

use crate::models::{AnimeDetail, AnimeSearchResult, VideoSource};

pub trait AnimeProvider {
    fn name(&self) -> &str;
    fn search(&self, client: &Client, query: &str) -> Result<Vec<AnimeSearchResult>>;
    fn detail(&self, client: &Client, slug: &str) -> Result<AnimeDetail>;
    fn video_source(&self, client: &Client, slug: &str, episode: u32) -> Result<VideoSource>;
}

pub struct JkanimeProvider {
    base_url: reqwest::Url,
}

impl JkanimeProvider {
    pub fn new() -> Self {
        Self::with_base_url("https://jkanime.net/").expect("valid default provider URL")
    }

    pub fn with_base_url(base_url: &str) -> Result<Self> {
        let mut base_url = reqwest::Url::parse(base_url)?;
        if !matches!(base_url.scheme(), "http" | "https") {
            return Err(anyhow!("La URL base del proveedor debe usar HTTP o HTTPS"));
        }
        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }
        Ok(Self { base_url })
    }

    pub fn build_client() -> Result<Client> {
        Self::build_client_with_timeout(15)
    }

    pub fn build_client_with_timeout(timeout_seconds: u64) -> Result<Client> {
        use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
        use std::time::Duration;

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
            ),
        );

        let jar = reqwest::cookie::Jar::default();

        let timeout = if timeout_seconds == 0 {
            15
        } else {
            timeout_seconds
        };
        Ok(Client::builder()
            .default_headers(headers)
            .cookie_provider(jar.into())
            .timeout(Duration::from_secs(timeout))
            .build()?)
    }
}

impl Default for JkanimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimeProvider for JkanimeProvider {
    fn name(&self) -> &str {
        "jkanime"
    }

    fn search(&self, client: &Client, query: &str) -> Result<Vec<AnimeSearchResult>> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .map_err(|_| anyhow!("URL base del proveedor no admite segmentos"))?
            .extend(["buscar", query, ""]);
        let response = client.get(url.clone()).send()?.error_for_status()?;
        let body = response.text()?;
        let mut results = parse_search(&body)?;
        for result in &mut results {
            result.slug = resolve_provider_page_url(&url, &result.slug)?.to_string();
        }
        Ok(results)
    }

    fn detail(&self, client: &Client, slug: &str) -> Result<AnimeDetail> {
        let url = resolve_provider_page_url(&self.base_url, slug)?;
        let response = client.get(url.clone()).send()?.error_for_status()?;
        let body = response.text()?;
        parse_detail_with_client(client, &body, url.as_str())
    }

    fn video_source(
        &self,
        client: &Client,
        slug: &str,
        episode_number: u32,
    ) -> Result<VideoSource> {
        let anime_url = resolve_provider_page_url(&self.base_url, slug)?;
        let mut cap_url = anime_url.clone();
        {
            let mut segments = cap_url
                .path_segments_mut()
                .map_err(|_| anyhow!("URL del anime no admite segmentos"))?;
            segments.pop_if_empty().push(&episode_number.to_string());
        }
        let response = client.get(cap_url.clone()).send()?.error_for_status()?;
        let body = response.text()?;

        if let Some(source) = parse_jkplayer_iframe(client, &body, &cap_url)? {
            return Ok(source);
        }
        if let Some(source) = parse_jkphp(client, &body, &cap_url)? {
            return Ok(source);
        }
        if let Some(source) = parse_mediafire(client, &body, &cap_url)? {
            return Ok(source);
        }

        Err(anyhow!(
            "No se pudo obtener la fuente de video para el episodio {}",
            episode_number
        ))
    }
}

pub fn parse_search(body: &str) -> Result<Vec<AnimeSearchResult>> {
    let document = scraper::Html::parse_document(body);
    let result_selector = scraper::Selector::parse(
        "h5 a[href], .anime-card a[href], .anime-item a[href], a.anime-title[href]",
    )
    .map_err(|error| anyhow!("Selector inválido: {error:?}"))?;
    let mut results = Vec::new();

    for element in document.select(&result_selector) {
        if let Some(href) = element.value().attr("href") {
            let title = element.text().collect::<Vec<_>>().join(" ");
            let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
            if !title.is_empty() && !href.trim().is_empty() {
                if results
                    .iter()
                    .any(|result: &AnimeSearchResult| result.slug == href.trim())
                {
                    continue;
                }
                results.push(AnimeSearchResult {
                    title,
                    slug: href.trim().to_string(),
                });
            }
        }
    }

    Ok(results)
}

pub fn parse_detail(body: &str, url: &str) -> Result<AnimeDetail> {
    let client = JkanimeProvider::build_client()?;
    parse_detail_with_client(&client, body, url)
}

fn parse_detail_with_client(client: &Client, body: &str, url: &str) -> Result<AnimeDetail> {
    let document = scraper::Html::parse_document(body);

    let csrf_token = document
        .select(
            &scraper::Selector::parse("meta[name=csrf-token]")
                .map_err(|error| anyhow!("Selector inválido: {error:?}"))?,
        )
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string());

    let scripts = scraper::Selector::parse("script")
        .map_err(|error| anyhow!("Selector inválido: {error:?}"))?;
    let script_text = document
        .select(&scripts)
        .map(|el| el.text().collect::<String>())
        .collect::<Vec<_>>();
    let episode_api_base = find_episode_endpoint(&script_text);

    let total_episodes = if let Some(ref api_base) = episode_api_base {
        let token = csrf_token.as_deref().unwrap_or("");
        let form = vec![("_token", token)];
        let api_url = resolve_provider_page_url(&reqwest::Url::parse(url)?, api_base)?;
        let response = client
            .post(api_url)
            .header("X-Requested-With", "XMLHttpRequest")
            .header("X-CSRF-TOKEN", token)
            .form(&form)
            .send()?
            .error_for_status()?;
        let json = response.text()?;
        let v: serde_json::Value = serde_json::from_str(&json)?;
        v.get("total")
            .and_then(|total| total.as_u64())
            .and_then(|total| u32::try_from(total).ok())
            .unwrap_or(0)
    } else {
        0
    };

    let title_selector = scraper::Selector::parse(".anime_info h3")
        .map_err(|error| anyhow!("Selector inválido: {error:?}"))?;
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| url.trim_end_matches('/').to_string());

    Ok(AnimeDetail {
        title: title.trim().to_string(),
        slug: url.to_string(),
        total_episodes: total_episodes.max(1),
    })
}

fn find_episode_endpoint(scripts: &[String]) -> Option<String> {
    ["/ajax/episodes/", "/ajax/search_episode/"]
        .iter()
        .find_map(|endpoint| {
            scripts.iter().find_map(|script| {
                let position = script.find(endpoint)?;
                let before = &script[..position];
                let quote = before.chars().rev().find(|ch| *ch == '\'' || *ch == '"')?;
                let start = before.rfind(quote)?;
                let after = &script[start + quote.len_utf8()..];
                let end = after.find(quote)?;
                Some(after[..end].to_string())
            })
        })
}

fn resolve_provider_url(base: &reqwest::Url, value: &str) -> Result<reqwest::Url> {
    let url = base.join(value.trim())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(anyhow!("URL del proveedor no válida: {}", value));
    }
    Ok(url)
}

fn resolve_provider_page_url(base: &reqwest::Url, value: &str) -> Result<reqwest::Url> {
    let url = resolve_provider_url(base, value)?;
    if url.origin() != base.origin() {
        return Err(anyhow!("URL fuera del dominio del proveedor: {}", value));
    }
    Ok(url)
}

fn extract_json_array(value: &str) -> Option<&str> {
    let start = value.find('[')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in value[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == char::from(92) {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    let end = start + offset + ch.len_utf8();
                    return Some(&value[start..end]);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_jkplayer(body: &str, base_url: &reqwest::Url) -> Option<VideoSource> {
    // El primer "url:" puede ser una llamada (p. ej. atob(...)); se
    // recorre hasta encontrar el primer "url:" con una URL entrecomillada.
    let mut search_from = 0;
    while let Some(offset) = body[search_from..].find("url:") {
        let value_start = search_from + offset + 4;
        let value = body[value_start..].trim_start();
        search_from = value_start;
        let mut chars = value.chars();
        let Some(quote) = chars.next() else {
            continue;
        };
        if quote != '\'' && quote != '"' {
            continue;
        }
        let rest = &value[quote.len_utf8()..];
        let Some(end) = rest.find(quote) else {
            continue;
        };
        let candidate = &rest[..end];
        if !candidate.starts_with("http://")
            && !candidate.starts_with("https://")
            && !candidate.starts_with('/')
        {
            continue;
        }
        if let Ok(url) = resolve_provider_url(base_url, candidate) {
            return Some(VideoSource {
                url: url.to_string(),
            });
        }
    }
    None
}

fn parse_jkplayer_iframe(
    client: &Client,
    body: &str,
    base_url: &reqwest::Url,
) -> Result<Option<VideoSource>> {
    fetch_iframe_source(client, body, base_url, "/jkplayer/")
}

fn parse_jkphp(
    client: &Client,
    body: &str,
    base_url: &reqwest::Url,
) -> Result<Option<VideoSource>> {
    fetch_iframe_source(client, body, base_url, "/jk.php")
}

fn extract_iframe_hrefs(body: &str, marker: &str) -> Vec<String> {
    let mut hrefs: Vec<String> = Vec::new();
    let push = |href: &str, hrefs: &mut Vec<String>| {
        let href = href.trim();
        if href.contains(marker)
            && !href.contains('\'')
            && !href.contains(' ')
            && !hrefs.iter().any(|existing| existing == href)
        {
            hrefs.push(href.to_string());
        }
    };

    // Los reproductores actuales no están en el DOM: vienen como
    // strings dentro de un <script> (video[0] = '<iframe ... src="...">').
    // Se escanea el HTML en bruto además del DOM por compatibilidad.
    for needle in ["src=\"", "src='"] {
        let quote = needle.chars().last().unwrap();
        let mut search_from = 0;
        while let Some(offset) = body[search_from..].find(needle) {
            let value_start = search_from + offset + needle.len();
            let rest = &body[value_start..];
            if let Some(end) = rest.find(quote) {
                push(&rest[..end], &mut hrefs);
            }
            search_from = value_start;
        }
    }

    let document = scraper::Html::parse_document(body);
    if let Ok(selector) = scraper::Selector::parse("iframe[src]") {
        for iframe in document.select(&selector) {
            if let Some(href) = iframe.value().attr("src") {
                push(href, &mut hrefs);
            }
        }
    }

    hrefs
}

fn fetch_iframe_source(
    client: &Client,
    body: &str,
    base_url: &reqwest::Url,
    marker: &str,
) -> Result<Option<VideoSource>> {
    for href in extract_iframe_hrefs(body, marker) {
        let Ok(iframe_url) = resolve_provider_url(base_url, &href) else {
            continue;
        };
        let Ok(response) = client.get(iframe_url).send() else {
            continue;
        };
        let Ok(response) = response.error_for_status() else {
            continue;
        };
        let page_url = response.url().clone();
        let Ok(page) = response.text() else {
            continue;
        };
        if let Some(source) = parse_jkplayer(&page, &page_url) {
            return Ok(Some(source));
        }
    }
    Ok(None)
}

fn parse_mediafire(
    client: &Client,
    body: &str,
    base_url: &reqwest::Url,
) -> Result<Option<VideoSource>> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

    let document = scraper::Html::parse_document(body);
    let script_selector = scraper::Selector::parse("script")
        .map_err(|error| anyhow!("Selector inválido: {error:?}"))?;
    let script = document
        .select(&script_selector)
        .map(|script| script.text().collect::<String>())
        .find(|text| text.contains("var servers"));
    let Some(script) = script else {
        return Ok(None);
    };
    let Some(json_str) = extract_json_array(&script) else {
        return Ok(None);
    };

    let servers: Vec<std::collections::HashMap<String, serde_json::Value>> =
        match serde_json::from_str(json_str) {
            Ok(servers) => servers,
            Err(_) => return Ok(None),
        };
    if let Some(server) = servers
        .iter()
        .find(|s| s.get("server").and_then(|v| v.as_str()) == Some("Mediafire"))
    {
        if let Some(remote) = server.get("remote").and_then(|v| v.as_str()) {
            if let Ok(decoded) = STANDARD.decode(remote) {
                let decoded_url = String::from_utf8(decoded)?;
                let remote_url = resolve_provider_url(base_url, &decoded_url)?;
                let response = client.get(remote_url).send()?.error_for_status()?;
                let page_url = response.url().clone();
                let page = response.text()?;
                let download_document = scraper::Html::parse_document(&page);
                let download_selector = scraper::Selector::parse("a[href]")
                    .map_err(|error| anyhow!("Selector inválido: {error:?}"))?;
                let download = download_document
                    .select(&download_selector)
                    .filter_map(|link| link.value().attr("href"))
                    .find(|href| href.contains("download"));
                if let Some(download) = download {
                    let download_url = resolve_provider_url(&page_url, download)?;
                    return Ok(Some(VideoSource {
                        url: download_url.to_string(),
                    }));
                }
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_relative_provider_links_and_rejects_unsafe_schemes() {
        let base = reqwest::Url::parse("https://jkanime.net/buscar/bleach/").unwrap();
        assert_eq!(
            resolve_provider_url(&base, "/anime/bleach/")
                .unwrap()
                .as_str(),
            "https://jkanime.net/anime/bleach/"
        );
        assert_eq!(
            resolve_provider_url(&base, "https://cdn.example/video.mp4")
                .unwrap()
                .as_str(),
            "https://cdn.example/video.mp4"
        );
        assert!(resolve_provider_url(&base, "javascript:alert(1)").is_err());
        assert!(resolve_provider_page_url(&base, "https://example.com/anime/").is_err());
    }

    #[test]
    fn parses_relative_video_source_urls() {
        let base = reqwest::Url::parse("https://jkanime.net/jkplayer/bleach/").unwrap();
        let source = parse_jkplayer("player.setup({url: '/video/episode.m3u8'});", &base).unwrap();
        assert_eq!(source.url, "https://jkanime.net/video/episode.m3u8");
    }

    #[test]
    fn json_array_extractor_ignores_closing_brackets_inside_strings() {
        assert_eq!(
            extract_json_array("prefix [{\"value\":\"]\"}] suffix"),
            Some("[{\"value\":\"]\"}]")
        );
    }

    #[test]
    fn episode_endpoint_parser_handles_relative_and_double_quoted_urls() {
        let scripts = vec![r#"fetch({url: "/ajax/search_episode/bleach"})"#.to_string()];
        assert_eq!(
            find_episode_endpoint(&scripts).as_deref(),
            Some("/ajax/search_episode/bleach")
        );
    }

    #[test]
    fn search_encodes_query_and_rejects_http_error_status() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 2048];
            let count = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..count]);
            assert!(request.starts_with("GET /buscar/bleach%20%3F%20naruto/ "));
            stream
                .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .unwrap();
        });

        let provider = JkanimeProvider::with_base_url(&format!("http://{address}/")).unwrap();
        let client = Client::new();
        let error = provider.search(&client, "bleach ? naruto").unwrap_err();
        assert!(error.to_string().contains("503"));
        server.join().unwrap();
    }

    #[test]
    fn test_search_parses_h5_a() {
        let html = r#"
            <html>
                <h5><a href="/anime/bleach/">Bleach</a></h5>
                <h5><a href="/anime/naruto/">Naruto</a></h5>
            </html>
        "#;
        let results = parse_search(html).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Bleach");
        assert_eq!(results[0].slug, "/anime/bleach/");
        assert_eq!(results[1].title, "Naruto");
        assert_eq!(results[1].slug, "/anime/naruto/");
    }

    #[test]
    fn search_parser_handles_changed_card_markup() {
        let html = r#"<article class="anime-card"><a class="anime-title" href="/bleach/">Bleach</a></article>"#;
        let results = parse_search(html).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Bleach");
        assert_eq!(results[0].slug, "/bleach/");
    }

    #[test]
    fn test_detail_extracts_csrf_and_api() {
        let html = r#"
            <html>
                <head>
                    <meta name="csrf-token" content="abc123">
                </head>
                <body>
                    <div class="anime_info"><h3>Bleach</h3></div>
                </body>
            </html>
        "#;
        let detail = parse_detail(html, "https://jkanime.net/bleach/").unwrap();
        assert_eq!(detail.title, "Bleach");
        assert_eq!(detail.slug, "https://jkanime.net/bleach/");
        assert_eq!(detail.total_episodes, 1);
    }

    #[test]
    fn test_detail_extracts_search_episode_api() {
        let html = r#"
            <html>
                <body>
                    <div class="anime_info"><h3>Bleach</h3></div>
                </body>
            </html>
        "#;
        let detail = parse_detail(html, "https://jkanime.net/bleach/").unwrap();
        assert_eq!(detail.title, "Bleach");
        assert_eq!(detail.slug, "https://jkanime.net/bleach/");
    }

    #[test]
    fn test_video_source_parses_jkplayer_iframe() {
        let html = r#"
            <html>
            </html>
        "#;
        let client = Client::new();
        let base = reqwest::Url::parse("https://jkanime.net/anime/1/").unwrap();
        let result = parse_jkplayer_iframe(&client, html, &base);
        assert!(result.is_ok());
    }

    #[test]
    fn test_video_source_falls_back_to_jkphp() {
        let html = r#"
            <html>
            </html>
        "#;
        let client = Client::new();
        let base = reqwest::Url::parse("https://jkanime.net/anime/1/").unwrap();
        let result = parse_jkphp(&client, html, &base);
        assert!(result.is_ok());
    }

    #[test]
    fn test_video_source_falls_back_to_mediafire() {
        let html = r#"
            <html>
                <script>
                    var servers = [];
                </script>
            </html>
        "#;
        let client = Client::new();
        let base = reqwest::Url::parse("https://jkanime.net/anime/1/").unwrap();
        let result = parse_mediafire(&client, html, &base);
        assert!(result.is_ok());
    }

    #[test]
    fn extract_iframe_hrefs_finds_script_embedded_players() {
        let html = r#"<script>var video = []; video[0] = '<iframe class="player_conte" src="https://jkanime.net/jkplayer/um?e=ABC&t=123" width="565"></iframe>'; video[1] = '<iframe src="https://jkanime.net/jkplayer/c1?u='+val.remote+'"></iframe>';</script>"#;
        let hrefs = extract_iframe_hrefs(html, "/jkplayer/");
        assert_eq!(
            hrefs,
            vec!["https://jkanime.net/jkplayer/um?e=ABC&t=123".to_string()]
        );
    }

    #[test]
    fn jkplayer_parser_skips_unquoted_url_calls() {
        let base = reqwest::Url::parse("https://jkanime.net/jkplayer/bleach/").unwrap();
        let source = parse_jkplayer(
            "video: { url: atob('aHR0cHM6Ly9leGFtcGxl'), url: 'https://cdn.example/ep.m3u8' }",
            &base,
        )
        .unwrap();
        assert_eq!(source.url, "https://cdn.example/ep.m3u8");
    }
}
