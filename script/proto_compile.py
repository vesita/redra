#!/usr/bin/env python3
"""
Protobuf编译脚本
用于清理旧的Python protobuf文件并重新编译它们
"""

import os
import subprocess
import glob
import sys
from pathlib import Path


def clean_proto_files():
    """清理旧的protobuf Python文件"""
    print("开始清理旧的protobuf文件...")
    
    # 查找所有_pb2.py文件
    pb2_files = glob.glob("script/proto/*_pb2.py")
    pb2_files.extend(glob.glob("script/proto/*_pb2.pyi"))  # 也包括类型提示文件
    
    for file in pb2_files:
        try:
            os.remove(file)
            print(f"已删除: {file}")
        except OSError as e:
            print(f"删除文件失败 {file}: {e}")
    
    # 清理__pycache__目录
    cache_dirs = glob.glob("script/proto/__pycache__/")
    cache_dirs.extend(glob.glob("script/proto/**/__pycache__/", recursive=True))
    
    for cache_dir in cache_dirs:
        try:
            import shutil
            shutil.rmtree(cache_dir)
            print(f"已删除缓存目录: {cache_dir}")
        except OSError as e:
            print(f"删除缓存目录失败 {cache_dir}: {e}")
    
    print("清理完成")


def compile_proto_files():
    """编译proto文件为Python代码"""
    print("开始编译protobuf文件...")
    
    # 确保目标目录存在
    proto_dir = Path("proto")
    output_dir = Path("script/proto")
    output_dir.mkdir(exist_ok=True)
    
    # 获取所有proto文件
    proto_files = list(proto_dir.glob("*.proto"))
    
    if not proto_files:
        print("在proto目录中未找到.proto文件")
        return False
    
    print(f"找到 {len(proto_files)} 个proto文件")
    
    for proto_file in proto_files:
        print(f"编译 {proto_file}")
        
        # 构建编译命令
        cmd = [
            "protoc",
            f"--proto_path={proto_dir}",
            f"--python_out={output_dir}",
            str(proto_file)
        ]
        
        try:
            result = subprocess.run(cmd, check=True, capture_output=True, text=True)
            if result.stdout:
                print(result.stdout)
            if result.stderr:
                print(result.stderr)
        except subprocess.CalledProcessError as e:
            print(f"编译失败 {proto_file}: {e}")
            print(f"错误输出: {e.stderr}")
            return False
        except FileNotFoundError:
            print("错误: 未找到protoc编译器，请确保已安装Protocol Buffers编译器")
            return False
    
    print("编译完成")
    return True


def create_init_file():
    """在proto目录中创建__init__.py文件"""
    init_file = Path("script/proto/__init__.py")
    if not init_file.exists():
        init_file.write_text('"""\nProto模块包\n"""\n')
        print(f"已创建 {init_file}")


def main():
    """主函数"""
    print("Protobuf清理和编译脚本")
    print("="*40)
    
    # 切换到项目根目录
    project_root = Path(__file__).parent.parent
    os.chdir(project_root)
    print(f"已切换到项目根目录: {os.getcwd()}")
    
    # 执行清理
    clean_proto_files()
    
    # 执行编译
    success = compile_proto_files()
    
    if success:
        # 创建__init__.py文件
        create_init_file()
        print("\n所有protobuf文件已成功清理和重新编译！")
        return 0
    else:
        print("\n编译过程中出现错误！")
        return 1


if __name__ == "__main__":
    # 只有当脚本被直接运行时才执行主逻辑
    sys.exit(main())

