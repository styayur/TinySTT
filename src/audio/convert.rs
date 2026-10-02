//! Sample conversion and a small streaming linear resampler.
//!
//! Linear interpolation is sufficient for the 16 kHz speech input used by
//! SenseVoice and avoids pulling in a general-purpose DSP framework.

use crate::error::{Result, TinySttError};

pub trait SampleToF32 {
    fn to_f32(self) -> f32;
}

macro_rules! impl_float_sample {
    ($($ty:ty),* $(,)?) => {
        $(impl SampleToF32 for $ty {
            fn to_f32(self) -> f32 {
                self as f32
            }
        })*
    };
}

macro_rules! impl_int_sample {
    ($($ty:ty => $scale:expr),* $(,)?) => {
        $(impl SampleToF32 for $ty {
            fn to_f32(self) -> f32 {
                self as f32 / $scale
            }
        })*
    };
}

impl_float_sample!(f32, f64);
impl_int_sample!(
    i8 => 128.0,
    i16 => 32768.0,
    i32 => 2_147_483_648.0,
    i64 => 9_223_372_036_854_775_808.0,
);

impl SampleToF32 for u8 {
    fn to_f32(self) -> f32 {
        (self as f32 - 128.0) / 128.0
    }
}

impl SampleToF32 for u16 {
    fn to_f32(self) -> f32 {
        (self as f32 - 32768.0) / 32768.0
    }
}

impl SampleToF32 for u32 {
    fn to_f32(self) -> f32 {
        (self as f64 - 2_147_483_648.0) as f32 / 2_147_483_648.0
    }
}

impl SampleToF32 for u64 {
    fn to_f32(self) -> f32 {
        (self as f64 - 9_223_372_036_854_775_808.0) as f32 / 9_223_372_036_854_775_808.0
    }
}

pub fn downmix_interleaved<T: Copy + SampleToF32>(input: &[T], channels: usize) -> Vec<f32> {
    if channels <= 1 || input.is_empty() {
        return input.iter().map(|sample| (*sample).to_f32()).collect();
    }

    let complete = input.len() / channels;
    let mut output = Vec::with_capacity(complete);
    for frame in input.chunks_exact(channels).take(complete) {
        let sum: f32 = frame.iter().map(|sample| (*sample).to_f32()).sum();
        output.push(sum / channels as f32);
    }
    output
}

#[derive(Debug, Clone)]
pub struct StreamingResampler {
    input_rate: u32,
    output_rate: u32,
    ratio: f64,
    next_global_position: f64,
    input_base: u64,
    last_sample: Option<f32>,
}

impl StreamingResampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Result<Self> {
        if input_rate == 0 || output_rate == 0 {
            return Err(TinySttError::Resample(
                "sample rates must be greater than zero".to_string(),
            ));
        }
        Ok(Self {
            input_rate,
            output_rate,
            ratio: input_rate as f64 / output_rate as f64,
            next_global_position: 0.0,
            input_base: 0,
            last_sample: None,
        })
    }

    pub fn input_rate(&self) -> u32 {
        self.input_rate
    }

    pub fn output_rate(&self) -> u32 {
        self.output_rate
    }

    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }
        if self.input_rate == self.output_rate {
            self.input_base += input.len() as u64;
            self.last_sample = input.last().copied();
            return input.to_vec();
        }

        let end_global = self.input_base + input.len() as u64;
        let mut output = Vec::with_capacity(
            ((input.len() as f64 / self.ratio).ceil() as usize).saturating_add(2),
        );

        let sample_at = |global_index: u64| -> f32 {
            if global_index < self.input_base {
                self.last_sample.unwrap_or(0.0)
            } else {
                input[(global_index - self.input_base) as usize]
            }
        };

        while self.next_global_position + 1.0 <= end_global as f64 {
            let index = self.next_global_position.floor() as u64;
            let fraction = (self.next_global_position - index as f64) as f32;
            let left = sample_at(index);
            let right = sample_at(index + 1);
            output.push(left + (right - left) * fraction);
            self.next_global_position += self.ratio;
        }

        self.input_base = end_global;
        self.last_sample = input.last().copied();
        output
    }
}

pub fn resample_offline(input: &[f32], input_rate: u32, output_rate: u32) -> Result<Vec<f32>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    let mut resampler = StreamingResampler::new(input_rate, output_rate)?;
    let mut output = resampler.process(input);
    let expected = (input.len() as f64 * output_rate as f64 / input_rate as f64).round() as usize;
    if output.len() < expected {
        if let Some(&last) = input.last() {
            output.resize(expected, last);
        }
    } else {
        output.truncate(expected);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_to_mono_averages_channels() {
        let mono = downmix_interleaved(&[1.0_f32, -1.0, 0.5, 0.5], 2);
        assert_eq!(mono, vec![0.0, 0.5]);
    }

    #[test]
    fn i16_to_f32_scales_to_unit_range() {
        let mono = downmix_interleaved(&[0_i16, i16::MAX, i16::MIN], 1);
        assert_eq!(mono[0], 0.0);
        assert!(mono[1] > 0.99);
        assert!(mono[2] <= -1.0);
    }

    #[test]
    fn resample_length_is_sane() {
        let input = vec![0.0_f32; 48_000];
        let output = resample_offline(&input, 48_000, 16_000).unwrap();
        assert_eq!(output.len(), 16_000);
    }
}
