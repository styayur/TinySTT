# Contributing

TinySTT is a small offline Windows utility. Contributions should preserve the
core promise:

**Hold → Speak → Release → Text.**

## Scope

The v0.1 boundary intentionally excludes VAD, streaming ASR, diarization,
speaker recognition, cloud services, plugins, history databases, translation,
LLM correction, and meeting-transcription features.

## Development

Requirements:

- Windows 10 or 11
- Rust stable, MSVC target
- Visual Studio Build Tools with Desktop development with C++

Commands:

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --all-features
cargo build --release
```

The repository does not commit model weights. Unit tests must use fake
recognizers and must not require a model download.

## Pull requests

- Keep changes focused on the push-to-talk workflow.
- Add or update tests for audio conversion, state transitions, settings, and
  error handling.
- Do not add telemetry, hidden network calls, or automatic model downloads.
- Document new dependencies and licenses in `THIRD_PARTY_NOTICES.md`.
- Keep user-facing errors actionable.
