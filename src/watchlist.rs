use anyhow::Result;

use crate::models::{WatchlistEntry, WatchlistMap, WatchlistStatus};
use crate::storage::{app_data_dir, atomic_write, legacy_data_dir};

pub fn watchlist_dir() -> Result<std::path::PathBuf> {
    app_data_dir().map_err(Into::into)
}

pub fn watchlist_json_path() -> Result<std::path::PathBuf> {
    Ok(watchlist_dir()?.join("watchlist.json"))
}

pub fn load_watchlist() -> Result<WatchlistMap> {
    let path = watchlist_json_path()?;
    let legacy = legacy_data_dir()
        .map(|dir| dir.join("watchlist.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    load_watchlist_from_paths(&path, &legacy)
}

fn load_watchlist_from_paths(
    path: &std::path::Path,
    legacy_path: &std::path::Path,
) -> Result<WatchlistMap> {
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            match std::fs::read_to_string(legacy_path) {
                Ok(data) => data,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(WatchlistMap::new());
                }
                Err(error) => return Err(error.into()),
            }
        }
        Err(error) => return Err(error.into()),
    };
    Ok(serde_json::from_str(&data)?)
}

pub fn save_watchlist(map: &WatchlistMap) -> Result<()> {
    let path = watchlist_json_path()?;
    let json = serde_json::to_string_pretty(map)?;
    atomic_write(&path, json.as_bytes()).map_err(Into::into)
}

pub fn ensure_entry_provider(map: &WatchlistMap, title: &str, provider: &str) -> Result<()> {
    if let Some(entry) = map.get(title) {
        let saved_provider = entry.provider.as_deref().unwrap_or("jkanime");
        if saved_provider != provider {
            anyhow::bail!(
                "watchlist '{title}' está guardada con el proveedor '{saved_provider}', no '{provider}'"
            );
        }
    }
    Ok(())
}

fn ensure_add_provider_compatible(map: &WatchlistMap, title: &str, provider: &str) -> Result<()> {
    for (saved_title, entry) in map
        .iter()
        .filter(|(saved_title, _)| saved_title.eq_ignore_ascii_case(title))
    {
        let saved_provider = entry.provider.as_deref().unwrap_or("jkanime");
        if saved_provider != provider {
            anyhow::bail!(
                "watchlist '{saved_title}' está guardada con el proveedor '{saved_provider}', no '{provider}'"
            );
        }
    }
    Ok(())
}

pub fn add_entry(map: &mut WatchlistMap, title: String, slug: String) -> bool {
    add_entry_for_provider(map, title, slug, "jkanime")
}

pub fn add_entry_for_provider(
    map: &mut WatchlistMap,
    title: String,
    slug: String,
    provider: &str,
) -> bool {
    if map.contains_key(&title) {
        return false;
    }
    let now = chrono::Utc::now().to_rfc3339();
    map.insert(
        title,
        WatchlistEntry {
            slug,
            provider: Some(provider.to_string()),
            status: WatchlistStatus::default(),
            added_at: now.clone(),
            updated_at: now,
        },
    );
    true
}

pub fn remove_entry(map: &mut WatchlistMap, title: &str) -> bool {
    map.remove(title).is_some()
}

pub fn mark_status_entry(map: &mut WatchlistMap, title: &str, status: WatchlistStatus) -> bool {
    match map.get_mut(title) {
        Some(entry) => {
            entry.status = status;
            entry.updated_at = chrono::Utc::now().to_rfc3339();
            true
        }
        None => false,
    }
}

pub fn add_watchlist(map: &mut WatchlistMap, title: String, slug: String) -> Result<()> {
    add_watchlist_for_provider(map, title, slug, "jkanime")
}

pub fn add_watchlist_for_provider(
    map: &mut WatchlistMap,
    title: String,
    slug: String,
    provider: &str,
) -> Result<()> {
    ensure_entry_provider(map, &title, provider)?;
    ensure_add_provider_compatible(map, &title, provider)?;
    add_entry_for_provider(map, title, slug, provider);
    save_watchlist(map)
}

pub fn remove_watchlist(map: &mut WatchlistMap, title: &str) -> Result<()> {
    remove_entry(map, title);
    save_watchlist(map)
}

pub fn mark_status(map: &mut WatchlistMap, title: &str, status: WatchlistStatus) -> Result<()> {
    if mark_status_entry(map, title, status) {
        save_watchlist(map)
    } else {
        Err(anyhow::Error::msg("Anime no encontrado en watchlist"))
    }
}

pub fn find_entry<'a>(map: &'a WatchlistMap, title: &str) -> Option<&'a WatchlistEntry> {
    map.get(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_legacy_watchlist_json() {
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("new/watchlist.json");
        let legacy = dir.path().join("old/watchlist.json");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(
            &legacy,
            r#"{"Bleach":{"slug":"/bleach/","status":"watching","added_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}}"#,
        )
        .unwrap();

        let list = load_watchlist_from_paths(&current, &legacy).unwrap();
        assert_eq!(list.get("Bleach").unwrap().slug, "/bleach/");
        assert!(matches!(
            list.get("Bleach").unwrap().status,
            WatchlistStatus::Watching
        ));
    }

    #[test]
    fn missing_legacy_timestamp_does_not_discard_watchlist_entry() {
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("new/watchlist.json");
        let legacy = dir.path().join("old/watchlist.json");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(
            &legacy,
            r#"{"Bleach":{"slug":"/bleach/","status":"watching","added_at":"2026-01-01T00:00:00Z"}}"#,
        )
        .unwrap();

        let list = load_watchlist_from_paths(&current, &legacy).unwrap();
        assert_eq!(list.get("Bleach").unwrap().updated_at, "");
    }

    #[test]
    fn add_entry_for_provider_records_source_provider() {
        let mut map = WatchlistMap::new();
        assert!(add_entry_for_provider(
            &mut map,
            "One Piece".to_string(),
            "https://animeav1.com/media/one-piece".to_string(),
            "animeav1",
        ));
        assert_eq!(
            map.get("One Piece").unwrap().provider.as_deref(),
            Some("animeav1")
        );
    }

    #[test]
    fn watchlist_entry_cannot_be_used_as_another_provider() {
        let mut map = WatchlistMap::new();
        add_entry_for_provider(
            &mut map,
            "One Piece".to_string(),
            "https://jkanime.net/one-piece/".to_string(),
            "jkanime",
        );

        assert!(ensure_entry_provider(&map, "One Piece", "animeav1").is_err());
        assert!(ensure_entry_provider(&map, "One Piece", "jkanime").is_ok());
    }

    #[test]
    fn watchlist_add_rejects_case_insensitive_provider_conflicts() {
        let mut map = WatchlistMap::new();
        add_entry_for_provider(
            &mut map,
            "One Piece".to_string(),
            "https://jkanime.net/one-piece/".to_string(),
            "jkanime",
        );

        assert!(ensure_add_provider_compatible(&map, "one piece", "animeav1").is_err());
    }

    #[test]
    fn old_watchlist_records_remain_readable_without_provider_metadata() {
        let entry: WatchlistEntry = serde_json::from_str(
            r#"{"slug":"/bleach/","status":"watching","added_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}"#,
        )
        .unwrap();
        assert_eq!(entry.provider, None);
    }

    #[test]
    fn add_entry_inserts_once_and_ignores_duplicates() {
        let mut map = WatchlistMap::new();
        assert!(add_entry(
            &mut map,
            "Bleach".to_string(),
            "/bleach/".to_string()
        ));
        assert!(!add_entry(
            &mut map,
            "Bleach".to_string(),
            "/other/".to_string()
        ));
        let entry = map.get("Bleach").unwrap();
        assert_eq!(entry.slug, "/bleach/");
        assert!(matches!(entry.status, WatchlistStatus::Planned));
        assert!(!entry.added_at.is_empty());
    }

    #[test]
    fn remove_entry_removes_only_the_requested_title() {
        let mut map = WatchlistMap::new();
        add_entry(&mut map, "Bleach".to_string(), "/bleach/".to_string());
        add_entry(&mut map, "Naruto".to_string(), "/naruto/".to_string());
        assert!(remove_entry(&mut map, "Bleach"));
        assert!(!map.contains_key("Bleach"));
        assert!(map.contains_key("Naruto"));
        assert!(!remove_entry(&mut map, "Unknown"));
    }

    #[test]
    fn mark_status_entry_updates_status_and_reports_unknown_title() {
        let mut map = WatchlistMap::new();
        add_entry(&mut map, "Bleach".to_string(), "/bleach/".to_string());
        assert!(mark_status_entry(
            &mut map,
            "Bleach",
            WatchlistStatus::Watching
        ));
        assert!(matches!(
            map.get("Bleach").unwrap().status,
            WatchlistStatus::Watching
        ));
        assert!(!mark_status_entry(
            &mut map,
            "Unknown",
            WatchlistStatus::Completed
        ));
    }

    #[test]
    fn find_entry_returns_none_for_unknown_title() {
        let mut map = WatchlistMap::new();
        add_entry(&mut map, "Bleach".to_string(), "/bleach/".to_string());
        assert_eq!(
            find_entry(&map, "Bleach").map(|e| e.slug.as_str()),
            Some("/bleach/")
        );
        assert!(find_entry(&map, "Unknown").is_none());
    }
}
