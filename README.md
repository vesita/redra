# Redra

Redra 是一个基于 Rust 的异步网络与图形交互系统，通过 Protobuf 进行通信，集成 Bevy 引擎实现可视化渲染。

## 项目概述

Redra 是一个网络通信与3D可视化系统，旨在通过网络通信实现实时图形渲染。

### 主要功能

- **网络通信**: 基于 Tokio 构建异步 TCP 通信，支持消息转发与连接管理
- **Protobuf 解析**: 使用 `prost` 自动生成 Protobuf 消息结构并编解码
- **几何建模**: 内置基础3D形状（点、线、立方体、球体）表示与变换处理
- **图形渲染**: 通过 Bevy 引擎支持3D场景构建与更新

## 依赖

### 必需依赖

- Rust (edition 2024)
- `protoc` (Protocol Buffers 编译器) - 请安装 `protobuf-compiler`
- Cargo

## 安装

### 开发环境搭建

1. 安装Rust
    推荐先配置rust镜像源，然后安装Rust
    1.1 配置rustup镜像源
        rustup源: <https://mirrors.tuna.tsinghua.edu.cn/help/rustup/>
    1.2 安装rust编译器
        ```bash
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        ```
    1.3 配置cargo源
        cargo源: <https://mirrors.tuna.tsinghua.edu.cn/help/crates.io-index/>
2. 安装Python
    本项目使用uv管理虚拟环境

    ```bash
    curl -LsSf https://astral.sh/uv/install.sh | sh
    ```

```bash
# 克隆项目
git clone https://github.com/vesita/redra
cd redra

# 构建项目
cargo build
```

### 模块说明

- `net`：网络通信模块，处理TCP连接和消息转发
- `graph`：图形渲染模块（可选），基于 Bevy 引擎
- `geometry`：几何建模模块，处理3D形状表示和变换
- `module`：核心功能模块，包括解析器、相机控制等
- `client`：客户端模块，处理网络消息的发送和接收

### 特性管理

项目通过 Cargo features 控制功能组合：

- `exec`：执行模块（默认启用）
- `graph`：图形渲染模块（可选）
- `client`：客户端功能（可选）

## 示例

要运行带有图形界面的完整版本：

```bash
cargo run
```

## 协议

Redra 使用 Protobuf 定义通信协议，支持多种数据类型和命令，包括：

- 形状数据（点、线、立方体、球体等）
- 格式数据（图像等）
- 变换命令（位置、旋转、缩放）
- 设计命令（创建实体等）

## 贡献

欢迎提交 PR 和 Issue 来改进 Redra！
