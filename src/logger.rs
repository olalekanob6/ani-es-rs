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

        let prefix = match level {
            Level::Debug => "[DEBUG]",
            Level::Info => "[INFO]",
            Level::Warn => "[WARN]",
            Level::Error => "[ERROR]",
        };

        let line = format!("{} {}\n", prefix, msg);

        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
        {
            let _ = f.write_all(line.as_bytes());
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
        let _ = fs::create_dir_all(parent);
    }

    let mut state = STATE.lock().unwrap();
    *state = Some(LoggerState {
        level: Level::from_str(level),
        file_path: path,
    });
}

pub fn log(level: Level, msg: &str) {
    let state = STATE.lock().unwrap();
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

pub fn error(msg: &str) {
    log(Level::Error, msg);
}
