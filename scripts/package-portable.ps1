param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$')]
    [string]$Version,

    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Root = (Resolve-Path -LiteralPath $Root).Path
$Dist = Join-Path $Root "dist"
$Stage = Join-Path $Dist "TinySTT-windows-x64"
$Zip = Join-Path $Dist "TinySTT-windows-x64-v$Version.zip"
$Sums = Join-Path $Dist "SHA256SUMS.txt"

if (-not $SkipBuild) {
    cargo build --release --manifest-path (Join-Path $Root "Cargo.toml")
}

$Exe = Join-Path $Root "target\release\TinySTT.exe"
if (-not (Test-Path -LiteralPath $Exe -PathType Leaf)) {
    throw "Release executable not found: $Exe"
}

New-Item -ItemType Directory -Force -Path $Dist | Out-Null

if (Test-Path -LiteralPath $Stage) {
    $ResolvedStage = (Resolve-Path -LiteralPath $Stage).Path
    $ResolvedDist = (Resolve-Path -LiteralPath $Dist).Path
    if (-not $ResolvedStage.StartsWith($ResolvedDist, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to remove staging directory outside dist: $ResolvedStage"
    }
    Remove-Item -LiteralPath $ResolvedStage -Recurse -Force
}

New-Item -ItemType Directory -Force -Path $Stage | Out-Null
Copy-Item -LiteralPath $Exe -Destination (Join-Path $Stage "TinySTT.exe")
Copy-Item -LiteralPath (Join-Path $Root "README.txt") -Destination $Stage
Copy-Item -LiteralPath (Join-Path $Root "LICENSE") -Destination $Stage
Copy-Item -LiteralPath (Join-Path $Root "THIRD_PARTY_NOTICES.md") -Destination $Stage
Copy-Item -LiteralPath (Join-Path $Root "config.example.toml") -Destination $Stage

$DocsDir = Join-Path $Stage "docs"
New-Item -ItemType Directory -Force -Path $DocsDir | Out-Null
Copy-Item -LiteralPath (Join-Path $Root "docs\architecture.md") -Destination $DocsDir
Copy-Item -LiteralPath (Join-Path $Root "docs\model-provenance.md") -Destination $DocsDir

$ModelDir = Join-Path $Stage "models\sensevoice"
New-Item -ItemType Directory -Force -Path $ModelDir | Out-Null
@"
Download the SenseVoice INT8 model and place these files here:

  model.int8.onnx
  tokens.txt

Official model:
https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2

TinySTT never downloads model files automatically.
"@ | Set-Content -LiteralPath (Join-Path $ModelDir "PLACE_MODEL_HERE.txt") -Encoding UTF8

if (Test-Path -LiteralPath $Zip) {
    Remove-Item -LiteralPath $Zip -Force
}
Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip -CompressionLevel Optimal

$Hash = (Get-FileHash -LiteralPath $Zip -Algorithm SHA256).Hash.ToLowerInvariant()
"$Hash  $(Split-Path -Leaf $Zip)" | Set-Content -LiteralPath $Sums -Encoding ASCII

Write-Host "Created: $Zip"
Write-Host "Created: $Sums"
