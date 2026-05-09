#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "=== 构建 release ==="
cargo build --release

echo "=== 打包到 ./redra ==="
# 确保清理旧内容
rm -rf redra

# 创建完整的目录结构（cp -r 不会自动创建中间目录）
mkdir -p redra/assets/{fonts,init,materials}

# 检查二进制是否存在
if [ ! -f target/release/redra ]; then
    echo "错误: target/release/redra 不存在，编译可能失败" >&2
    exit 1
fi

# 可执行文件
cp target/release/redra redra/

# 资源文件（使用源目录/. 确保复制内容而非目录本身）
cp -r assets/fonts/.     redra/assets/fonts/
cp -r assets/init/.      redra/assets/init/
cp -r assets/materials/. redra/assets/materials/

# 验证
if [ ! -f redra/redra ]; then
    echo "错误: redra/redra 复制失败" >&2
    exit 1
fi

echo "=== 完成 ==="
echo "目录结构:"
find redra -type f | head -30
echo ""
echo "可执行: ./redra/redra"
