#!/usr/bin/env bash
# 云端工作区初始化：装好这个仓库需要的工具链。
# 可以重复执行，每次开机跑一遍也没问题。
#
# 用法（仓库根目录）：bash scripts/cloud-install.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

log() { printf '[cloud-install] %s\n' "$*"; }

# 1. 系统依赖。容器里通常没有 sudo，失败就跳过。
SUDO=""
if [ "$(id -u)" -ne 0 ]; then
  if command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
    SUDO="sudo"
  fi
fi

if command -v apt-get >/dev/null 2>&1 && { [ "$(id -u)" -eq 0 ] || [ -n "$SUDO" ]; }; then
  log "installing build tools"
  $SUDO apt-get update -qq
  $SUDO apt-get install -y --no-install-recommends \
    build-essential pkg-config curl ca-certificates unzip
else
  log "skipping apt (no root or no apt); assuming build tools exist"
  command -v cc >/dev/null 2>&1 || log "warning: cc not found, ring will fail to build"
fi

# 2. Rust。装在 $CARGO_HOME 下，和宿主机无关。
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export PATH="$CARGO_HOME/bin:$PATH"

if ! command -v rustup >/dev/null 2>&1; then
  log "installing rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable
fi

# 交叉检查用的标准库，不需要 NDK。
rustup target add aarch64-linux-android >/dev/null 2>&1 || true

# 3. 预取依赖，缩短第一次 cargo test 的等待。
log "fetching crates"
cargo fetch --locked 2>/dev/null || cargo fetch

# 4. 可选的 Android 工具链。默认不装，见 docs/cloud-environment.md。
if [ "${PANAHON_INSTALL_ANDROID:-0}" = "1" ]; then
  bash "$REPO_ROOT/scripts/cloud-install-android.sh"
fi

log "rustc: $(rustc --version)"
log "cargo: $(cargo --version)"
log "targets: $(rustup target list --installed | tr '\n' ' ')"
log "done"
