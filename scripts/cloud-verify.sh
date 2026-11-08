#!/usr/bin/env bash
# 云端冒烟测试：跑单元测试，再用真实网络拉一次天气和空气质量。
# 用法：bash scripts/cloud-verify.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "== 单元测试 =="
cargo test -p panahon-core --features network -- --nocapture

echo
echo "== 工作区检查 =="
cargo check --workspace --features network

echo
echo "== Android 目标检查（只需要标准库，不需要 NDK）=="
cargo check --workspace --target aarch64-linux-android --features network

echo
echo "== 联网探活 =="
cargo run -p panahon-core --example live --features network

echo
echo "全部通过"
