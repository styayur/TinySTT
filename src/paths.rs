//! Portable path resolution and model discovery.

use std::path::{Path, PathBuf};

use crate::error::{Result, TinySttError};
use crate::settings::Settings;

#[derive(Debug, Clone)]
pub struct ModelPaths {
    pub directory: PathBuf,
    pub model: PathBuf,
    pub tokens: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Paths {
    base_dir: PathBuf,
}

impl Paths {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn from_exe() -> Result<Self> {
        let exe = std::env::current_exe()
            .map_err(|e| TinySttError::Settings(format!("cannot locate executable: {e}")))?;
        let dir = exe.parent().ok_or_else(|| {
            TinySttError::Settings(format!("executable has no parent: {}", exe.display()))
        })?;
        Ok(Self::new(dir))
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn model_dir(&self) -> PathBuf {
        self.base_dir.join("models").join("sensevoice")
    }

    pub fn config_file(&self) -> PathBuf {
        self.base_dir.join("config").join("config.toml")
    }

    pub fn log_file(&self) -> PathBuf {
        self.base_dir.join("logs").join("tinystt.log")
    }

    pub fn debug_wav_file(&self) -> PathBuf {
        self.base_dir.join("debug").join("last-recording.wav")
    }

    pub fn model_candidates(&self, settings: &Settings) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        let configured = settings.model_dir.trim();
        if !configured.is_empty() {
            let configured = PathBuf::from(configured);
            let configured = if configured.is_absolute() {
                configured
            } else {
                self.base_dir.join(configured)
            };
            return vec![configured];
        }

        candidates.push(self.model_dir());

        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("models").join("sensevoice"));
        }

        // Development layout: target/debug/TinySTT.exe and repository/models.
        candidates.push(
            self.base_dir
                .join("..")
                .join("..")
                .join("models")
                .join("sensevoice"),
        );

        let mut unique = Vec::new();
        for candidate in candidates {
            if !unique.contains(&candidate) {
                unique.push(candidate);
            }
        }
        unique
    }

    pub fn resolve_model(&self, settings: &Settings) -> Result<ModelPaths> {
        let candidates = self.model_candidates(settings);
        for directory in &candidates {
            let model = directory.join("model.int8.onnx");
            let tokens = directory.join("tokens.txt");
            if model.is_file() && tokens.is_file() {
                return Ok(ModelPaths {
                    directory: directory.clone(),
                    model,
                    tokens,
                });
            }
        }

        let searched = candidates
            .iter()
            .map(|path| format!("  - {}", path.display()))
            .collect::<Vec<_>>()
            .join("\n");
        Err(TinySttError::ModelNotFound(searched))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_layout_is_stable() {
        let paths = Paths::new("C:/apps/TinySTT");
        assert_eq!(
            paths.model_dir(),
            Path::new("C:/apps/TinySTT/models/sensevoice")
        );
        assert_eq!(
            paths.config_file(),
            Path::new("C:/apps/TinySTT/config/config.toml")
        );
    }

    #[test]
    fn missing_model_lists_search_paths() {
        let base = std::env::temp_dir().join(format!("tinystt-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let settings = Settings {
            model_dir: base.join("does-not-exist").display().to_string(),
            ..Settings::default()
        };
        let paths = Paths::new(&base);
        let err = paths.resolve_model(&settings).unwrap_err();
        assert!(err
            .to_string()
            .contains("SenseVoice model files were not found."));
    }

    #[test]
    fn complete_model_is_found() {
        let base = std::env::temp_dir().join(format!("tinystt-found-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let paths = Paths::new(&base);
        let dir = paths.model_dir();
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("model.int8.onnx"), b"x").unwrap();
        std::fs::write(dir.join("tokens.txt"), b"x").unwrap();
        let settings = Settings {
            model_dir: dir.display().to_string(),
            ..Settings::default()
        };
        assert!(paths.resolve_model(&settings).is_ok());
        let _ = std::fs::remove_dir_all(base);
    }
}
