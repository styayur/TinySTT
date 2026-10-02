//! Application error type.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TinySttError {
    #[error("SenseVoice model files were not found.\nSearched paths:\n{0}")]
    ModelNotFound(String),

    #[error("Failed to load the SenseVoice model: {0}")]
    ModelLoadFailed(String),

    #[error("No usable microphone was found: {0}")]
    AudioDeviceUnavailable(String),

    #[error("Unsupported audio format: {0}")]
    UnsupportedAudioFormat(String),

    #[error("Audio capture failed: {0}")]
    AudioCapture(String),

    #[error("Audio resampling failed: {0}")]
    Resample(String),

    #[error("Recognition failed: {0}")]
    Recognition(String),

    #[error("Recognition succeeded, but clipboard copy failed: {0}")]
    ClipboardUnavailable(String),

    #[error("Global hotkey registration failed: {0}")]
    HotkeyRegistration(String),

    #[error("Automatic paste failed: {0}")]
    PasteFailed(String),

    #[error("WAV error: {0}")]
    Wav(String),

    #[error("Settings error: {0}")]
    Settings(String),

    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    #[error("The recognizer is busy.")]
    Busy,

    #[error("The recording was cancelled.")]
    Cancelled,
}

impl From<hound::Error> for TinySttError {
    fn from(value: hound::Error) -> Self {
        Self::Wav(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TinySttError>;
