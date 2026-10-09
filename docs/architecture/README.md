# Architecture evidence

Source review: `ab3e795c02df5fe06bbca54d8844507b34097061` (2026-10-09).

The marked Mermaid block in [README](../../README.md) is the only maintained diagram source. GitHub renders it natively in the reader's theme. No duplicate SVG or independent `.mmd` is committed; extracted Mermaid and SVG files are disposable verification artifacts.

TinySTT is a native desktop push-to-talk application with an additional WAV CLI, not a CLI-only SDK. Capture is controlled by key press/release or UI buttons. Preprocessing downmixes and resamples; there is no separate VAD module. The ASR worker owns one recognizer and processes requests serially. The UI rejects recording while busy and forwards model/recognition errors.

Recording cancellation discards buffered audio before inference; in-flight ASR cancellation is not implemented. Audio/transcripts remain local at runtime. Clipboard delivery crosses into the OS and optional auto-paste affects the focused application; debug WAV export is explicit. See the existing architecture document for buffer contention, state transitions and failure details.

## Source map

- [src/app.rs](../../src/app.rs): `worker.recognize`, `clipboard::write_text`, `paste::send_ctrl_v`
- [src/hotkey.rs](../../src/hotkey.rs): `Pressed`, `Released`
- [src/audio/capture.rs](../../src/audio/capture.rs): `downmix_interleaved`, `StreamingResampler`
- [src/audio/buffer.rs](../../src/audio/buffer.rs): `MAX_RECORDING`
- [src/asr/worker.rs](../../src/asr/worker.rs): `mpsc::channel`, `clean_transcript`
- [src/asr/sensevoice.rs](../../src/asr/sensevoice.rs): `OfflineRecognizer`, `self.recognizer.decode`
- [src/state.rs](../../src/state.rs): `Recognizing`

The anchors in `evidence.json` catch renamed/deleted source symbols; they do not prove call semantics. The source review above checked the actual call sites and boundaries. A significant change to data flow, persistence, authentication, recovery or process boundaries requires reviewing this diagram and updating the evidence. Routine edits do not require redrawing it.

## Verification

Requires Python 3, Node.js 22+ and network access for the documentation-only Mermaid CLI. From the repository root:

```sh
python docs/architecture/verify.py --render
```

This checks local README image references and source anchors, extracts the authoritative block, renders it twice with Mermaid CLI 11.12.0 using deterministic IDs, compares SVG bytes, validates SVG XML, and also renders the dark theme. If the bundled browser is unavailable, pass `--chrome /absolute/path/to/chrome` (or set `PUPPETEER_EXECUTABLE_PATH`). The CLI version is pinned; its transitive npm dependencies and the browser are environment-dependent, so the byte comparison proves repeatability within the same installed toolchain. Output goes to a temporary directory, never application runtime dependencies. GitHub Markdown/browser rendering still requires visual review; CLI validation alone is not evidence of GitHub rendering.

GitDiagram returned an initial diagram on 2026-10-09 for the public repository as a discovery aid. Its generated output is not imported as authoritative architecture or licensed artwork. No private source, config or credentials were submitted.

Existing repository licenses and third-party notices continue to apply. These diagrams are documentation authored from this repository's public source; no app icons, installer assets or third-party marks are replaced.
