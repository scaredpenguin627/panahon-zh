param(
    [switch]$Network,
    [ValidateSet("aarch64-linux-android", "x86_64-linux-android")]
    [string]$Target = "aarch64-linux-android"
)

$ErrorActionPreference = "Stop"
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw "Rust/cargo is required." }
if (-not (Get-Command cargo-apk -ErrorAction SilentlyContinue)) { throw "Install cargo-apk first: cargo install cargo-apk" }
if (-not $env:ANDROID_HOME) {
    $localSdk = Join-Path $PSScriptRoot "..\android-sdk"
    if (Test-Path -LiteralPath $localSdk) { $env:ANDROID_HOME = (Resolve-Path $localSdk).Path }
}
if (-not $env:ANDROID_HOME) { throw "Set ANDROID_HOME to an Android SDK (API 35 recommended)." }
if (-not $env:ANDROID_NDK_HOME) {
    $localNdk = Join-Path $env:ANDROID_HOME "ndk\27.2.12479018"
    if (Test-Path -LiteralPath $localNdk) { $env:ANDROID_NDK_HOME = (Resolve-Path $localNdk).Path }
}
if (-not $env:ANDROID_NDK_HOME) { throw "Set ANDROID_NDK_HOME to an installed NDK." }

# Use the NDK clang driver explicitly. Besides avoiding host-linker discovery
# surprises on Windows, this keeps the native library's ELF LOAD segments
# aligned for Android devices that use 16 KB pages.
$llvmBin = Join-Path $env:ANDROID_NDK_HOME "toolchains\llvm\prebuilt\windows-x86_64\bin"
$clangName = if ($Target -eq "aarch64-linux-android") {
    "aarch64-linux-android35-clang.cmd"
} else {
    "x86_64-linux-android35-clang.cmd"
}
$targetKey = $Target.ToUpperInvariant().Replace("-", "_")
$linkerPath = Join-Path $llvmBin $clangName
if (Test-Path -LiteralPath $linkerPath) {
    Set-Item -Path "Env:CARGO_TARGET_${targetKey}_LINKER" -Value $linkerPath
}
$alignmentFlags = "-C link-arg=-z -C link-arg=max-page-size=16384"
if (-not ($env:RUSTFLAGS -match "max-page-size=16384")) {
    $env:RUSTFLAGS = (([string]$env:RUSTFLAGS + " " + $alignmentFlags).Trim())
}
if (-not $env:CARGO_APK_RELEASE_KEYSTORE) { throw "Set CARGO_APK_RELEASE_KEYSTORE to your release keystore path." }
if (-not $env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD) { throw "Set CARGO_APK_RELEASE_KEYSTORE_PASSWORD before making a release APK." }

$args = @("apk", "build", "--release", "--lib", "--target", $Target)
if ($Network) { $args += "--features"; $args += "network" }
& cargo @args
if ($LASTEXITCODE -ne 0) { throw "cargo-apk failed with exit code $LASTEXITCODE" }
Write-Host "APK output: target\release\apk\"
