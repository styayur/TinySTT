//! sherpa-onnx SenseVoice INT8 implementation.
//!
//! Initialization follows the official Rust example:
//! `rust-api-examples/examples/sense_voice.rs` in k2-fsa/sherpa-onnx.

use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig, OfflineSenseVoiceModelConfig};

use crate::asr::SpeechRecognizer;
use crate::error::{Result, TinySttError};
use crate::paths::ModelPaths;

pub struct SenseVoiceRecognizer {
    recognizer: OfflineRecognizer,
}

impl SenseVoiceRecognizer {
    pub fn new(paths: &ModelPaths, num_threads: u8, language: &str) -> Result<Self> {
        let mut config = OfflineRecognizerConfig::default();
        config.model_config.sense_voice = OfflineSenseVoiceModelConfig {
            model: Some(paths.model.display().to_string()),
            language: Some(language.to_string()),
            use_itn: true,
        };
        config.model_config.tokens = Some(paths.tokens.display().to_string());
        config.model_config.provider = Some("cpu".to_string());
        config.model_config.num_threads = num_threads as i32;
        config.model_config.debug = false;

        let recognizer = OfflineRecognizer::create(&config).ok_or_else(|| {
            TinySttError::ModelLoadFailed(format!(
                "sherpa-onnx could not initialize {}",
                paths.model.display()
            ))
        })?;

        Ok(Self { recognizer })
    }
}

impl SpeechRecognizer for SenseVoiceRecognizer {
    fn recognize(&mut self, samples: &[f32], sample_rate: u32) -> Result<String> {
        if samples.is_empty() {
            return Err(TinySttError::Recognition("no audio samples".to_string()));
        }
        let stream = self.recognizer.create_stream();
        stream.accept_waveform(sample_rate as i32, samples);
        self.recognizer.decode(&stream);
        let result = stream.get_result().ok_or_else(|| {
            TinySttError::Recognition("sherpa-onnx returned no result".to_string())
        })?;
        Ok(result.text)
    }
}
