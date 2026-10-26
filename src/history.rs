use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{HistoryEntry, HistoryMap};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_last_cap_and_progress() {
        let mut map = HistoryMap::default();
        map.insert(
            "Bleach".to_string(),
            HistoryEntry {
                last_cap: 5,
                progress: "00:12:34".to_string(),
                slug: Some("https://jkanime.net/bleach/".to_string()),
                updated_at: Some("2026-01-01 00:00:00".to_string()),
            },
        );

        assert_eq!(get_last_cap(&map, "Bleach"), 5);
        assert_eq!(get_progress(&map, "Bleach"), "00:12:34");
        assert_eq!(get_last_cap(&map, "Unknown"), 0);
        assert_eq!(get_progress(&map, "Unknown"), "00:00:00");
    }

    #[test]
    fn test_update_history() {
        let mut map = HistoryMap::default();
        update_history(
            &mut map,
            "Naruto",
            10,
            "01:23:45",
            Some("https://jkanime.net/naruto/".to_string()),
        );
        assert_eq!(get_last_cap(&map, "Naruto"), 10);
        assert_eq!(get_progress(&map, "Naruto"), "01:23:45");
        assert_eq!(
            map.get("Naruto").unwrap().slug,
            Some("https://jkanime.net/naruto/".to_string())
        );
    }
}

pub fn history_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ani-es")
}

pub fn history_json_path() -> PathBuf {
    history_dir().join("history.json")
}

pub fn history_db_path() -> PathBuf {
    history_dir().join("history.db")
}

pub fn load_history() -> Result<HistoryMap> {
    let path = history_json_path();
    if !path.exists() {
        return Ok(HistoryMap::default());
    }
    let data = fs::read_to_string(&path)?;
    let map: HistoryMap = serde_json::from_str(&data)?;
    Ok(map)
}

pub fn save_history(map: &HistoryMap) -> Result<()> {
    let dir = history_dir();
    fs::create_dir_all(&dir)?;
    let path = history_json_path();
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_string_pretty(map)?;
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn migrate_legacy_db_if_exists() -> Result<()> {
    let db_path = history_db_path();
    if !db_path.exists() {
        return Ok(());
    }

    println!("El sistema de guardado de historial ha cambiado.");
    println!("¿Desea importar su antiguo historial? (y/n): ");
    use std::io::{self, Write};
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim().to_lowercase();

    if answer == "y" || answer == "s" {
        import_from_db(&db_path)?;
        fs::remove_file(&db_path)?;
        println!("Historial importado correctamente.");
    } else {
        fs::remove_file(&db_path)?;
        println!("El historial antiguo ha sido eliminado.");
    }

    Ok(())
}

fn import_from_db(db_path: &Path) -> Result<()> {
    let mut map = load_history().unwrap_or_default();
    let conn = rusqlite::Connection::open(db_path)?;

    let table_exists: bool = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='animes'",
        [],
        |row| row.get(0),
    )?;

    if !table_exists {
        return Ok(());
    }

    let mut stmt = conn.prepare("SELECT anime, last_cap, progress FROM animes")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<i64>>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    })?;

    for row in rows {
        let (anime, last_cap_opt, progress_opt) = row?;
        let last_cap = match last_cap_opt {
            Some(v) => v.max(0) as u32,
            None => 0,
        };
        let progress = progress_opt.unwrap_or_else(|| "00:00:00".to_string());
        map.insert(
            anime,
            HistoryEntry {
                last_cap,
                progress,
                slug: None,
                updated_at: None,
            },
        );
    }

    save_history(&map)?;
    Ok(())
}

pub fn get_last_cap(map: &HistoryMap, anime: &str) -> u32 {
    map.get(anime).map(|e| e.last_cap).unwrap_or(0)
}

pub fn get_progress(map: &HistoryMap, anime: &str) -> String {
    map.get(anime)
        .map(|e| e.progress.clone())
        .unwrap_or_else(|| "00:00:00".to_string())
}

pub fn update_history(
    map: &mut HistoryMap,
    anime: &str,
    cap: u32,
    progress: &str,
    slug: Option<String>,
) {
    map.insert(
        anime.to_string(),
        HistoryEntry {
            last_cap: cap,
            progress: progress.to_string(),
            slug,
            updated_at: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        },
    );
}
