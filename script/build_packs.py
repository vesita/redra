#!/usr/bin/env python3
"""
Packs构建脚本

此脚本将进入packs目录并运行python -m build命令
"""

import os
import subprocess
import sys
from pathlib import Path


def build_packs():
    """在packs目录下执行构建命令"""
    # 获取脚本所在目录的父目录（项目根目录）
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent  # 从script目录返回到项目根目录
    packs_dir = project_root / "packs"
    
    # 检查packs目录是否存在
    if not packs_dir.exists():
        print(f"错误: packs目录不存在: {packs_dir}")
        return False
    
    # 检查pyproject.toml是否存在，以确认是否可以构建
    pyproject_file = packs_dir / "pyproject.toml"
    if not pyproject_file.exists():
        print(f"警告: {pyproject_file}不存在，但仍尝试构建")
    
    # 准备构建命令
    build_cmd = [
        sys.executable,  # 使用当前Python解释器
        "-m", "build"
    ]
    
    try:
        print(f"正在进入 {packs_dir} 目录并执行构建...")
        result = subprocess.run(
            build_cmd,
            cwd=packs_dir,  # 在packs目录下执行
            check=True,
            capture_output=True,
            text=True
        )
        
        if result.stdout:
            print(f"标准输出:\n{result.stdout}")
        if result.stderr:
            print(f"错误输出:\n{result.stderr}")
            
        print(f"\n成功构建包!")
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"构建失败: {e}")
        print(f"错误输出: {e.stderr}")
        return False
    except FileNotFoundError:
        print("错误: 未找到build模块，请确保已安装build包 (pip install build)")
        return False


def main():
    """主函数"""
    print("开始构建packs...")
    success = build_packs()
    
    if success:
        print("\npacks构建成功!")
        return 0
    else:
        print("\npacks构建失败!")
        return 1


if __name__ == "__main__":
    sys.exit(main())