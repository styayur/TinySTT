# WAV demonstration provenance

Captured 2026-10-09 on Windows, exit code 0, using the existing v0.1.0 portable executable. No microphone, private recording, clipboard paste or network inference was used. The English WAV is the upstream model package sample, downloaded using the existing README model instructions; it is not redistributed in this change.

- Executable SHA-256: `6cb81f2ee4558fc9e9cd88c788a8ecc3e544c6c9178bd6b99ef5a2def97a1268`.
- Input WAV SHA-256: `eb1eb008904465b74c304aad8342e8c7d3c6e61ffe9f66adcaca9cf0f76a93f4`.
- Arguments: `--file models/sensevoice/test_wavs/en.wav --model-dir models/sensevoice --language en` (use paths appropriate to the extracted package).
- Stdout is quoted without correction in the README; timing/debug output is not a performance claim.

The executable was an existing local portable build, not rebuilt in this documentation change. Its hash identifies exactly what ran; source-level architecture was reviewed separately at the commit in evidence.json. The public model package's licenses and provenance remain in the existing model documentation.

Native screenshot remains pending: the Windows automation helper could list windows but launching/capturing the native target failed with `GetCursorPos: access denied (0x80070005)`, and no targetable TinySTT window appeared. No substitute screenshot was generated. On an interactive Windows desktop, launch the portable app with the documented model installed, confirm the model-ready state, use non-private sample speech with auto-paste off, then capture only the readable app window. Review visible microphone/device names and paths before publishing.
