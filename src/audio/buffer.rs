//! Bounded in-memory recording buffer.

use super::{MAX_RECORDING_SECONDS, TARGET_SAMPLE_RATE};

#[derive(Debug, Clone, PartialEq)]
pub struct RecordingBuffer {
    samples: Vec<f32>,
    max_samples: usize,
    truncated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppendResult {
    pub added: usize,
    pub limit_reached: bool,
}

impl Default for RecordingBuffer {
    fn default() -> Self {
        Self::new(MAX_RECORDING_SECONDS)
    }
}

impl RecordingBuffer {
    pub fn new(max_seconds: u32) -> Self {
        let max_samples = TARGET_SAMPLE_RATE as usize * max_seconds as usize;
        Self {
            samples: Vec::with_capacity((TARGET_SAMPLE_RATE as usize * 5).min(max_samples)),
            max_samples,
            truncated: false,
        }
    }

    pub fn append(&mut self, input: &[f32]) -> AppendResult {
        let remaining = self.max_samples.saturating_sub(self.samples.len());
        let added = input.len().min(remaining);
        self.samples.extend_from_slice(&input[..added]);
        let limit_reached = added < input.len() || self.samples.len() >= self.max_samples;
        if added < input.len() {
            self.truncated = true;
        }
        AppendResult {
            added,
            limit_reached,
        }
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn is_truncated(&self) -> bool {
        self.truncated
    }

    pub fn duration_seconds(&self) -> f32 {
        self.samples.len() as f32 / TARGET_SAMPLE_RATE as f32
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    pub fn clear(&mut self) {
        self.samples.clear();
        self.truncated = false;
    }

    pub fn into_vec(self) -> Vec<f32> {
        self.samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_clear_work() {
        let mut buffer = RecordingBuffer::new(1);
        assert_eq!(buffer.append(&[0.1, 0.2]).added, 2);
        assert_eq!(buffer.len(), 2);
        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[test]
    fn maximum_duration_is_enforced() {
        let mut buffer = RecordingBuffer::new(1);
        let result = buffer.append(&vec![0.0; TARGET_SAMPLE_RATE as usize + 100]);
        assert!(result.limit_reached);
        assert_eq!(buffer.len(), TARGET_SAMPLE_RATE as usize);
        assert!(buffer.is_truncated());
    }
}
