use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => Level::Debug,
            "info" => Level::Info,
            "warn" | "warning" => Level::Warn,
            "error" => Level::Error,
            _ => Level::Info,
        }
    }
}

struct LoggerState {
    level: Level,
    file_path: PathBuf,
}

impl LoggerState {
    fn log(&self, level: Level, msg: &str) {
        if level < self.level {
            return;
        }

        let line = format_log_line(level, msg);

        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
        {
            Ok(mut file) => {
                if let Err(error) = file.write_all(line.as_bytes()) {
                    eprintln!("No se pudo escribir el log: {error}");
                }
            }
            Err(error) => eprintln!(
                "No se pudo abrir el archivo de log {}: {error}",
                self.file_path.display()
            ),
        }
    }
}

lazy_static::lazy_static! {
    static ref STATE: Arc<Mutex<Option<LoggerState>>> = Arc::new(Mutex::new(None));
}

fn log_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ani-es-rs")
        .join("logs")
        .join("ani-es-rs.log")
}

pub fn init(level: &str) {
    let path = log_path();
    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("No se pudo crear el directorio de logs: {error}");
        }
    }

    let mut state = STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *state = Some(LoggerState {
        level: Level::from_str(level),
        file_path: path,
    });
}

pub fn log(level: Level, msg: &str) {
    let state = STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let state = match state.as_ref() {
        Some(s) => s,
        None => return,
    };

    state.log(level, msg);
}

pub fn debug(msg: &str) {
    log(Level::Debug, msg);
}

pub fn info(msg: &str) {
    log(Level::Info, msg);
}

pub fn warn(msg: &str) {
    log(Level::Warn, msg);
}

fn format_log_line(level: Level, msg: &str) -> String {
    let prefix = match level {
        Level::Debug => "[DEBUG]",
        Level::Info => "[INFO]",
        Level::Warn => "[WARN]",
        Level::Error => "[ERROR]",
    };
    format!(
        "{} {} {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        prefix,
        msg
    )
}

pub fn error(msg: &str) {
    log(Level::Error, msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_line_starts_with_timestamp_and_level() {
        let line = format_log_line(Level::Info, "hello");
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 4, "{line}");
        assert_eq!(parts[2], "[INFO]");
        assert_eq!(parts[3], "hello");
        assert_eq!(parts[0].len(), 10, "{line}");
        assert!(
            parts[0].chars().enumerate().all(|(i, c)| {
                if i == 4 || i == 7 {
                    c == '-'
                } else {
                    c.is_ascii_digit()
                }
            }),
            "{line}"
        );
    }

    #[test]
    fn log_line_formats_each_level() {
        assert!(format_log_line(Level::Debug, "d").contains("[DEBUG]"));
        assert!(format_log_line(Level::Warn, "w").contains("[WARN]"));
        assert!(format_log_line(Level::Error, "e").contains("[ERROR]"));
    }
}
