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

pub fn add_watchlist(map: &mut WatchlistMap, title: String, slug: String) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    map.entry(title.clone()).or_insert(WatchlistEntry {
        slug,
        status: WatchlistStatus::default(),
        added_at: now.clone(),
        updated_at: now,
    });
    save_watchlist(map)
}

pub fn remove_watchlist(map: &mut WatchlistMap, title: &str) -> Result<()> {
    map.remove(title);
    save_watchlist(map)
}

pub fn mark_status(map: &mut WatchlistMap, title: &str, status: WatchlistStatus) -> Result<()> {
    if let Some(entry) = map.get_mut(title) {
        entry.status = status;
        entry.updated_at = chrono::Utc::now().to_rfc3339();
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
}
