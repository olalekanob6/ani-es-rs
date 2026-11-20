use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use scraper::{Html, Selector};

use super::jkanime::AnimeProvider;
use crate::models::{AnimeDetail, AnimeSearchResult, VideoSource};

const DEFAULT_BASE_URL: &str = "https://animeav1.com/";

pub struct AnimeAv1Provider {
    base_url: reqwest::Url,
}

impl AnimeAv1Provider {
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_BASE_URL).expect("valid default AnimeAV1 URL")
    }

    pub fn with_base_url(base_url: &str) -> Result<Self> {
        let mut base_url = reqwest::Url::parse(base_url)?;
        if !matches!(base_url.scheme(), "http" | "https") || base_url.host_str().is_none() {
            return Err(anyhow!("La URL base de AnimeAV1 debe usar HTTP o HTTPS"));
        }
        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }
        Ok(Self { base_url })
    }

    fn resolve_anime_url(&self, value: &str) -> Result<reqwest::Url> {
        let url = self.base_url.join(value.trim())?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || url.origin() != self.base_url.origin()
            || !url.path().starts_with("/media/")
        {
            return Err(anyhow!("URL de anime fuera del proveedor AnimeAV1"));
        }
        Ok(url)
    }
}

impl Default for AnimeAv1Provider {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimeProvider for AnimeAv1Provider {
    fn name(&self) -> &str {
        "animeav1"
    }

    fn search(&self, client: &Client, query: &str) -> Result<Vec<AnimeSearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let mut url = self.base_url.join("catalogo")?;
        url.query_pairs_mut().append_pair("search", query.trim());
        let response = client.get(url).send()?.error_for_status()?;
        ensure_same_origin(&self.base_url, response.url())?;
        parse_search(&response.text()?, &self.base_url)
    }

    fn detail(&self, client: &Client, slug: &str) -> Result<AnimeDetail> {
        let url = self.resolve_anime_url(slug)?;
        let response = client.get(url.clone()).send()?.error_for_status()?;
        ensure_same_origin(&url, response.url())?;
        let body = response.text()?;
        parse_detail(&body, url.as_str())
    }

    fn video_source(&self, client: &Client, slug: &str, episode: u32) -> Result<VideoSource> {
        if episode == 0 {
            return Err(anyhow!("El número de episodio debe ser mayor que cero"));
        }
        let anime_url = self.resolve_anime_url(slug)?;
        let mut episode_url = anime_url.clone();
        episode_url
            .path_segments_mut()
            .map_err(|_| anyhow!("URL de AnimeAV1 no admite segmentos"))?
            .pop_if_empty()
            .push(&episode.to_string());
        let response = client.get(episode_url).send()?.error_for_status()?;
        ensure_same_origin(&anime_url, response.url())?;
        let page_url = response.url().clone();
        let body = response.text()?;
        parse_episode_source(&body, &page_url)?.ok_or_else(|| {
            anyhow!("AnimeAV1 no publicó un enlace de reproducción reconocible para este episodio")
        })
    }
}

fn parse_search(body: &str, base_url: &reqwest::Url) -> Result<Vec<AnimeSearchResult>> {
    let document = Html::parse_document(body);
    let article_selector = Selector::parse("main > section > div > article")
        .map_err(|error| anyhow!("Selector AnimeAV1 inválido: {error:?}"))?;
    let title_selector = Selector::parse("header > h3")
        .map_err(|error| anyhow!("Selector AnimeAV1 inválido: {error:?}"))?;
    let link_selector = Selector::parse("a[href]")
        .map_err(|error| anyhow!("Selector AnimeAV1 inválido: {error:?}"))?;

    let mut results = Vec::new();
    for article in document.select(&article_selector) {
        let Some(title) = article
            .select(&title_selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string())
            .filter(|title| !title.is_empty())
        else {
            continue;
        };
        let href = article
            .select(&link_selector)
            .filter_map(|link| link.value().attr("href"))
            .find(|href| href.starts_with("/media/"));
        let Some(href) = href else {
            continue;
        };
        let url = base_url.join(href)?;
        if url.origin() != base_url.origin() {
            continue;
        }
        let slug = url.to_string();
        if results
            .iter()
            .any(|result: &AnimeSearchResult| result.slug == slug)
        {
            continue;
        }
        results.push(AnimeSearchResult { title, slug });
    }
    Ok(results)
}

fn parse_detail(body: &str, url: &str) -> Result<AnimeDetail> {
    let document = Html::parse_document(body);
    let title_selector = Selector::parse("main h1")
        .map_err(|error| anyhow!("Selector AnimeAV1 inválido: {error:?}"))?;
    let title = document
        .select(&title_selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_string())
        .filter(|title| !title.is_empty())
        .ok_or_else(|| anyhow!("No se encontró el título en la página de AnimeAV1"))?;
    let total_episodes = extract_episode_count(body)
        .ok_or_else(|| anyhow!("No se pudo determinar el número de episodios en AnimeAV1"))?;
    if total_episodes == 0 {
        return Err(anyhow!("AnimeAV1 indicó un total de episodios inválido"));
    }
    Ok(AnimeDetail {
        title,
        slug: url.to_string(),
        total_episodes,
    })
}

fn extract_episode_count(body: &str) -> Option<u32> {
    for marker in ["episodesCount:", "\"episodesCount\":"] {
        let Some(index) = body.find(marker) else {
            continue;
        };
        let value = body[index + marker.len()..].trim_start();
        let digits: String = value.chars().take_while(|ch| ch.is_ascii_digit()).collect();
        if let Ok(count) = digits.parse::<u32>() {
            return Some(count);
        }
    }
    None
}

fn parse_episode_source(body: &str, page_url: &reqwest::Url) -> Result<Option<VideoSource>> {
    if let Some(raw_url) = extract_embed_url(body) {
        if let Ok(url) = resolve_http_url(page_url, &raw_url) {
            return Ok(Some(VideoSource {
                url: url.to_string(),
            }));
        }
    }

    let document = Html::parse_document(body);
    let iframe_selector = Selector::parse("iframe[src]")
        .map_err(|error| anyhow!("Selector AnimeAV1 inválido: {error:?}"))?;
    for iframe in document.select(&iframe_selector) {
        if let Some(src) = iframe.value().attr("src") {
            if let Ok(url) = resolve_http_url(page_url, src) {
                return Ok(Some(VideoSource {
                    url: url.to_string(),
                }));
            }
        }
    }
    Ok(None)
}

fn extract_embed_url(body: &str) -> Option<String> {
    let start = property_value_start(body, "embeds")?;
    let embeds = &body[start..];
    let end = property_key_start(embeds, "downloads").unwrap_or(embeds.len());
    let embeds = &embeds[..end];

    let mut remaining = embeds;
    while let Some(value_start) = property_value_start(remaining, "server") {
        let Some((server, consumed)) = parse_quoted_value(&remaining[value_start..]) else {
            break;
        };
        let after_server = &remaining[value_start + consumed..];
        if server.eq_ignore_ascii_case("HLS") {
            if let Some(url) = quoted_field_after(after_server, "url") {
                return Some(url);
            }
        }
        remaining = after_server;
    }

    quoted_field_after(embeds, "url")
}

fn property_key_start(text: &str, key: &str) -> Option<usize> {
    let mut offset = 0;
    while let Some(relative) = text[offset..].find(key) {
        let start = offset + relative;
        let previous = text[..start].chars().next_back();
        if previous.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
            offset = start + key.len();
            continue;
        }

        let mut cursor = start + key.len();
        if text[cursor..].starts_with('"') || text[cursor..].starts_with('\'') {
            cursor += text[cursor..].chars().next()?.len_utf8();
        }
        while text[cursor..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            cursor += text[cursor..].chars().next()?.len_utf8();
        }
        if text[cursor..].starts_with(':') {
            return Some(start);
        }
        offset = start + key.len();
    }
    None
}

fn property_value_start(text: &str, key: &str) -> Option<usize> {
    let key_start = property_key_start(text, key)?;
    let mut cursor = key_start + key.len();
    if text[cursor..].starts_with('"') || text[cursor..].starts_with('\'') {
        cursor += text[cursor..].chars().next()?.len_utf8();
    }
    while text[cursor..]
        .chars()
        .next()
        .is_some_and(char::is_whitespace)
    {
        cursor += text[cursor..].chars().next()?.len_utf8();
    }
    text[cursor..].starts_with(':').then_some(cursor + 1)
}

fn parse_quoted_value(text: &str) -> Option<(String, usize)> {
    let quote = text.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let value = &text[quote.len_utf8()..];
    let end = value.find(quote)?;
    Some((value[..end].replace("\\/", "/"), end + quote.len_utf8() * 2))
}

fn quoted_field_after(text: &str, field: &str) -> Option<String> {
    let value_start = property_value_start(text, field)?;
    parse_quoted_value(&text[value_start..]).map(|(value, _)| value)
}

fn ensure_same_origin(expected: &reqwest::Url, actual: &reqwest::Url) -> Result<()> {
    if expected.origin() != actual.origin() {
        return Err(anyhow!(
            "AnimeAV1 redirigió la petición fuera de su dominio"
        ));
    }
    Ok(())
}

fn resolve_http_url(base: &reqwest::Url, value: &str) -> Result<reqwest::Url> {
    let url = base.join(value.trim())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(anyhow!("URL de reproducción no válida"));
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_parses_media_cards_and_normalizes_slugs() {
        let html = r#"
            <main><section><div>
                <article><a href="/media/one-piece"><header><h3>One Piece</h3></header></a></article>
                <article><a href="/media/one-piece-heroines"><header><h3>One Piece: Heroines</h3></header></a></article>
            </div></section></main>
        "#;
        let base = reqwest::Url::parse("https://animeav1.com/").unwrap();
        let results = parse_search(html, &base).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "One Piece");
        assert_eq!(results[0].slug, "https://animeav1.com/media/one-piece");
    }

    #[test]
    fn detail_reads_episode_count_from_svelte_page_data() {
        let html = r#"
            <main><h1>One Piece</h1></main>
            <script>data:{media:{id:1,episodesCount:1180}}</script>
        "#;
        let detail = parse_detail(html, "https://animeav1.com/media/one-piece").unwrap();
        assert_eq!(detail.title, "One Piece");
        assert_eq!(detail.total_episodes, 1180);
    }

    #[test]
    fn episode_selects_hls_embed_url() {
        let html = r#"
            <script>data:{media:{id:1},embeds:{SUB:[{server:"HLS",url:"https://player.example/play/abc"}]},downloads:{SUB:[]}}</script>
        "#;
        let base = reqwest::Url::parse("https://animeav1.com/media/one-piece/1").unwrap();
        let source = parse_episode_source(html, &base).unwrap().unwrap();
        assert_eq!(source.url, "https://player.example/play/abc");
    }

    #[test]
    fn episode_parser_does_not_fall_through_to_download_links() {
        let html = r#"
            <script>data:{media:{},embeds:{SUB:[]}, downloads:{SUB:[{server:"DL",url:"https://download.example/file"}]}}</script>
        "#;
        let page = reqwest::Url::parse("https://animeav1.com/media/one-piece/1").unwrap();
        assert!(parse_episode_source(html, &page).unwrap().is_none());
    }

    #[test]
    fn episode_parser_handles_whitespace_before_downloads_field() {
        let html = r#"
            <script>data:{media:{},embeds:{SUB:[{server:"HLS",url:"https://player.example/play/abc"}]}, downloads:{SUB:[]}}</script>
        "#;
        let page = reqwest::Url::parse("https://animeav1.com/media/one-piece/1").unwrap();
        let source = parse_episode_source(html, &page).unwrap().unwrap();
        assert_eq!(source.url, "https://player.example/play/abc");
    }

    #[test]
    fn episode_parser_accepts_quoted_property_names() {
        let html = r##"
            <script>data:{"media":{},"embeds":{"SUB":[{"server":"HLS","url":"https://player.example/play/abc"}]},"downloads":{"SUB":[]}}</script>
        "##;
        let page = reqwest::Url::parse("https://animeav1.com/media/one-piece/1").unwrap();
        let source = parse_episode_source(html, &page).unwrap().unwrap();
        assert_eq!(source.url, "https://player.example/play/abc");
    }

    #[test]
    fn provider_response_rejects_cross_origin_redirects() {
        let expected = reqwest::Url::parse("https://animeav1.com/media/one-piece").unwrap();
        let redirected = reqwest::Url::parse("https://untrusted.example/media/one-piece").unwrap();

        assert!(ensure_same_origin(&expected, &redirected).is_err());
        assert!(ensure_same_origin(&expected, &expected).is_ok());
    }

    #[test]
    fn anime_page_resolver_rejects_urls_outside_provider_origin() {
        let provider = AnimeAv1Provider::new();
        assert!(provider
            .resolve_anime_url("https://example.com/media/one-piece")
            .is_err());
    }
}
