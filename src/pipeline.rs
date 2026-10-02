//! Recording pipeline wrapper used by the UI.

#[cfg(feature = "native")]
use crate::audio::{AudioCapture, CapturedAudio};
#[cfg(feature = "native")]
use crate::error::Result;
#[cfg(feature = "native")]
use crate::settings::Settings;

#[cfg(feature = "native")]
pub struct RecordingPipeline {
    capture: Option<AudioCapture>,
}

#[cfg(feature = "native")]
impl RecordingPipeline {
    pub fn start(settings: &Settings) -> Result<Self> {
        let microphone =
            (!settings.microphone.trim().is_empty()).then_some(settings.microphone.as_str());
        Ok(Self {
            capture: Some(AudioCapture::start(microphone)?),
        })
    }

    pub fn capture(&self) -> Option<&AudioCapture> {
        self.capture.as_ref()
    }

    pub fn stop(mut self) -> CapturedAudio {
        self.capture
            .take()
            .expect("recording pipeline can only be stopped once")
            .stop()
    }

    pub fn cancel(mut self) {
        if let Some(capture) = self.capture.take() {
            capture.cancel();
        }
    }
}
