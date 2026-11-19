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

pub fn add_entry(map: &mut WatchlistMap, title: String, slug: String) -> bool {
    if map.contains_key(&title) {
        return false;
    }
    let now = chrono::Utc::now().to_rfc3339();
    map.insert(
        title,
        WatchlistEntry {
            slug,
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
    add_entry(map, title, slug);
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
