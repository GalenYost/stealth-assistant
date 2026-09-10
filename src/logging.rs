use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;

struct FileLogger(Mutex<File>);

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        if let Ok(mut file) = self.0.lock() {
            let _ = writeln!(
                file,
                "[{}] {}: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            );
            let _ = file.flush();
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.0.lock() {
            let _ = file.flush();
        }
    }
}

static LOGGER: OnceLock<FileLogger> = OnceLock::new();

pub fn init(log_file_name: &str) -> Result<PathBuf, String> {
    let path = log_path(log_file_name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("failed to open log file {}: {}", path.display(), e))?;

    let logger = LOGGER.get_or_init(|| FileLogger(Mutex::new(file)));
    if log::set_logger(logger).is_err() {
        return Err("a logger is already initialized".to_string());
    }
    log::set_max_level(LevelFilter::Info);
    Ok(path)
}

pub fn log_path(file_name: &str) -> PathBuf {
    crate::config::config_dir()
        .map(|dir| dir.join("stealth-assistant").join(file_name))
        .unwrap_or_else(|| PathBuf::from(file_name))
}
