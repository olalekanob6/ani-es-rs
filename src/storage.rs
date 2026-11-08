use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

pub fn app_data_dir() -> io::Result<PathBuf> {
    dirs::data_dir()
        .map(|path| path.join("ani-es-rs"))
        .or_else(|| dirs::home_dir().map(|path| path.join(".ani-es-rs")))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "No se encontró directorio de datos",
            )
        })
}

pub fn legacy_data_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|path| path.join("ani-es"))
}

pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Ruta sin directorio padre"))?;
    fs::create_dir_all(parent)?;

    let temp_path = parent.join(format!(
        ".{}.{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("state"),
        std::process::id(),
        NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut temp_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp_path)?;
    if let Err(error) = temp_file.write_all(data).and_then(|_| temp_file.sync_all()) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }
    drop(temp_file);

    #[cfg(windows)]
    {
        if path.exists() {
            let backup_path = temp_path.with_extension("bak");
            fs::rename(path, &backup_path)?;
            if let Err(error) = fs::rename(&temp_path, path) {
                let _ = fs::rename(&backup_path, path);
                let _ = fs::remove_file(&temp_path);
                return Err(error);
            }
            let _ = fs::remove_file(backup_path);
            return Ok(());
        }
    }

    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_file_without_leaving_temporary_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");

        atomic_write(&path, b"first").unwrap();
        atomic_write(&path, b"second").unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), b"second");
        assert!(std::fs::read_dir(dir.path()).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")));
    }
}
