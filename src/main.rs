use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reqwest::blocking::Client;

use ani_es_rs::{
    add_watchlist, color, fetch_anime_detail, info, init as init_logger, latest_anime,
    load_history, load_watchlist, mark_status, migrate_legacy_db_if_exists,
    models::AnimeSearchResult, play_video, prompt_search, providers::AnimeProvider,
    providers::JkanimeProvider, remove_watchlist, save_history, scraper::search_anime_cached,
    select_episode, select_from_list, select_post_episode_action, set_color_enabled, Config,
    HistoryMap, ScraperCache, WatchlistStatus, BOLD, CYAN, GREEN, RED, RESET, YELLOW,
};

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

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Gestiona tu lista de anime para ver
    Watchlist {
        #[command(subcommand)]
        action: WatchlistAction,
    },
}

#[derive(Subcommand, Debug)]
enum WatchlistAction {
    /// Añade un anime a la watchlist
    Add { title: String },
    /// Elimina un anime de la watchlist
    Remove { title: String },
    /// Muestra la watchlist
    List,
    /// Marca como "viendo"
    MarkWatching { title: String },
    /// Marca como "completado"
    MarkCompleted { title: String },
    /// Marca como "planeado"
    MarkPlanned { title: String },
}

fn main() -> Result<()> {
    let config = Config::load_checked()?;
    set_color_enabled(config.app.color);
    init_logger(&config.app.log_level);
    info("Iniciando ani-es-rs");

    migrate_legacy_db_if_exists()?;
    let mut history = load_history()?;
    let mut cache = ScraperCache::new();

    let args = Args::parse();
    let client = JkanimeProvider::build_client_with_timeout(config.app.timeout_seconds)?;
    let provider = JkanimeProvider::new();

    if let Some(Commands::Watchlist { action }) = args.command {
        let mut watchlist = load_watchlist()?;
        match action {
            WatchlistAction::Add { title } => {
                let results = search_anime_cached(&client, &title, &mut cache)?;
                let selected = results
                    .iter()
                    .find(|result| result.title.eq_ignore_ascii_case(title.trim()))
                    .or_else(|| results.first())
                    .ok_or_else(|| {
                        anyhow::anyhow!("No se encontraron resultados para '{title}'")
                    })?;
                add_watchlist(
                    &mut watchlist,
                    selected.title.clone(),
                    selected.slug.clone(),
                )?;
                println!(
                    "{}",
                    color(format!("'{}' añadido a watchlist.", selected.title), GREEN)
                );
            }
            WatchlistAction::Remove { title } => {
                remove_watchlist(&mut watchlist, &title)?;
                println!(
                    "{}",
                    color(format!("'{}' eliminado de watchlist.", title), GREEN)
                );
            }
            WatchlistAction::List => {
                if watchlist.is_empty() {
                    println!("{}", color("Watchlist vacia.", YELLOW));
                } else {
                    for (title, entry) in watchlist.iter() {
                        let status = match entry.status {
                            WatchlistStatus::Planned => "Planeado",
                            WatchlistStatus::Watching => "Viendo",
                            WatchlistStatus::Completed => "Completado",
                        };
                        println!(
                            "- {} [{}] (añadido: {})",
                            color(title, CYAN),
                            color(status, GREEN),
                            entry.added_at
                        );
                    }
                }
            }
            WatchlistAction::MarkWatching { title } => {
                mark_status(&mut watchlist, &title, WatchlistStatus::Watching)?;
                println!(
                    "{}",
                    color(format!("'{}' marcado como viendo.", title), GREEN)
                );
            }
            WatchlistAction::MarkCompleted { title } => {
                mark_status(&mut watchlist, &title, WatchlistStatus::Completed)?;
                println!(
                    "{}",
                    color(format!("'{}' marcado como completado.", title), GREEN)
                );
            }
            WatchlistAction::MarkPlanned { title } => {
                mark_status(&mut watchlist, &title, WatchlistStatus::Planned)?;
                println!(
                    "{}",
                    color(format!("'{}' marcado como planeado.", title), GREEN)
                );
            }
        }
        return Ok(());
    }

    if args.r#continue {
        run_continue(&client, &mut history, &mut cache, &provider, args.no_fzf)?;
        return Ok(());
    }

    if args.query.is_some() || args.episode.is_some() || args.no_fzf {
        let query = args.query.unwrap_or_default();
        run_search_and_play(
            &client,
            &mut history,
            &query,
            args.episode,
            args.no_fzf,
            args.auto_next,
            &mut cache,
            &provider,
        )?;
        return Ok(());
    }

    run_main_menu(&client, &mut history, &mut cache, &provider)
}

fn run_main_menu(
    client: &Client,
    history: &mut HistoryMap,
    cache: &mut ScraperCache,
    provider: &JkanimeProvider,
) -> Result<()> {
    loop {
        let options = vec![
            color("Buscar anime", BOLD),
            color("Continuar ultimo", BOLD),
            color("Historial", BOLD),
            color("Watchlist", BOLD),
            color("Salir", BOLD),
        ];

        let action = match select_from_list(&options, &color("Menu principal: ", CYAN), "~8") {
            Ok(Some(a)) => a,
            Ok(None) => {
                println!("{}", color("Hasta luego", GREEN));
                return Ok(());
            }
            Err(_) => {
                eprintln!(
                    "{}",
                    color(
                        "Error: fzf no esta instalado. Instalalo para usar el menu interactivo.",
                        RED
                    )
                );
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
                if let Err(error) = run_search_and_play(
                    client, history, &query, None, false, false, cache, provider,
                ) {
                    eprintln!("{}", color(format!("Error: {error:#}"), RED));
                }
            }
            "Continuar ultimo" => {
                if let Err(error) = run_continue(client, history, cache, provider, false) {
                    eprintln!("{}", color(format!("Error: {error:#}"), RED));
                }
            }
            "Historial" => {
                show_history(history);
                let _ = prompt_search("Presiona enter para volver...");
            }
            "Watchlist" => {
                show_watchlist_menu(client, history, cache, provider)?;
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
    entries.sort_by(|a, b| {
        b.1.updated_at
            .as_deref()
            .unwrap_or("")
            .cmp(a.1.updated_at.as_deref().unwrap_or(""))
            .then_with(|| b.1.last_cap.cmp(&a.1.last_cap))
    });

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

fn run_continue(
    client: &Client,
    history: &mut HistoryMap,
    cache: &mut ScraperCache,
    provider: &JkanimeProvider,
    no_fzf: bool,
) -> Result<()> {
    let last = latest_anime(history);

    if let Some(anime) = last {
        let last_cap = ani_es_rs::history::get_last_cap(history, &anime);
        let progress = ani_es_rs::history::get_progress(history, &anime);
        let mut slug = history.get(&anime).and_then(|e| e.slug.clone());

        if slug.is_none() {
            println!("{}", color("Buscando anime para continuar...", YELLOW));
            let results = match search_anime_cached(client, &anime, cache) {
                Ok(results) => results,
                Err(error) => {
                    return Err(error).context("No se pudo buscar el anime para continuar")
                }
            };
            if results.is_empty() {
                println!(
                    "{}",
                    color("No se encontraron resultados para continuar.", RED)
                );
                return Ok(());
            }
            let selected = if results.len() == 1 {
                results.into_iter().next().unwrap()
            } else {
                let options: Vec<String> = results
                    .iter()
                    .map(|r| format!("{} ({})", r.title, r.slug))
                    .collect();
                let selected =
                    select_from_list(&options, "Selecciona el anime para continuar:", "40%");
                if let Ok(Some(sel)) = selected {
                    if let Some(inner) = sel.strip_suffix(')') {
                        if let Some((title, slug)) = inner.rsplit_once(" (") {
                            AnimeSearchResult {
                                title: title.to_string(),
                                slug: slug.to_string(),
                            }
                        } else {
                            results.into_iter().next().unwrap()
                        }
                    } else {
                        results.into_iter().next().unwrap()
                    }
                } else {
                    return Ok(());
                }
            };
            slug = Some(selected.slug);
        }

        let slug = slug.as_deref().unwrap_or(&anime);
        play_anime(
            client, history, &anime, slug, last_cap, &progress, false, no_fzf, cache, provider,
        )?;
    } else {
        println!("{}", color("No hay historial de animes recientes.", YELLOW));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_search_and_play(
    client: &Client,
    history: &mut HistoryMap,
    query: &str,
    episode: Option<u32>,
    no_fzf: bool,
    auto_next: bool,
    cache: &mut ScraperCache,
    provider: &JkanimeProvider,
) -> Result<()> {
    let results = match search_anime_cached(client, query, cache) {
        Ok(r) => r,
        Err(e) => {
            return Err(e).context("Error buscando anime");
        }
    };
    if results.is_empty() {
        println!("No se encontraron resultados para: {}", query);
        return Ok(());
    }

    let selected_title = if no_fzf {
        match results.first().map(|r| r.title.clone()) {
            Some(t) => t,
            None => return Ok(()),
        }
    } else {
        let items: Vec<String> = results.iter().map(|r| r.title.clone()).collect();
        match select_from_list(&items, &color("Selecciona una opción: ", CYAN), "~20") {
            Ok(Some(t)) => t,
            Ok(None) => return Ok(()),
            Err(_) => {
                eprintln!("{}", color("Error: fzf no esta instalado.", RED));
                return Ok(());
            }
        }
    };

    let selected = match results.iter().find(|r| r.title == selected_title) {
        Some(s) => s,
        None => return Ok(()),
    };

    let detail = match fetch_anime_detail(client, &selected.slug, cache) {
        Ok(d) => d,
        Err(e) => {
            return Err(e).context("Error obteniendo detalle del anime");
        }
    };
    println!("Anime: {}", color(&detail.title, BOLD));
    println!(
        "Episodios: {}",
        color(detail.total_episodes.to_string(), GREEN)
    );

    let last_cap = ani_es_rs::history::get_last_cap(history, &detail.title);
    let progress = ani_es_rs::history::get_progress(history, &detail.title);

    if episode.is_some() || no_fzf {
        let ep = episode.unwrap_or(1);
        if ep == 0 || ep > detail.total_episodes {
            println!("{}", color("Episodio inválido", RED));
            return Ok(());
        }
        play_anime(
            client,
            history,
            &detail.title,
            &detail.slug,
            ep,
            if ep == last_cap {
                &progress
            } else {
                "00:00:00"
            },
            auto_next,
            no_fzf,
            cache,
            provider,
        )?;
    } else if last_cap > 0 && last_cap <= detail.total_episodes {
        play_anime(
            client,
            history,
            &detail.title,
            &detail.slug,
            last_cap,
            &progress,
            auto_next,
            no_fzf,
            cache,
            provider,
        )?;
    } else {
        let cap = match select_episode(detail.total_episodes, &color("Episodio: ", CYAN)) {
            Ok(Some(c)) => c,
            Ok(None) => return Ok(()),
            Err(_) => {
                eprintln!("{}", color("Error: fzf no esta instalado.", RED));
                return Ok(());
            }
        };
        play_anime(
            client,
            history,
            &detail.title,
            &detail.slug,
            cap,
            "00:00:00",
            auto_next,
            no_fzf,
            cache,
            provider,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn play_anime(
    client: &Client,
    history: &mut HistoryMap,
    title: &str,
    slug: &str,
    mut episode: u32,
    progress: &str,
    auto_next: bool,
    no_fzf: bool,
    cache: &mut ScraperCache,
    provider: &JkanimeProvider,
) -> Result<()> {
    let detail = fetch_anime_detail(client, slug, cache)?;
    if episode == 0 || episode > detail.total_episodes {
        anyhow::bail!(
            "Episodio {episode} fuera de rango (1-{})",
            detail.total_episodes
        );
    }
    let source = provider.video_source(client, &detail.slug, episode)?;
    let episode_label = format!("{}-{}", title, episode);

    let pos = play_video(&source.url, progress, &episode_label)?;
    ani_es_rs::history::update_history(history, title, episode, &pos, Some(detail.slug.clone()));
    save_history(history)?;

    if auto_next {
        if episode < detail.total_episodes {
            for next in (episode + 1)..=detail.total_episodes {
                println!("Siguiente: {}-{}", title, next);
                let source = match provider.video_source(client, &detail.slug, next) {
                    Ok(s) => s,
                    Err(e) => {
                        return Err(e).context("Error obteniendo el siguiente episodio");
                    }
                };
                let pos = match play_video(&source.url, "00:00:00", &format!("{}-{}", title, next))
                {
                    Ok(p) => p,
                    Err(e) => {
                        return Err(e).context("Error reproduciendo el siguiente episodio");
                    }
                };
                ani_es_rs::history::update_history(
                    history,
                    title,
                    next,
                    &pos,
                    Some(detail.slug.clone()),
                );
                save_history(history)?;
            }
        }
        return Ok(());
    }

    if no_fzf {
        return Ok(());
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
                let Some(next) = next_episode(episode, detail.total_episodes) else {
                    println!("{}", color("No hay más episodios.", YELLOW));
                    break;
                };
                let source = match provider.video_source(client, &detail.slug, next) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{}", color(format!("Error obteniendo video: {}", e), RED));
                        break;
                    }
                };
                let pos = match play_video(&source.url, "00:00:00", &format!("{}-{}", title, next))
                {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("{}", color(format!("Error reproduciendo: {}", e), RED));
                        break;
                    }
                };
                episode = next;
                ani_es_rs::history::update_history(
                    history,
                    title,
                    episode,
                    &pos,
                    Some(detail.slug.clone()),
                );
                save_history(history)?;
            }
            "Retroceder" => {
                if episode <= 1 {
                    println!("{}", color("No hay episodios anteriores.", YELLOW));
                    continue;
                }
                let prev = episode - 1;
                let source = match provider.video_source(client, &detail.slug, prev) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{}", color(format!("Error obteniendo video: {}", e), RED));
                        break;
                    }
                };
                let pos = match play_video(&source.url, "00:00:00", &format!("{}-{}", title, prev))
                {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("{}", color(format!("Error reproduciendo: {}", e), RED));
                        break;
                    }
                };
                episode = prev;
                ani_es_rs::history::update_history(
                    history,
                    title,
                    episode,
                    &pos,
                    Some(detail.slug.clone()),
                );
                save_history(history)?;
            }
            "Atras" => {
                break;
            }
            _ => break,
        }
    }
    Ok(())
}

fn next_episode(current: u32, total: u32) -> Option<u32> {
    current.checked_add(1).filter(|next| *next <= total)
}

fn show_watchlist_menu(
    client: &Client,
    history: &mut HistoryMap,
    cache: &mut ScraperCache,
    provider: &JkanimeProvider,
) -> Result<()> {
    loop {
        let mut watchlist = load_watchlist()?;
        let options = vec![
            color("Ver watchlist", BOLD),
            color("Añadir anime", BOLD),
            color("Volver", BOLD),
        ];

        let action = match select_from_list(&options, &color("Watchlist:", CYAN), "~8") {
            Ok(Some(a)) => a,
            Ok(None) => break,
            Err(_) => {
                eprintln!("{}", color("Error: fzf no esta instalado.", RED));
                break;
            }
        };

        match action.as_str() {
            "Ver watchlist" => {
                if watchlist.is_empty() {
                    println!("{}", color("Watchlist vacia.", YELLOW));
                    let _ = prompt_search("Presiona enter para volver...");
                    continue;
                }

                let mut entries: Vec<_> = watchlist.iter().collect();
                entries.sort_by_key(|(_, e)| &e.added_at);

                let titles: Vec<String> = entries
                    .iter()
                    .map(|(title, entry)| {
                        let status = match entry.status {
                            WatchlistStatus::Planned => "Planeado",
                            WatchlistStatus::Watching => "Viendo",
                            WatchlistStatus::Completed => "Completado",
                        };
                        format!("{} [{}]", title, status)
                    })
                    .collect();

                let selected = match select_from_list(&titles, "Selecciona anime:", "~10") {
                    Ok(Some(s)) => s,
                    Ok(None) => continue,
                    Err(_) => continue,
                };

                let selected_title = if let Some((title, _)) = selected.rsplit_once(" [") {
                    title
                } else {
                    selected.as_str()
                };

                let entry_options = vec![
                    color("Reproducir", BOLD),
                    color("Marcar como viendo", BOLD),
                    color("Marcar como completado", BOLD),
                    color("Marcar como planeado", BOLD),
                    color("Eliminar", BOLD),
                    color("Volver", BOLD),
                ];

                let entry_action =
                    match select_from_list(&entry_options, &color("Accion:", CYAN), "~8") {
                        Ok(Some(a)) => a,
                        Ok(None) => continue,
                        Err(_) => continue,
                    };

                match entry_action.as_str() {
                    "Reproducir" => {
                        let slug = watchlist
                            .get(selected_title)
                            .map(|e| e.slug.clone())
                            .unwrap_or_default();
                        if slug.is_empty() {
                            println!("{}", color("Slug vacio para este anime.", RED));
                            continue;
                        }
                        let last_cap = ani_es_rs::history::get_last_cap(history, selected_title);
                        let progress = ani_es_rs::history::get_progress(history, selected_title);
                        play_anime(
                            client,
                            history,
                            selected_title,
                            &slug,
                            last_cap.max(1),
                            &progress,
                            true,
                            false,
                            cache,
                            provider,
                        )?;
                    }
                    "Marcar como viendo" => {
                        mark_status(&mut watchlist, selected_title, WatchlistStatus::Watching)?;
                    }
                    "Marcar como completado" => {
                        mark_status(&mut watchlist, selected_title, WatchlistStatus::Completed)?;
                    }
                    "Marcar como planeado" => {
                        mark_status(&mut watchlist, selected_title, WatchlistStatus::Planned)?;
                    }
                    "Eliminar" => {
                        remove_watchlist(&mut watchlist, selected_title)?;
                    }
                    _ => break,
                }
            }
            "Añadir anime" => {
                let title = match prompt_search("Titulo del anime:") {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                if title.is_empty() {
                    continue;
                }
                let results = match search_anime_cached(client, &title, cache) {
                    Ok(r) => r,
                    Err(error) => {
                        eprintln!("{}", color(format!("Error buscando anime: {error:#}"), RED));
                        continue;
                    }
                };
                if results.is_empty() {
                    println!("{}", color("No se encontraron resultados.", YELLOW));
                    continue;
                }

                let selected = if results.len() == 1 {
                    results.into_iter().next().unwrap()
                } else {
                    let options: Vec<String> = results
                        .iter()
                        .map(|r| format!("{} ({})", r.title, r.slug))
                        .collect();
                    let sel = select_from_list(&options, "Selecciona:", "~10");
                    match sel {
                        Ok(Some(s)) => {
                            if let Some(inner) = s.strip_suffix(')') {
                                if let Some((t, slug)) = inner.rsplit_once(" (") {
                                    AnimeSearchResult {
                                        title: t.to_string(),
                                        slug: slug.to_string(),
                                    }
                                } else {
                                    results.into_iter().next().unwrap()
                                }
                            } else {
                                results.into_iter().next().unwrap()
                            }
                        }
                        _ => continue,
                    }
                };

                let mut wl = load_watchlist()?;
                add_watchlist(&mut wl, selected.title.clone(), selected.slug.clone())?;
                println!(
                    "{}",
                    color(format!("'{}' añadido a watchlist.", selected.title), GREEN)
                );
            }
            _ => break,
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::next_episode;

    #[test]
    fn next_episode_respects_limit_and_integer_boundary() {
        assert_eq!(next_episode(2, 3), Some(3));
        assert_eq!(next_episode(3, 3), None);
        assert_eq!(next_episode(u32::MAX, u32::MAX), None);
    }
}
