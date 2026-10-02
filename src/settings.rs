//! Tiny local settings, persisted as TOML next to the executable.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Result, TinySttError};

pub const ALLOWED_THREADS: [u8; 3] = [1, 2, 4];
pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+D";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub model_dir: String,
    pub num_threads: u8,
    pub hotkey: String,
    pub auto_paste: bool,
    pub microphone: String,
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            model_dir: String::new(),
            num_threads: 2,
            hotkey: DEFAULT_HOTKEY.to_string(),
            auto_paste: false,
            microphone: String::new(),
            language: "auto".to_string(),
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)
            .map_err(|e| TinySttError::Settings(format!("cannot read {}: {e}", path.display())))?;
        let settings: Self = toml::from_str(&text)
            .map_err(|e| TinySttError::Settings(format!("cannot parse {}: {e}", path.display())))?;
        Ok(settings.normalized())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let normalized = self.clone().normalized();
        let text = toml::to_string_pretty(&normalized)
            .map_err(|e| TinySttError::Settings(format!("cannot serialize settings: {e}")))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TinySttError::Settings(format!("cannot create {}: {e}", parent.display()))
            })?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, text)
            .map_err(|e| TinySttError::Settings(format!("cannot write {}: {e}", tmp.display())))?;
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        std::fs::rename(&tmp, path)
            .map_err(|e| TinySttError::Settings(format!("cannot write {}: {e}", path.display())))
    }

    pub fn normalized(mut self) -> Self {
        let nearest = ALLOWED_THREADS
            .iter()
            .min_by_key(|&&n| (n as i32 - self.num_threads as i32).abs())
            .copied()
            .unwrap_or(2);
        self.num_threads = nearest;
        if self.hotkey.trim().is_empty() {
            self.hotkey = DEFAULT_HOTKEY.to_string();
        }
        if self.language.trim().is_empty() {
            self.language = "auto".to_string();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "tinystt-settings-{tag}-{}.toml",
            std::process::id()
        ));
        p
    }

    #[test]
    fn defaults_are_safe() {
        let s = Settings::default();
        assert_eq!(s.num_threads, 2);
        assert!(!s.auto_paste);
        assert_eq!(s.hotkey, DEFAULT_HOTKEY);
    }

    #[test]
    fn round_trip_preserves_values() {
        let path = temp_path("roundtrip");
        let settings = Settings {
            model_dir: "models/sensevoice".into(),
            num_threads: 4,
            hotkey: "Ctrl+Shift+D".into(),
            auto_paste: true,
            microphone: "USB microphone".into(),
            language: "zh".into(),
        };
        settings.save(&path).unwrap();
        let loaded = Settings::load(&path).unwrap();
        assert_eq!(loaded, settings);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn missing_file_uses_defaults() {
        let path = temp_path("missing");
        let _ = std::fs::remove_file(&path);
        assert_eq!(Settings::load(&path).unwrap(), Settings::default());
    }

    #[test]
    fn normalization_clamps_thread_count() {
        let settings = Settings {
            num_threads: 99,
            hotkey: String::new(),
            ..Settings::default()
        }
        .normalized();
        assert!(ALLOWED_THREADS.contains(&settings.num_threads));
        assert_eq!(settings.hotkey, DEFAULT_HOTKEY);
    }
}
