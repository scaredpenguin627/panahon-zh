param(
    [switch]$Network,
    [ValidateSet("aarch64-linux-android", "x86_64-linux-android")]
    [string]$Target = "aarch64-linux-android"
)

$ErrorActionPreference = "Stop"
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw "Rust/cargo is required." }
if (-not (Get-Command cargo-apk -ErrorAction SilentlyContinue)) { throw "Install cargo-apk first: cargo install cargo-apk" }
if (-not $env:ANDROID_HOME) { throw "Set ANDROID_HOME to an Android SDK (API 35 recommended)." }
if (-not $env:ANDROID_NDK_HOME) { throw "Set ANDROID_NDK_HOME to an installed NDK." }

$args = @("apk", "build", "--release", "--target", $Target)
if ($Network) { $args += "--features"; $args += "network" }
& cargo @args
if ($LASTEXITCODE -ne 0) { throw "cargo-apk failed with exit code $LASTEXITCODE" }
Write-Host "APK output: target\android-artifacts\release\apk\"
