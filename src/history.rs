use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{HistoryEntry, HistoryMap};
use crate::storage::{app_data_dir, atomic_write, legacy_data_dir};

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
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

    #[test]
    fn latest_anime_uses_activity_timestamp_before_episode_number() {
        let mut map = HistoryMap::default();
        map.insert(
            "Old series".to_string(),
            HistoryEntry {
                last_cap: 200,
                updated_at: Some("2026-01-01 10:00:00".to_string()),
                ..HistoryEntry::default()
            },
        );
        map.insert(
            "Recent series".to_string(),
            HistoryEntry {
                last_cap: 3,
                updated_at: Some("2026-02-01 10:00:00".to_string()),
                ..HistoryEntry::default()
            },
        );

        assert_eq!(latest_anime(&map).as_deref(), Some("Recent series"));
    }

    #[test]
    fn reads_legacy_history_json_when_new_path_does_not_exist() {
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("new/history.json");
        let legacy = dir.path().join("old/history.json");
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(
            &legacy,
            r#"{"Bleach":{"last_cap":8,"progress":"00:12:00","slug":"/bleach/"}}"#,
        )
        .unwrap();

        let history = load_history_from_paths(&current, &legacy).unwrap();
        assert_eq!(get_last_cap(&history, "Bleach"), 8);
        assert_eq!(get_progress(&history, "Bleach"), "00:12:00");
    }

    #[test]
    fn missing_legacy_progress_uses_zero_position() {
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("new/history.json");
        let legacy = dir.path().join("old/history.json");
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(&legacy, r#"{"Bleach":{"last_cap":8}}"#).unwrap();

        let history = load_history_from_paths(&current, &legacy).unwrap();
        assert_eq!(get_progress(&history, "Bleach"), "00:00:00");
    }

    #[test]
    fn legacy_database_merge_preserves_json_entries_and_imports_missing_titles() {
        let mut map = HistoryMap::default();
        map.insert(
            "Existing".to_string(),
            HistoryEntry {
                last_cap: 12,
                progress: "00:05:00".to_string(),
                ..HistoryEntry::default()
            },
        );

        merge_legacy_history(
            &mut map,
            vec![
                (
                    "Existing".to_string(),
                    Some(4),
                    Some("00:01:00".to_string()),
                ),
                ("Imported".to_string(), Some(7), None),
            ],
        );

        assert_eq!(get_last_cap(&map, "Existing"), 12);
        assert_eq!(get_progress(&map, "Existing"), "00:05:00");
        assert_eq!(get_last_cap(&map, "Imported"), 7);
        assert_eq!(get_progress(&map, "Imported"), "00:00:00");
    }
}

pub fn history_dir() -> PathBuf {
    app_data_dir().unwrap_or_else(|_| PathBuf::from(".").join("ani-es-rs"))
}

pub fn history_json_path() -> PathBuf {
    history_dir().join("history.json")
}

pub fn history_db_path() -> PathBuf {
    legacy_data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("history.db")
}

pub fn load_history() -> Result<HistoryMap> {
    let path = history_json_path();
    let legacy = legacy_data_dir()
        .map(|dir| dir.join("history.json"))
        .unwrap_or_else(|| PathBuf::from("."));
    load_history_from_paths(&path, &legacy)
}

fn load_history_from_paths(path: &Path, legacy_path: &Path) -> Result<HistoryMap> {
    let Some(data) = read_preferred_file(path, legacy_path)? else {
        return Ok(HistoryMap::default());
    };
    Ok(serde_json::from_str(&data)?)
}

fn read_preferred_file(path: &Path, legacy_path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(data) => Ok(Some(data)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            match fs::read_to_string(legacy_path) {
                Ok(data) => Ok(Some(data)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(error.into()),
            }
        }
        Err(error) => Err(error.into()),
    }
}

pub fn save_history(map: &HistoryMap) -> Result<()> {
    let path = history_json_path();
    let data = serde_json::to_string_pretty(map)?;
    atomic_write(&path, data.as_bytes()).map_err(Into::into)
}

pub fn migrate_legacy_db_if_exists() -> Result<()> {
    let db_path = history_db_path();
    if !path_exists(&db_path)? {
        return Ok(());
    }

    let declined_path = db_path.with_extension("migration-declined");
    if !atty::is(atty::Stream::Stdin) {
        if path_exists(&declined_path)? {
            crate::logger::info("Se omite la migración SQLite que se rechazó previamente.");
            return Ok(());
        }
        import_from_db(&db_path)?;
        crate::logger::info("Historial SQLite importado de forma no destructiva.");
        return Ok(());
    }

    println!("Se encontró una base de datos antigua de historial.");
    println!("¿Desea importar su antiguo historial? (y/n): ");
    use std::io::{self, Write};
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim().to_lowercase();

    if answer == "y" || answer == "s" {
        import_from_db(&db_path)?;
        if path_exists(&declined_path)? {
            fs::remove_file(&declined_path)?;
        }
        println!("Historial importado correctamente; la base antigua se conserva como copia.");
    } else {
        atomic_write(&declined_path, b"declined\n")?;
        println!("El historial antiguo se conserva sin cambios.");
    }

    Ok(())
}

fn path_exists(path: &Path) -> Result<bool> {
    match fs::metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn import_from_db(db_path: &Path) -> Result<()> {
    let mut map = load_history()?;
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

    let rows = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    if merge_legacy_history(&mut map, rows) {
        save_history(&map)?;
    }
    Ok(())
}

fn merge_legacy_history(
    map: &mut HistoryMap,
    rows: impl IntoIterator<Item = (String, Option<i64>, Option<String>)>,
) -> bool {
    let mut changed = false;
    for (anime, last_cap, progress) in rows {
        if map.contains_key(&anime) {
            continue;
        }
        map.insert(
            anime,
            HistoryEntry {
                last_cap: last_cap
                    .map(|value| u32::try_from(value.max(0)).unwrap_or(u32::MAX))
                    .unwrap_or(0),
                progress: progress.unwrap_or_else(|| "00:00:00".to_string()),
                slug: None,
                updated_at: None,
            },
        );
        changed = true;
    }
    changed
}

pub fn get_last_cap(map: &HistoryMap, anime: &str) -> u32 {
    map.get(anime).map(|e| e.last_cap).unwrap_or(0)
}

pub fn latest_anime(map: &HistoryMap) -> Option<String> {
    map.iter()
        .max_by(|(_, a), (_, b)| {
            a.updated_at
                .as_deref()
                .unwrap_or("")
                .cmp(b.updated_at.as_deref().unwrap_or(""))
                .then_with(|| a.last_cap.cmp(&b.last_cap))
        })
        .map(|(title, _)| title.clone())
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
