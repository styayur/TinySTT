//! Minimal, dependency-free file logger.
//!
//! Transcripts and raw audio are never written to the log.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

pub fn init(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let file = OpenOptions::new().create(true).append(true).open(path).ok();
    let _ = LOG_FILE.set(Mutex::new(file));
}

pub fn info(args: std::fmt::Arguments<'_>) {
    write_line("INFO", args);
}

pub fn error(args: std::fmt::Arguments<'_>) {
    write_line("ERROR", args);
}

fn write_line(level: &str, args: std::fmt::Arguments<'_>) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let line = format!("[{timestamp}] [{level}] {args}");
    if let Some(slot) = LOG_FILE.get() {
        if let Ok(mut guard) = slot.lock() {
            if let Some(file) = guard.as_mut() {
                let _ = writeln!(file, "{line}");
                let _ = file.flush();
            }
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::log::info(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::log::error(format_args!($($arg)*)) };
}
