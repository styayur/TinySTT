# TinySTT

使用 SenseVoice INT8 和 sherpa-onnx 的 Windows 离线按键语音识别工具。

[English](README.md) · [v0.1.0 下载](https://github.com/styayur/TinySTT/releases/tag/v0.1.0) · [详细使用参考](docs/USAGE.md)

**早期预览版。** 按住 `Ctrl+Alt+D` 说话，松开后识别并复制文字。本地 CPU 推理不会上传音频或识别结果。未签名便携包和源码构建都需要另行下载模型。

## 真实识别输出

解压 v0.1.0 便携包并放好模型后，在程序目录运行：

```powershell
.\TinySTT.exe --file .\models\sensevoice\test_wavs\en.wav --model-dir .\models\sensevoice --language en
```

09-10-2026 记录的实际 stdout：

```text
The tribal chieftain called for the boy and presented him with 50 pieces of code.
```

这是模型附带样本的原始结果；最后一个词保留原样，不是准确率基准、麦克风验收或 GUI 截图。见[运行来源与捕获限制](docs/architecture/DEMONSTRATION.md)。

## 安装与首次运行

使用 Windows x64。从 [v0.1.0](https://github.com/styayur/TinySTT/releases/tag/v0.1.0) 下载 ZIP 及 `SHA256SUMS.txt`，逐项核对文件名和完整 SHA-256 后解压。便携包包含 EXE、配置示例、声明和空模型目录，不含模型。包未签名；校验和只能确认与公开包一致，不能代替代码签名。

在解压目录下载固定版本的 SenseVoice，**先校验再解压**，然后移到默认目录。以下适用于新安装，不要覆盖已有 `models/sensevoice`。

```powershell
New-Item -ItemType Directory -Force models | Out-Null
curl.exe -L -o sensevoice.tar.bz2 `
  https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2

Get-FileHash .\sensevoice.tar.bz2 -Algorithm SHA256
$expected = '7D1EFA2138A65B0B488DF37F8B89E3D91A60676E416F515B952358D83DFD347E'
if ((Get-FileHash ./sensevoice.tar.bz2 -Algorithm SHA256).Hash -ne $expected) { throw 'Model checksum mismatch' }

tar -xf .\sensevoice.tar.bz2 -C .\models
Move-Item `
  .\models\sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17 `
  .\models\sensevoice
Remove-Item .\sensevoice.tar.bz2
```

EXE 旁需有 `models/sensevoice/model.int8.onnx` 与 `models/sensevoice/tokens.txt`。保留 `test_wavs` 可运行上方示例。约 156 MiB 的模型有独立许可，见[来源记录](docs/model-provenance.md)。缺少模型时应用列出搜索路径，不会自动下载。

```powershell
.\TinySTT.exe
```

在 Windows 允许麦克风访问，等待模型加载后按住快捷键说话并松开。也可用 Record / Stop / Cancel；结果自动复制到剪贴板。自动粘贴会向当前应用插入文字，因此默认关闭。

## 从源码构建

需要 stable Rust 的 `x86_64-pc-windows-msvc` 目标、Visual Studio Build Tools 的 **Desktop development with C++**，以及 Windows 可识别的麦克风（录音时）。首次原生构建下载 sherpa-onnx 库；运行时推理离线。

```powershell
git clone https://github.com/styayur/TinySTT.git
cd TinySTT
cargo build --release
```

将模型放入 `target/release/models/sensevoice` 后运行 `target/release/TinySTT.exe`，或指定模型目录。便携包用户不需要 Rust / C++ 工具。

## 功能、配置与限制

支持中文、英文、粤语、日语和韩语，单个复用 ASR worker，60 秒有界录音，托盘控制、剪贴板、可选调试 WAV 导出及无麦克风 WAV CLI。小于 250 ms 的录音被忽略，识别忙碌时拒绝新录音。没有 VAD、流式部分结果、说话人分离或运行中 ASR 取消。

设置在 EXE 旁的 `config/config.toml`，见 [config.example.toml](config.example.toml)。模型目录、线程数（1/2/4）和语言修改后重启；快捷键、麦克风和自动粘贴在保存/下次录音生效。[详细配置及 CLI 参数](docs/USAGE.md#configuration)保留全部字段。

无账号、遥测、更新检查或运行时 HTTP 服务；日志不含原始音频和完整文本。剪贴板及主动导出的 WAV 仍须妥善保管。模型识别质量、麦克风驱动和快捷键冲突需要真实设备验收，模拟测试不能替代。取消录音仅丢弃推理前的缓冲，不中断已运行的 ASR。

架构图唯一维护源见[英文版](README.md#architecture)；[源码依据](docs/architecture/README.md)及[线程与状态](docs/architecture.md)说明 Rust 预处理、串行识别和错误边界。

## 测试、贡献与后续工作

在仓库根目录运行：

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --all-features
cargo build --release
```

[Windows CI](https://github.com/styayur/TinySTT/actions/workflows/ci.yml)使用这些检查。单元测试使用模拟识别器，不加载真实模型。见[贡献指南](CONTRIBUTING.md)、[Issues](https://github.com/styayur/TinySTT/issues)、[安全报告](SECURITY.md)和 [CHANGELOG.md](CHANGELOG.md)。签名发布、更好的麦克风界面及可能的部分文本仍是后续方向，不是现有功能；[详细参考](docs/USAGE.md#roadmap)保留路线图和排错说明。

## 许可证

应用源码为 [AGPL-3.0-only](LICENSE)。SenseVoice、sherpa-onnx、ONNX Runtime 等保留原条款，见[第三方声明](THIRD_PARTY_NOTICES.md)与[模型来源](docs/model-provenance.md)。
