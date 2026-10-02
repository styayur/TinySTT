//! CPAL microphone capture.
//!
//! The realtime callback only converts the incoming buffer to 16 kHz mono and
//! appends it to a bounded buffer. It never performs recognition or blocking I/O.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;

use crate::audio::buffer::RecordingBuffer;
use crate::audio::convert::{downmix_interleaved, StreamingResampler};
use crate::audio::TARGET_SAMPLE_RATE;
use crate::error::{Result, TinySttError};

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureEvent {
    MaxReached,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapturedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub duration_seconds: f32,
    pub device_name: String,
    pub max_reached: bool,
}

pub struct AudioCapture {
    stream: Option<cpal::Stream>,
    buffer: Arc<Mutex<RecordingBuffer>>,
    events: Receiver<CaptureEvent>,
    max_reached: Arc<AtomicBool>,
    dropped_callbacks: Arc<AtomicUsize>,
    device_name: String,
}

impl AudioCapture {
    pub fn start(device_name: Option<&str>) -> Result<Self> {
        let host = cpal::default_host();
        let device = select_device(&host, device_name.unwrap_or_default())?;
        let name = device
            .name()
            .unwrap_or_else(|_| "Unknown microphone".to_string());
        let supported = device
            .default_input_config()
            .map_err(|e| TinySttError::AudioDeviceUnavailable(format!("{name}: {e}")))?;
        let sample_rate = supported.sample_rate().0;
        let channels = supported.channels() as usize;
        let sample_format = supported.sample_format();
        let stream_config = supported.config();

        let buffer = Arc::new(Mutex::new(RecordingBuffer::default()));
        let max_reached = Arc::new(AtomicBool::new(false));
        let dropped_callbacks = Arc::new(AtomicUsize::new(0));
        let (event_tx, event_rx) = mpsc::channel();

        let stream = build_stream(
            &device,
            &stream_config,
            sample_format,
            sample_rate,
            channels,
            buffer.clone(),
            max_reached.clone(),
            dropped_callbacks.clone(),
            event_tx,
        )?;
        stream
            .play()
            .map_err(|e| TinySttError::AudioCapture(format!("failed to start {name}: {e}")))?;

        Ok(Self {
            stream: Some(stream),
            buffer,
            events: event_rx,
            max_reached,
            dropped_callbacks,
            device_name: name,
        })
    }

    pub fn poll_event(&self) -> Option<CaptureEvent> {
        self.events.try_recv().ok()
    }

    pub fn elapsed_samples(&self) -> usize {
        lock_buffer(&self.buffer).len()
    }

    pub fn duration_seconds(&self) -> f32 {
        self.elapsed_samples() as f32 / TARGET_SAMPLE_RATE as f32
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn dropped_callbacks(&self) -> usize {
        self.dropped_callbacks.load(Ordering::Relaxed)
    }

    pub fn stop(mut self) -> CapturedAudio {
        if let Some(stream) = self.stream.take() {
            let _ = stream.pause();
            drop(stream);
        }
        let (samples, max_reached_in_buffer) = {
            let buffer = lock_buffer(&self.buffer);
            let max_reached_in_buffer = buffer.is_truncated();
            (buffer.samples().to_vec(), max_reached_in_buffer)
        };
        let duration_seconds = samples.len() as f32 / TARGET_SAMPLE_RATE as f32;
        CapturedAudio {
            samples,
            sample_rate: TARGET_SAMPLE_RATE,
            duration_seconds,
            device_name: self.device_name.clone(),
            max_reached: self.max_reached.load(Ordering::Relaxed) || max_reached_in_buffer,
        }
    }

    pub fn cancel(mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.pause();
            drop(stream);
        }
        lock_buffer(&self.buffer).clear();
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.pause();
            drop(stream);
        }
    }
}

fn select_device(host: &cpal::Host, requested_name: &str) -> Result<cpal::Device> {
    let requested_name = requested_name.trim();
    if requested_name.is_empty() {
        return host.default_input_device().ok_or_else(|| {
            TinySttError::AudioDeviceUnavailable("no default input device".to_string())
        });
    }

    let devices = host
        .input_devices()
        .map_err(|e| TinySttError::AudioDeviceUnavailable(e.to_string()))?;
    for device in devices {
        if device
            .name()
            .map(|name| name.eq_ignore_ascii_case(requested_name))
            .unwrap_or(false)
        {
            return Ok(device);
        }
    }

    Err(TinySttError::AudioDeviceUnavailable(format!(
        "configured microphone '{requested_name}' was not found"
    )))
}

pub fn input_device_names() -> Vec<String> {
    let host = cpal::default_host();
    match host.input_devices() {
        Ok(devices) => devices.filter_map(|device| device.name().ok()).collect(),
        Err(_) => Vec::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: SampleFormat,
    input_rate: u32,
    channels: usize,
    buffer: Arc<Mutex<RecordingBuffer>>,
    max_reached: Arc<AtomicBool>,
    dropped_callbacks: Arc<AtomicUsize>,
    event_tx: mpsc::Sender<CaptureEvent>,
) -> Result<cpal::Stream> {
    let error_tx = event_tx.clone();
    let error_fn = move |error: cpal::StreamError| {
        let _ = error_tx.send(CaptureEvent::Error(error.to_string()));
    };

    macro_rules! build {
        ($sample:ty) => {{
            let callback_buffer = buffer.clone();
            let callback_max = max_reached.clone();
            let callback_dropped = dropped_callbacks.clone();
            let callback_events = event_tx.clone();
            let mut resampler = StreamingResampler::new(input_rate, TARGET_SAMPLE_RATE)?;
            device.build_input_stream(
                config,
                move |data: &[$sample], _| {
                    if let Ok(mut target) = callback_buffer.try_lock() {
                        let mono = downmix_interleaved(data, channels);
                        let converted = resampler.process(&mono);
                        let result = target.append(&converted);
                        if result.limit_reached && !callback_max.swap(true, Ordering::Relaxed) {
                            let _ = callback_events.send(CaptureEvent::MaxReached);
                        }
                    } else {
                        callback_dropped.fetch_add(1, Ordering::Relaxed);
                    }
                },
                error_fn,
                None,
            )
        }};
    }

    let stream = match sample_format {
        SampleFormat::I8 => build!(i8),
        SampleFormat::I16 => build!(i16),
        SampleFormat::I32 => build!(i32),
        SampleFormat::I64 => build!(i64),
        SampleFormat::U8 => build!(u8),
        SampleFormat::U16 => build!(u16),
        SampleFormat::U32 => build!(u32),
        SampleFormat::U64 => build!(u64),
        SampleFormat::F32 => build!(f32),
        SampleFormat::F64 => build!(f64),
        SampleFormat::I24 => {
            return Err(TinySttError::UnsupportedAudioFormat(
                "24-bit packed audio is not supported".to_string(),
            ));
        }
        other => {
            return Err(TinySttError::UnsupportedAudioFormat(other.to_string()));
        }
    };

    stream.map_err(|e| TinySttError::AudioCapture(e.to_string()))
}

fn lock_buffer(buffer: &Arc<Mutex<RecordingBuffer>>) -> std::sync::MutexGuard<'_, RecordingBuffer> {
    buffer
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_rate_and_limit_are_consistent() {
        assert_eq!(TARGET_SAMPLE_RATE, 16_000);
        assert_eq!(
            RecordingBuffer::new(crate::audio::MAX_RECORDING_SECONDS).len(),
            0
        );
    }
}
