# Native security backport

TinySTT keeps the sherpa-onnx 1.13.8 native ABI. The local `sherpa-onnx-sys` fork, `1.13.8+tinystt.zip9.1`, reuses the reviewed TinyTTS ZIP 9.1.3 build-dependency backport. FFI sources and the Apache-2.0 license text match the verified crates.io 1.13.8 archive. Windows still downloads the hash-pinned 1.13.8 TAR archive.

Do not replace this fork with sherpa-onnx 2.x until SenseVoice real inference passes, including cancellation and recovery. A shared Siny runtime also needs the Kokoro unknown-word regression that TinyTTS already recorded. This document does not claim an iOS device test or that the Windows speech path executed the vulnerable ZIP extractor.
