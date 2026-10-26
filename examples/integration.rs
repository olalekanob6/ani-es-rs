use ani_es_rs::{build_client, fetch_anime_detail, fetch_video_source, search_anime, ScraperCache};

#[test]
fn test_search_and_detail_fixtures() {
    let client = build_client().expect("client");
    let mut cache = ScraperCache::new();

    let results = search_anime(&client, "naruto").expect("search");
    assert!(!results.is_empty(), "sin resultados");

    let slug = results.first().map(|r| r.slug.as_str()).expect("slug");
    let detail = fetch_anime_detail(&client, slug, &mut cache).expect("detail");
    assert!(detail.total_episodes > 0, "episodios invalidos");
    assert!(!detail.slug.is_empty(), "slug vacio");

    let source = fetch_video_source(&client, &detail.slug, 1, &mut cache).expect("video source");
    assert!(source.url.starts_with("http"), "url invalida");
}
