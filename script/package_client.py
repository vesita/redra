#!/usr/bin/env python3
"""
Redra Python客户端打包脚本
用于创建可分发的Python包
"""

import os
import sys
import shutil
import subprocess
from pathlib import Path


def clean_dist():
    """清理分发目录"""
    print("清理旧的分发文件...")
    for dir_name in ["build", "script/build", "redra_client.egg-info", "redra.egg-info"]:
        path = Path(dir_name)
        if path.exists():
            if path.is_dir():
                shutil.rmtree(path)
            else:
                path.unlink()
    print("清理完成")


def compile_protobuf():
    """编译protobuf文件"""
    print("清理并重新编译protobuf文件...")
    
    # 导入之前创建的proto编译脚本
    import proto_compile
    proto_compile.clean_proto_files()
    success = proto_compile.compile_proto_files()
    if success:
        proto_compile.create_init_file()
        print("Protobuf编译完成")
        return True
    else:
        print("Protobuf编译失败")
        return False


def build_package():
    """构建Python包"""
    print("构建Python包...")
    
    # 检查是否安装了build工具
    try:
        import build
    except ImportError:
        print("安装build工具...")
        try:
            # 尝试使用uv安装，如果不存在则使用pip
            subprocess.run([sys.executable, "-m", "uv", "pip", "install", "build"], check=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            print("uv未找到，使用pip安装build工具...")
            subprocess.run([sys.executable, "-m", "pip", "install", "build"], check=True)
    
    try:
        # 使用python -m build命令构建，指定输出目录
        result = subprocess.run([
            sys.executable, "-m", "build", "--outdir", "script/build"
        ], check=True, capture_output=True, text=True)
        
        if result.stdout:
            print(result.stdout)
        if result.stderr:
            print(result.stderr)
            
        print("构建成功")
        return True
    except subprocess.CalledProcessError as e:
        print(f"构建失败: {e}")
        print(f"错误输出: {e.stderr}")
        return False


def install_package():
    """安装包到本地环境"""
    print("安装包到本地环境...")
    
    try:
        # 先尝试使用uv安装
        try:
            subprocess.run([sys.executable, "-m", "uv", "pip", "install", "--upgrade", "pip", "wheel"], check=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            print("uv未找到，使用pip...")
            subprocess.run([sys.executable, "-m", "pip", "install", "--upgrade", "pip", "wheel"], check=True)
        
        # 尝试使用uv安装打包的wheel文件
        try:
            result = subprocess.run([
                sys.executable, "-m", "uv", "pip", "install", "script/build/*.whl", "--force-reinstall"
            ], check=True, capture_output=True, text=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            print("uv未找到或安装失败，使用pip...")
            result = subprocess.run([
                sys.executable, "-m", "pip", "install", "script/build/*.whl", "--force-reinstall"
            ], check=True, capture_output=True, text=True)
        
        if result.stdout:
            print(result.stdout)
        if result.stderr:
            print(result.stderr)
            
        print("安装成功")
        return True
    except subprocess.CalledProcessError as e:
        print(f"安装失败: {e}")
        print(f"错误输出: {e.stderr}")
        return False


def show_dist_files():
    """显示生成的分发文件"""
    dist_path = Path("script/build")
    if dist_path.exists():
        print("\n生成的分发文件:")
        for file in dist_path.iterdir():
            if file.is_file():
                print(f"  - {file}")
    else:
        print("\n未找到分发目录")


def main():
    """主函数"""
    print("Redra Python客户端打包工具")
    print("="*40)
    
    # 切换到项目根目录
    project_root = Path(__file__).parent.parent
    os.chdir(project_root)
    print(f"已切换到项目根目录: {os.getcwd()}")
    
    # 执行打包流程
    clean_dist()
    
    if not compile_protobuf():
        print("终止: Protobuf编译失败")
        return 1
    
    if not build_package():
        print("终止: 包构建失败")
        return 1
    
    show_dist_files()
    
    response = input("\n是否安装到本地环境? (y/n): ")
    if response.lower() in ['y', 'yes']:
        install_package()
    
    print("\n打包完成！")
    print("生成的包文件位于 script/build/ 目录中")
    print("可以使用 'uv pip install script/build/*.whl' 安装到其他项目")
    print("或者上传到PyPI进行分发")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())