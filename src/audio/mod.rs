//! Audio capture, conversion, bounded buffering, and WAV helpers.

pub mod buffer;
#[cfg(feature = "native")]
pub mod capture;
pub mod convert;
pub mod wav;

pub use buffer::RecordingBuffer;
#[cfg(feature = "native")]
pub use capture::{AudioCapture, CaptureEvent, CapturedAudio};
pub use convert::{downmix_interleaved, resample_offline, StreamingResampler};
pub use wav::{read_wav_as_16k_mono, write_wav_16k_mono};

pub const TARGET_SAMPLE_RATE: u32 = 16_000;
pub const MAX_RECORDING_SECONDS: u32 = 60;
pub const MIN_RECORDING_SECONDS: f32 = 0.25;
