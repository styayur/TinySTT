# Third-Party Notices

TinySTT application source is licensed under **AGPL-3.0-only**. Third-party
components remain under their respective licenses. This file is informational
and does not relicense any component.

## Runtime and model components

### sherpa-onnx

- Project: <https://github.com/k2-fsa/sherpa-onnx>
- License: Apache-2.0
- Use: SenseVoice offline ASR runtime and Rust bindings.
- Component: `sherpa-onnx` and `sherpa-onnx-sys` v1.13.8.

### ONNX Runtime

- Project: <https://github.com/microsoft/onnxruntime>
- License: MIT
- Use: neural-network inference engine bundled into the prebuilt
  sherpa-onnx native library.

### SenseVoice model

- Project: <https://github.com/QwenAudio/SenseVoice>
- License: MIT
- Distributed asset:
  `sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2`
- Use: ASR model weights and `tokens.txt`.
- Distribution: **not included** in the TinySTT source repository or release
  ZIP; users download the model from the official sherpa-onnx release.

### Native dependencies bundled by sherpa-onnx

The upstream sherpa-onnx prebuilt library may include Apache-2.0 or other
compatible components such as kaldi-native-fbank, kissfft, kaldi-decoder,
kaldi-fst, FST, SentencePiece, and espeak-ng. Their upstream notices and
license files should be reviewed when redistributing a modified or separately
bundled sherpa-onnx runtime. The canonical license inventory is maintained by
the sherpa-onnx project.

## Rust and application libraries

| Component | Declared license | Purpose |
|---|---|---|
| `cpal` | Apache-2.0 | Windows microphone capture |
| `eframe` / `egui` | MIT OR Apache-2.0 | native GUI |
| `hound` | Apache-2.0 | WAV reading and debug export |
| `serde` | MIT OR Apache-2.0 | settings serialization |
| `toml` | MIT OR Apache-2.0 | TOML configuration |
| `thiserror` | MIT OR Apache-2.0 | typed error definitions |
| `tray-icon` | MIT OR Apache-2.0 | system tray icon |
| `muda` | MIT OR Apache-2.0 | tray context menu |
| `windows-sys` | MIT OR Apache-2.0 | Windows clipboard, keyboard, and console FFI |

The resampler is first-party TinySTT code and adds no third-party DSP
dependency. It uses a small streaming linear interpolation implementation.

Full license texts and notices are available from each project. Cargo.lock
records the exact resolved dependency versions used for a release.

## Models and data are separate

Do not assume that the TinySTT application license applies to SenseVoice model
weights, test audio, or other downloaded data. Those materials keep the terms
published by their upstream authors.

## Disclaimer

License information is provided in good faith based on public project metadata
at the time of writing. Verify upstream licenses and notices before
redistributing binaries, native libraries, or model files.
