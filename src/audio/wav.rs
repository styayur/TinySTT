//! Minimal WAV read/write support for the CLI and debug export.

use std::path::Path;

use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

use crate::audio::convert::SampleToF32;
use crate::audio::{resample_offline, TARGET_SAMPLE_RATE};
use crate::error::{Result, TinySttError};

pub fn read_wav_as_16k_mono(path: &Path) -> Result<Vec<f32>> {
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let raw = match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Float, 32) => reader
            .samples::<f32>()
            .collect::<std::result::Result<Vec<_>, _>>()?,
        (SampleFormat::Int, 8) => reader
            .samples::<i8>()
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .map(SampleToF32::to_f32)
            .collect(),
        (SampleFormat::Int, 16) => reader
            .samples::<i16>()
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .map(SampleToF32::to_f32)
            .collect(),
        (SampleFormat::Int, 24) => reader
            .samples::<i32>()
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .map(|sample| sample as f32 / 8_388_608.0)
            .collect(),
        (SampleFormat::Int, 32) => reader
            .samples::<i32>()
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .map(|sample| sample as f32 / 2_147_483_648.0)
            .collect(),
        _ => {
            return Err(TinySttError::UnsupportedAudioFormat(format!(
                "{} bit {} WAV",
                spec.bits_per_sample,
                match spec.sample_format {
                    SampleFormat::Float => "float",
                    SampleFormat::Int => "integer",
                }
            )))
        }
    };

    let mono = if channels <= 1 {
        raw
    } else {
        let complete = raw.len() / channels;
        raw.chunks_exact(channels)
            .take(complete)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    };

    resample_offline(&mono, spec.sample_rate, TARGET_SAMPLE_RATE)
}

pub fn write_wav_16k_mono(path: &Path, samples: &[f32]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| TinySttError::Wav(format!("cannot create {}: {e}", parent.display())))?;
    }
    let spec = WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec)?;
    for &sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        writer.write_sample(value)?;
    }
    writer.finalize()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_round_trip_converts_to_target_rate() {
        let path = std::env::temp_dir().join(format!("tinystt-{}.wav", std::process::id()));
        let spec = WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(&path, spec).unwrap();
        for _ in 0..4_800 {
            writer.write_sample(1_000_i16).unwrap();
            writer.write_sample(1_000_i16).unwrap();
        }
        writer.finalize().unwrap();

        let samples = read_wav_as_16k_mono(&path).unwrap();
        assert!((samples.len() as i64 - 1_600).abs() <= 1);
        let _ = std::fs::remove_file(path);
    }
}
