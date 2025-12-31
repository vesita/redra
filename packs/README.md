# Redra Python 客户端

Redra Python 客户端是一个用于与 Redra 系统进行通信的 Python 库。
Redra 是一个基于 Rust 的异步网络与图形交互系统，支持通过 Protobuf 进行通信，并集成 Bevy 引擎实现可视化渲染。

## 项目概述

Redra 项目由两部分组成：

- **Rust 后端**: 提供高性能的异步网络通信和可选的图形渲染功能
- **Python 客户端 (rdsend)**: 提供与 Redra 系统交互的 Python 接口

## 功能特性

- **协议支持**: 基于 Protobuf 的高效二进制协议通信
- **客户端功能**: 支持从 Python 发送命令到 Redra 系统
- **数据类型支持**: 支持多种几何形状和变换数据的发送

## 安装要求

要使用 Redra Python 客户端，你需要：

1. **Python 版本**: Python 3.12 或更高版本
2. **Protocol Buffers 编译器**: 系统级 `protoc` 安装
3. **Python 依赖包**:
   - `protobuf` 库

### 安装依赖

```bash
# 安装 Protocol Buffers 编译器 (Ubuntu/Debian)
sudo apt install protobuf-compiler

# 或在 macOS 上使用 Homebrew
brew install protobuf

# 安装 Python 依赖
pip install protobuf
```

## 安装

你可以通过以下方式安装 Redra Python 客户端:

```bash
# 构建并安装包
cd packs
python -m build
pip install .
```

## 使用方法

```python
from rdsend.client import ClientSender

client = ClientSender()

# 示例：发送一个简单的命令
client.send_point(5.0, 5.0, 5.0)

# 更多使用示例请参考 example.py
```

更多使用示例请参考 [example.py](./example.py) 文件。

## 目录结构

```
packs/
├── rdsend/              # Python 客户端主模块
│   ├── __init__.py      # 模块初始化文件
│   ├── client.py        # 客户端实现
│   ├── proto/           # Protobuf 生成的 Python 文件
│   │   ├── __init__.py
│   │   └── *.py         # 各种协议定义对应的 Python 文件
│   └── example.py       # 使用示例
├── pyproject.toml       # Python 项目配置文件
├── README.md            # 本文件
└── build.rs             # 构建脚本
```

## 构建

要构建 Python 包，请运行:

```bash
cd packs
python -m build
```

这将生成可用于安装的 wheel 和源码分发包。

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](../LICENSE) 文件。
