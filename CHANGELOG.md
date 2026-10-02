# Changelog

## 0.1.0 - 2026-10-03

- Initial TinySTT preview.
- Offline SenseVoice INT8 inference through sherpa-onnx.
- CPAL microphone capture with 16 kHz mono conversion and a 60-second bound.
- `Ctrl+Alt+D` push-to-talk with separate key-down and key-up handling.
- Single background ASR worker; no concurrent recognition.
- Clipboard output, optional auto-paste, settings, tray, and WAV debug export.
- `--file` WAV CLI test mode.
- Windows CI and portable release packaging.
