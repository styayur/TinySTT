//! ASR trait, SenseVoice implementation, and the single background worker.

#[cfg(feature = "native")]
pub mod sensevoice;
#[cfg(feature = "native")]
pub mod worker;

use crate::error::{Result, TinySttError};

pub trait SpeechRecognizer: Send {
    fn recognize(&mut self, samples: &[f32], sample_rate: u32) -> Result<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecognitionStats {
    pub audio_seconds: f64,
    pub inference_seconds: f64,
    pub rtf: f64,
}

pub fn clean_transcript(text: &str) -> Result<String> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        Err(TinySttError::Recognition(
            "the model returned an empty transcript".to_string(),
        ))
    } else {
        Ok(trimmed.to_string())
    }
}

#[cfg(feature = "native")]
pub use worker::{AsrWorker, WorkerEvent};

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeRecognizer;

    impl SpeechRecognizer for FakeRecognizer {
        fn recognize(&mut self, samples: &[f32], _sample_rate: u32) -> Result<String> {
            Ok(format!("fake:{}", samples.len()))
        }
    }

    #[test]
    fn fake_recognizer_obeys_abstraction() {
        let mut recognizer = FakeRecognizer;
        assert_eq!(recognizer.recognize(&[0.0; 10], 16_000).unwrap(), "fake:10");
    }

    #[test]
    fn transcript_cleanup_is_conservative() {
        assert_eq!(
            clean_transcript("  hello\r\nworld  ").unwrap(),
            "hello\nworld"
        );
        assert!(clean_transcript(" \n ").is_err());
    }
}
