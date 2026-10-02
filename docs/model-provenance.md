# Model provenance

TinySTT v0.1 uses one ASR model:

- Name: `sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17`
- Source repository: <https://github.com/k2-fsa/sherpa-onnx>
- Release tag: `asr-models`
- Official asset:
  <https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2>
- Asset size: 163,002,883 bytes
- Required files: `model.int8.onnx`, `tokens.txt`
- Runtime: CPU, ONNX Runtime through sherpa-onnx
- SenseVoice project: <https://github.com/QwenAudio/SenseVoice>
- Model/source license: MIT

The model is **not** committed to the TinySTT repository and is **not** included
in the portable ZIP. The user downloads it from the official upstream release.
SHA-256 of the upstream archive:

```text
7D1EFA2138A65B0B488DF37F8B89E3D91A60676E416F515B952358D83DFD347E
```

TinySTT does not silently download, replace, fine-tune, or modify the model.
