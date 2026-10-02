TinySTT v0.1.0 - portable Windows build
=======================================

Tiny, offline speech-to-text for Windows.
Hold Ctrl+Alt+D -> Speak -> Release -> Text.

1. Put TinySTT.exe in any folder.
2. Download the SenseVoice INT8 model:
   https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
3. Extract it and place model.int8.onnx and tokens.txt in:
   models\sensevoice\
4. Start TinySTT.exe.

TinySTT is fully offline. It does not send audio or transcripts to any server.

The model is not included in this ZIP because it is about 156 MiB and has its
own upstream license. See THIRD_PARTY_NOTICES.md and docs/model-provenance.md.

CLI test:
  TinySTT.exe --file test.wav

Full documentation:
  https://github.com/styayur/TinySTT
