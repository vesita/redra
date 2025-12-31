#!/usr/bin/env python3
"""
构建脚本组合

此脚本将依次执行proto编译和packs构建两个任务
"""

import sys
import subprocess
from pathlib import Path


def run_proto_compile():
    """运行proto编译"""
    print("=" * 60)
    print("开始编译proto文件...")
    print("=" * 60)
    
    # 获取脚本所在目录的父目录（项目根目录），然后定位到script目录
    script_dir = Path(__file__).resolve().parent
    compile_script = script_dir / "compile_proto.py"
    
    if not compile_script.exists():
        print(f"错误: 编译脚本不存在: {compile_script}")
        return False
    
    try:
        result = subprocess.run(
            [sys.executable, str(compile_script)],
            check=True,
            capture_output=True,
            text=True
        )
        
        print(result.stdout)
        if result.stderr:
            print(f"错误输出: {result.stderr}")
        
        return True
    except subprocess.CalledProcessError as e:
        print(f"proto编译失败: {e}")
        print(f"错误输出: {e.stderr}")
        return False


def run_packs_build():
    """运行packs构建"""
    print("\n" + "=" * 60)
    print("开始构建packs...")
    print("=" * 60)
    
    # 获取脚本所在目录的父目录（项目根目录），然后定位到script目录
    script_dir = Path(__file__).resolve().parent
    build_script = script_dir / "build_packs.py"
    
    if not build_script.exists():
        print(f"错误: 构建脚本不存在: {build_script}")
        return False
    
    try:
        result = subprocess.run(
            [sys.executable, str(build_script)],
            check=True,
            capture_output=True,
            text=True
        )
        
        print(result.stdout)
        if result.stderr:
            print(f"错误输出: {result.stderr}")
        
        return True
    except subprocess.CalledProcessError as e:
        print(f"packs构建失败: {e}")
        print(f"错误输出: {e.stderr}")
        return False


def main():
    """主函数，执行所有构建步骤"""
    print("开始执行构建流程...")
    
    # 运行proto编译
    success = run_proto_compile()
    if not success:
        print("proto编译失败，终止构建流程")
        return 1
    
    # 运行packs构建
    success = run_packs_build()
    if not success:
        print("packs构建失败")
        return 1
    
    print("\n" + "=" * 60)
    print("所有构建步骤完成!")
    print("=" * 60)
    return 0


if __name__ == "__main__":
    sys.exit(main())