"""为所有 base 颜色材质生成半透明版本。

读取 assets/materials/base/*.toml 中的每个颜色，
创建 *_transparent.toml 副本，降低 alpha 并添加 Blend 模式。
"""

import os
import re

BASE_DIRS = [
    "assets/materials/base",
    "redra/assets/materials/base",
]
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def process_file(src_path):
    with open(src_path, encoding="utf-8") as f:
        content = f.read()

    # 解析 alpha 值，默认 1.0
    alpha_match = re.search(r"base_color\.Srgba\s*=\s*\[([\d.]+),\s*([\d.]+),\s*([\d.]+),\s*([\d.]+)\]", content)
    if not alpha_match:
        alpha_match = re.search(r"base_color\s*=\s*\[([\d.]+),\s*([\d.]+),\s*([\d.]+),\s*([\d.]+)\]", content)
    if not alpha_match:
        print(f"  SKIP: no base_color found")
        return None

    r, g, b, a = map(float, alpha_match.groups())
    new_alpha = min(a, 0.5)  # 最多 0.5

    # 替换或添加 alpha_mode
    if "alpha_mode" in content:
        content = re.sub(r"alpha_mode\s*=\s*\".*?\"", 'alpha_mode = "Blend"', content)
    else:
        # 在 [material] 块内添加 alpha_mode
        content = re.sub(
            r"(\[material\])",
            "[material]\nalpha_mode = \"Blend\"",
            content
        )

    # 替换 alpha 值
    if "base_color.Srgba" in content:
        content = re.sub(
            r"(base_color\.Srgba\s*=\s*\[[\d.]+,\s*[\d.]+,\s*[\d.]+,\s*)[\d.]+(\])",
            lambda m: f"{m.group(1)}{new_alpha}{m.group(2)}",
            content
        )
    else:
        content = re.sub(
            r"(base_color\s*=\s*\[[\d.]+,\s*[\d.]+,\s*[\d.]+,\s*)[\d.]+(\])",
            lambda m: f"{m.group(1)}{new_alpha}{m.group(2)}",
            content
        )

    # 更新注释
    if content.startswith("#"):
        first_line = content.split("\n")[0]
        content = content.replace(first_line, f"# {first_line[2:].strip()}（半透明版）", 1)
    else:
        content = f"# 半透明材质\n{content}"

    return content


def main():
    all_colors = set()
    for base_dir in BASE_DIRS:
        full_dir = os.path.join(ROOT, base_dir)
        if not os.path.isdir(full_dir):
            print(f"跳过不存在: {full_dir}")
            continue

        print(f"\n处理: {base_dir}")
        for fname in sorted(os.listdir(full_dir)):
            if not fname.endswith(".toml"):
                continue
            if "_transparent" in fname:
                continue

            src_path = os.path.join(full_dir, fname)
            name = fname.replace(".toml", "")
            dst_name = f"{name}_transparent.toml"
            dst_path = os.path.join(full_dir, dst_name)

            print(f"  {name} → {dst_name}")
            new_content = process_file(src_path)
            if new_content is None:
                continue

            with open(dst_path, "w", encoding="utf-8") as f:
                f.write(new_content)
            all_colors.add(name)

    print(f"\n完成！为 {len(all_colors)} 种颜色生成了半透明版本")


if __name__ == "__main__":
    main()
