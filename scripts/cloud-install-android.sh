#!/usr/bin/env bash
# 可选：在云端装 Android SDK/NDK 和 cargo-apk，用来出 APK。
#
# 只在需要「云端产出 APK 文件」时使用。装完约 3 GB，第一次跑要十几分钟。
# 需要网络白名单额外放开 dl.google.com 和 static.rust-lang.org。
set -euo pipefail

log() { printf '[android] %s\n' "$*"; }

SDK_ROOT="${ANDROID_HOME:-/opt/android-sdk}"
NDK_VERSION="${NDK_VERSION:-27.2.12479018}"
CMDLINE_TOOLS="commandlinetools-linux-11076708_latest.zip"

if [ "$(id -u)" -ne 0 ] && ! sudo -n true 2>/dev/null; then
  log "needs root or passwordless sudo for apt"; exit 1
fi
SUDO=""
[ "$(id -u)" -ne 0 ] && SUDO="sudo"

log "installing JDK 17"
$SUDO apt-get update -qq
$SUDO apt-get install -y --no-install-recommends openjdk-17-jdk-headless unzip

if [ ! -x "$SDK_ROOT/cmdline-tools/latest/bin/sdkmanager" ]; then
  log "installing command line tools"
  $SUDO mkdir -p "$SDK_ROOT/cmdline-tools"
  curl -fsSL "https://dl.google.com/android/repository/$CMDLINE_TOOLS" -o /tmp/cmdline-tools.zip
  $SUDO unzip -q /tmp/cmdline-tools.zip -d /tmp/cmdline-tools
  $SUDO mv /tmp/cmdline-tools/cmdline-tools "$SDK_ROOT/cmdline-tools/latest"
  rm -rf /tmp/cmdline-tools /tmp/cmdline-tools.zip
fi

export ANDROID_HOME="$SDK_ROOT"
export ANDROID_NDK_HOME="$SDK_ROOT/ndk/$NDK_VERSION"
export PATH="$SDK_ROOT/cmdline-tools/latest/bin:$SDK_ROOT/platform-tools:$PATH"

log "accepting licenses"
yes | sdkmanager --licenses >/dev/null 2>&1 || true

log "installing platform, build-tools, NDK"
sdkmanager "platform-tools" "platforms;android-35" "build-tools;35.0.1" "ndk;$NDK_VERSION" >/dev/null

if ! command -v cargo-apk >/dev/null 2>&1; then
  log "installing cargo-apk"
  cargo install cargo-apk --locked
fi

log "ANDROID_HOME=$ANDROID_HOME"
log "ANDROID_NDK_HOME=$ANDROID_NDK_HOME"
log "done"
