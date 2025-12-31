#!/usr/bin/env python3
"""
Proto文件编译脚本

此脚本将proto目录下的所有.proto文件编译到packs/rdsend/proto目录下
使用protoc编译器生成Python代码
"""

import os
import subprocess
import sys
from pathlib import Path


def compile_proto_files():
    """编译proto文件到Python代码"""
    # 获取脚本所在目录的父目录（项目根目录）
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent  # 从script目录返回到项目根目录
    
    proto_dir = project_root / "proto"
    output_dir = project_root / "packs" / "rdsend" / "proto"
    
    # 确保输出目录存在
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # 检查proto目录是否存在
    if not proto_dir.exists():
        print(f"错误: proto目录不存在: {proto_dir}")
        return False
    
    # 获取所有.proto文件
    proto_files = list(proto_dir.glob("*.proto"))
    
    if not proto_files:
        print(f"错误: 在{proto_dir}中没有找到.proto文件")
        return False
    
    print(f"找到 {len(proto_files)} 个.proto文件:")
    for proto_file in proto_files:
        print(f"  - {proto_file.name}")
    
    # 准备protoc命令
    protoc_cmd = [
        "protoc",
        f"--proto_path={proto_dir}",
        f"--python_out={output_dir}"
    ] + [str(proto_file) for proto_file in proto_files]
    
    try:
        print("\n正在编译proto文件...")
        result = subprocess.run(
            protoc_cmd,
            check=True,
            capture_output=True,
            text=True
        )
        
        if result.stdout:
            print(f"标准输出: {result.stdout}")
        if result.stderr:
            print(f"错误输出: {result.stderr}")
            
        print(f"\n成功编译proto文件到: {output_dir}")
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"编译失败: {e}")
        print(f"错误输出: {e.stderr}")
        return False
    except FileNotFoundError:
        print("错误: 未找到protoc编译器，请确保已安装protobuf-compiler")
        return False


def main():
    """主函数"""
    print("开始编译proto文件...")
    success = compile_proto_files()
    
    if success:
        print("\nproto文件编译成功!")
        return 0
    else:
        print("\nproto文件编译失败!")
        return 1


if __name__ == "__main__":
    sys.exit(main())