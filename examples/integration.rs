use ani_es_rs::{
    build_client, fetch_anime_detail, fetch_video_source, search_anime,
};

fn main() {
    let client = build_client().expect("client");

    let query = "naruto";
    println!("Busqueda: {}", query);
    let results = search_anime(&client, query).expect("search");
    println!("Resultados: {}", results.len());
    assert!(!results.is_empty(), "sin resultados");

    let slug = results.first().map(|r| r.slug.as_str()).expect("slug");
    println!("Slug: {}", slug);

    let detail = fetch_anime_detail(&client, slug).expect("detail");
    println!("Titulo: {}", detail.title);
    println!("Episodios: {}", detail.total_episodes);
    assert!(detail.total_episodes > 0, "episodios invalidos");
    assert!(!detail.slug.is_empty(), "slug vacio");

    let source = fetch_video_source(&client, &detail.slug, 1).expect("video source");
    println!("Fuente: {}", source.url);
    assert!(source.url.starts_with("http"), "url invalida");
    println!("OK");
}
