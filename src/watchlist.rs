use anyhow::Result;

use crate::models::{WatchlistEntry, WatchlistMap, WatchlistStatus};

pub fn watchlist_dir() -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::Error::msg("No se pudo obtener el home directory"))?;
    let dir = home.join("ani-es");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn watchlist_json_path() -> Result<std::path::PathBuf> {
    Ok(watchlist_dir()?.join("watchlist.json"))
}

pub fn load_watchlist() -> Result<WatchlistMap> {
    let path = watchlist_json_path()?;
    if !path.exists() {
        return Ok(WatchlistMap::new());
    }
    let data = std::fs::read_to_string(&path)?;
    let map: std::collections::HashMap<String, WatchlistEntry> = serde_json::from_str(&data)?;
    Ok(map)
}

pub fn save_watchlist(map: &WatchlistMap) -> Result<()> {
    let path = watchlist_json_path()?;
    let json = serde_json::to_string_pretty(map)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn add_watchlist(map: &mut WatchlistMap, title: String, slug: String) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    map.entry(title.clone())
        .or_insert(WatchlistEntry {
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
