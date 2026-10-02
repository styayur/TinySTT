# TinySTT architecture

TinySTT keeps the runtime deliberately small. There are four meaningful
execution contexts:

1. **egui main thread** — window, controls, state machine, clipboard, settings.
2. **CPAL callback** — converts incoming samples to 16 kHz mono and appends
   them to a bounded buffer. It never runs ASR or blocking I/O.
3. **Global keyboard hook thread** — receives Windows key-down and key-up
   events and sends `Pressed` / `Released` messages to the UI.
4. **Single ASR worker thread** — owns the SenseVoice recognizer for the life
   of the process and processes recognition requests serially.

No second recognition task is allowed. While a recognition is in progress, a
new shortcut press is rejected as `Busy`.

## Recording path

```text
microphone
  -> CPAL callback
  -> device channels -> mono
  -> device sample rate -> 16 kHz f32
  -> RecordingBuffer (maximum 60 seconds)
  -> Stop
  -> ASR worker channel
  -> SenseVoice INT8
  -> deterministic trim / newline cleanup
  -> transcript UI + clipboard
  -> optional Ctrl+V
```

The buffer has a fixed maximum of 960,000 samples. Recording stops
automatically when that limit is reached. The callback uses a non-blocking
`try_lock`; if the buffer is briefly unavailable, that callback's samples are
dropped rather than blocking the realtime audio thread.

## Model lifetime

`sherpa-onnx` is initialized once in the ASR worker. The worker receives
`OfflineRecognizerConfig` values from the official Rust SenseVoice example:

```rust
OfflineSenseVoiceModelConfig {
    model: Some(model_path),
    language: Some("auto"),
    use_itn: true,
}
```

The configured provider is `cpu`, and the default thread count is 2. The
model is never reloaded for each utterance.

## State machine

```text
LoadingModel -> Idle
Idle -> Recording
Recording -> Recognizing | Cancelled | Error
Recognizing -> Completed | Error
Completed -> Idle | Recording
Cancelled -> Idle | Recording
Error -> Idle | LoadingModel
```

`Idle`, `Completed`, and `Cancelled` are the only states from which a new
recording can start. The state machine rejects invalid transitions in tests.

## Failure boundaries

- Missing model: user-facing error with every searched path.
- Missing microphone: user-facing device error.
- Sample format unsupported: explicit unsupported-format error.
- Clipboard busy: bounded retry, then a non-fatal UI message.
- Hotkey conflict: registration error shown in the UI.
- ASR failure: worker error forwarded to the UI; the process stays alive.
- ASR cancellation: intentionally not claimed. Recording cancellation is
  separate and discards the buffer before inference starts.

## Privacy boundary

The logger records lifecycle events, device names, durations, RTF, and errors.
It does not record raw audio, sample data, or the full recognized transcript.
There is no telemetry or runtime network client.
