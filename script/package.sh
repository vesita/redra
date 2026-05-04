#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "=== 构建 release ==="
cargo build --release

echo "=== 打包到 ./redra ==="
rm -rf redra
mkdir -p redra

# 可执行文件
cp target/release/redra redra/

# 资源文件
cp -r assets/fonts redra/assets/fonts
cp -r assets/init  redra/assets/init
cp -r assets/materials redra/assets/materials

echo "=== 完成 ==="
echo "目录结构:"
find redra -type f | head -30
echo ""
echo "可执行: ./redra/redra"
