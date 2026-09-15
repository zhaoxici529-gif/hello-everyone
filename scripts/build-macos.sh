#!/usr/bin/env bash
# 在 macOS 上打 dmg。用法：
#   ./scripts/build-macos.sh              # 通用包（Intel + Apple Silicon）
#   ./scripts/build-macos.sh --arm        # 只打 Apple Silicon
#   ./scripts/build-macos.sh --intel      # 只打 Intel
set -euo pipefail

cd "$(dirname "$0")/.."

MODE="${1:---universal}"

if ! command -v pnpm >/dev/null 2>&1; then
  echo "缺少 pnpm，先执行：npm install -g pnpm"
  exit 1
fi

echo "==> 安装依赖"
pnpm install --frozen-lockfile

echo "==> 注入体验 Key"
node scripts/inject-trial-key.mjs

case "$MODE" in
  --arm)
    echo "==> 只打 Apple Silicon（aarch64-apple-darwin）"
    rustup target add aarch64-apple-darwin
    pnpm tauri build --target aarch64-apple-darwin --bundles dmg
    ;;
  --intel)
    echo "==> 只打 Intel（x86_64-apple-darwin）"
    rustup target add x86_64-apple-darwin
    pnpm tauri build --target x86_64-apple-darwin --bundles dmg
    ;;
  *)
    echo "==> 打通用包（Intel + Apple Silicon）"
    rustup target add aarch64-apple-darwin x86_64-apple-darwin
    pnpm tauri build --target universal-apple-darwin --bundles dmg
    ;;
esac

echo
echo "==> 完成，产物在："
find src-tauri/target -name "*.dmg" -maxdepth 4 2>/dev/null || true
echo
echo "提示：没有 Apple 开发者证书时 dmg 是未签名的，"
echo "首次打开需要「右键 → 打开」绕过 Gatekeeper，详见 使用说明.md。"