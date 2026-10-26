use ani_es_rs::{
    color, load_history, migrate_legacy_db_if_exists, save_history, HistoryMap,
    build_client, fetch_anime_detail, fetch_video_source, search_anime,
    prompt_search, select_episode, select_from_list, select_post_episode_action,
    play_video, Config, init as init_logger, info, BOLD, CYAN, GREEN, RED, RESET, YELLOW,
};
use anyhow::Result;
use clap::Parser;
use reqwest::blocking::Client;

#[derive(Parser, Debug)]
#[command(author, version, about = "Ver anime en español desde la terminal", long_about = None)]
struct Args {
    /// Consulta de búsqueda de anime
    query: Option<String>,

    /// Continuar el último anime visto
    #[arg(short, long)]
    r#continue: bool,

    /// Episodio a reproducir (modo non-interactive)
    #[arg(long)]
    episode: Option<u32>,

    /// Desactivar interfaz interactiva fzf
    #[arg(long)]
    no_fzf: bool,

    /// Reproducir automáticamente todos los episodios
    #[arg(long)]
    auto_next: bool,
}

fn main() -> Result<()> {
    let config = Config::load();
    init_logger(&config.app.log_level);
    info("Iniciando ani-es-rs");

    migrate_legacy_db_if_exists()?;
    let mut history = load_history()?;

    let args = Args::parse();
    let client = build_client()?;

    if args.r#continue {
        let _ = run_continue(&client, &mut history);
        return Ok(());
    }

    if args.query.is_some() || args.episode.is_some() || args.no_fzf {
        let query = args.query.unwrap_or_default();
        let _ = run_search_and_play(&client, &mut history, &query, args.episode, args.no_fzf, args.auto_next);
        return Ok(());
    }

    run_main_menu(&client, &mut history)
}

fn run_main_menu(client: &Client, history: &mut HistoryMap) -> Result<()> {
    loop {
        let options = vec![
            color("Buscar anime", BOLD),
            color("Continuar ultimo", BOLD),
            color("Historial", BOLD),
            color("Salir", BOLD),
        ];

        let action = match select_from_list(&options, &color("Menu principal: ", CYAN), "~8") {
            Ok(Some(a)) => a,
            Ok(None) => {
                println!("{}", color("Hasta luego", GREEN));
                return Ok(());
            }
            Err(_) => {
                eprintln!("{}", color("Error: fzf no esta instalado. Instalalo para usar el menu interactivo.", RED));
                std::process::exit(1);
            }
        };

        match action.as_str() {
            "Buscar anime" => {
                let query = match prompt_search("") {
                    Ok(q) => q,
                    Err(_) => continue,
                };
                if query.is_empty() {
                    continue;
                }
                let _ = run_search_and_play(client, history, &query, None, false, false);
            }
            "Continuar ultimo" => {
                let _ = run_continue(client, history);
            }
            "Historial" => {
                show_history(history);
                let _ = prompt_search("Presiona enter para volver...");
            }
            "Salir" => {
                println!("{}", color("Hasta luego", GREEN));
                return Ok(());
            }
            _ => break,
        }
    }

    Ok(())
}

fn show_history(history: &HistoryMap) {
    if history.is_empty() {
        println!("{}", color("Sin historial", YELLOW));
        return;
    }

    let mut entries: Vec<_> = history.iter().collect();
    entries.sort_by(|a, b| b.1.last_cap.cmp(&a.1.last_cap));

    println!("\n{}", color("=== Historial ===", BOLD));
    for (title, entry) in entries {
        let ts = entry.updated_at.as_deref().unwrap_or("--");
        println!(
            "- {} {} cap {}{} ({})",
            color(title, CYAN),
            RESET,
            color(entry.last_cap.to_string(), YELLOW),
            RESET,
            ts
        );
    }
    println!();
}

fn run_continue(client: &Client, history: &mut HistoryMap) {
    let last = history
        .iter()
        .max_by_key(|(_, entry)| entry.last_cap)
        .map(|(k, _)| k.clone());

    if let Some(anime) = last {
        let last_cap = ani_es_rs::history::get_last_cap(history, &anime);
        let progress = ani_es_rs::history::get_progress(history, &anime);
        let mut slug = history.get(&anime).and_then(|e| e.slug.clone());

        if slug.is_none() {
            if let Ok(results) = search_anime(client, &anime) {
                if let Some(first) = results.into_iter().find(|r| r.title == anime) {
                    slug = Some(first.slug);
                }
            }
        }

        let slug = slug.as_deref().unwrap_or(&anime);
        let _ = play_anime(client, history, &anime, slug, last_cap, &progress, false);
    } else {
        println!("{}", color("No hay historial de animes recientes.", YELLOW));
    }
}

fn run_search_and_play(
    client: &Client,
    history: &mut HistoryMap,
    query: &str,
    episode: Option<u32>,
    no_fzf: bool,
    auto_next: bool,
) {
    let results = match search_anime(client, query) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", color(format!("Error buscando: {}", e), RED));
            return;
        }
    };
    if results.is_empty() {
        println!("No se encontraron resultados para: {}", query);
        return;
    }

    let selected_title = if no_fzf {
        match results.first().map(|r| r.title.clone()) {
            Some(t) => t,
            None => return,
        }
    } else {
        let items: Vec<String> = results.iter().map(|r| r.title.clone()).collect();
        match select_from_list(&items, &color("Selecciona una opción: ", CYAN), "~20") {
            Ok(Some(t)) => t,
            Ok(None) => return,
            Err(_) => {
                eprintln!("{}", color("Error: fzf no esta instalado.", RED));
                return;
            }
        }
    };

    let selected = match results.iter().find(|r| r.title == selected_title) {
        Some(s) => s,
        None => return,
    };

    let detail = match fetch_anime_detail(client, &selected.slug) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", color(format!("Error obteniendo detalle: {}", e), RED));
            return;
        }
    };
    println!("Anime: {}", color(&detail.title, BOLD));
    println!("Episodios: {}", color(detail.total_episodes.to_string(), GREEN));

    let last_cap = ani_es_rs::history::get_last_cap(history, &detail.title);
    let progress = ani_es_rs::history::get_progress(history, &detail.title);

    if episode.is_some() || no_fzf {
        let ep = episode.unwrap_or(1);
        if ep == 0 || ep > detail.total_episodes {
            println!("{}", color("Episodio inválido", RED));
            return;
        }
        let _ = play_anime(
            client,
            history,
            &detail.title,
            &detail.slug,
            ep,
            if ep == last_cap { &progress } else { "00:00:00" },
            auto_next,
        );
    } else if last_cap > 0 && last_cap <= detail.total_episodes {
        let _ = play_anime(
            client,
            history,
            &detail.title,
            &detail.slug,
            last_cap,
            &progress,
            auto_next,
        );
    } else {
        let cap = match select_episode(detail.total_episodes, &color("Episodio: ", CYAN)) {
            Ok(Some(c)) => c,
            Ok(None) => return,
            Err(_) => {
                eprintln!("{}", color("Error: fzf no esta instalado.", RED));
                return;
            }
        };
        let _ = play_anime(
            client,
            history,
            &detail.title,
            &detail.slug,
            cap,
            "00:00:00",
            auto_next,
        );
    }
}

fn play_anime(
    client: &Client,
    history: &mut HistoryMap,
    title: &str,
    slug: &str,
    episode: u32,
    progress: &str,
    auto_next: bool,
) {
    let detail = match fetch_anime_detail(client, slug) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", color(format!("Error obteniendo detalle: {}", e), RED));
            return;
        }
    };
    let source = match fetch_video_source(client, &detail.slug, episode) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", color(format!("Error obteniendo video: {}", e), RED));
            return;
        }
    };
    let episode_label = format!("{}-{}", title, episode);

    let pos = match play_video(&source.url, progress, &episode_label) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", color(format!("Error reproduciendo: {}", e), RED));
            return;
        }
    };
    ani_es_rs::history::update_history(history, title, episode, &pos, Some(detail.slug.clone()));
    let _ = save_history(history);

    if auto_next {
        for next in (episode + 1)..=detail.total_episodes {
            println!("Siguiente: {}-{}", title, next);
            let source = match fetch_video_source(client, &detail.slug, next) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}", color(format!("Error obteniendo video: {}", e), RED));
                    return;
                }
            };
            let pos = match play_video(&source.url, "00:00:00", &format!("{}-{}", title, next)) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{}", color(format!("Error reproduciendo: {}", e), RED));
                    return;
                }
            };
            ani_es_rs::history::update_history(history, title, next, &pos, Some(detail.slug.clone()));
            let _ = save_history(history);
        }
        return;
    }

    if no_fzf_flag_global() {
        return;
    }

    loop {
        let action = match select_post_episode_action() {
            Ok(Some(a)) => a,
            Ok(None) => break,
            Err(_) => {
                eprintln!("{}", color("Error: fzf no esta instalado.", RED));
                break;
            }
        };

        match action.as_str() {
            "Siguiente" => {
                let next = episode + 1;
                if next > detail.total_episodes {
                    println!("{}", color("No hay más episodios.", YELLOW));
                    break;
                }
                let source = match fetch_video_source(client, &detail.slug, next) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{}", color(format!("Error obteniendo video: {}", e), RED));
                        break;
                    }
                };
                let pos = match play_video(&source.url, "00:00:00", &format!("{}-{}", title, next)) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("{}", color(format!("Error reproduciendo: {}", e), RED));
                        break;
                    }
                };
                ani_es_rs::history::update_history(history, title, next, &pos, Some(detail.slug.clone()));
                let _ = save_history(history);
            }
            "Retroceder" => {
                if episode <= 1 {
                    println!("{}", color("No hay episodios anteriores.", YELLOW));
                    continue;
                }
                let prev = episode - 1;
                let source = match fetch_video_source(client, &detail.slug, prev) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{}", color(format!("Error obteniendo video: {}", e), RED));
                        break;
                    }
                };
                let pos = match play_video(&source.url, "00:00:00", &format!("{}-{}", title, prev)) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("{}", color(format!("Error reproduciendo: {}", e), RED));
                        break;
                    }
                };
                ani_es_rs::history::update_history(history, title, prev, &pos, Some(detail.slug.clone()));
                let _ = save_history(history);
            }
            "Atras" => {
                break;
            }
            _ => break,
        }
    }
}

fn no_fzf_flag_global() -> bool {
    use std::env;
    env::var("ANI_ES_NO_FZF").ok().map(|v| v == "1").unwrap_or(false)
}
