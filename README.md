# Redra

Redra 是一个基于 Rust 的异步网络与图形交互系统，通过 Protobuf 进行通信，集成 Bevy 引擎实现可视化渲染。

## 项目概述

Redra 是一个网络通信与3D可视化系统，旨在通过网络通信实现实时图形渲染。

### 主要功能

- **网络通信**: 基于 Tokio 构建异步 TCP 通信，支持消息转发与连接管理
- **Protobuf 解析**: 使用 `prost` 自动生成 Protobuf 消息结构并编解码
- **几何建模**: 内置基础3D形状（点、线、立方体、球体）表示与变换处理
- **图形渲染**: 通过 Bevy 引擎支持3D场景构建与更新
- **材质系统**: 声明式材质注册器，支持基础色、聚类色板（12色）、语义色、效果材质分类管理
- **坐标系配置**: 支持左手系/右手系切换、六轴方向切换、坐标轴显示/隐藏
- **实体标签**: 支持悬浮标签显示与手动编辑实体 Tag
- **文件管理**: 帧数据保存/加载（.rdra）、PCD 点云文件加载

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
cargo build --release
# 或者

# windows
rustup target add x86_64-pc-windows-gnu
cargo build --target x86_64-pc-windows-gnu --release

# linux
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu --release
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

### 运行主程序

要运行带有图形界面的完整版本：

```bash
cargo run
```

### 测试示例

项目提供了多个测试示例来验证不同功能：

#### 数据管理功能测试

发送多帧3D对象数据，用于测试帧回放UI：

```bash
cargo run --example redra_test --package redra_client
```

此示例会发送5帧数据，包含对象的创建、移动、新增和删除操作。

#### 标签功能测试

发送带标签的3D对象数据，用于测试标签显示功能：

```bash
cargo run --example label_test --package redra_client
```

此示例会发送5帧数据，测试以下标签功能：
- 基础标签显示（默认样式）
- 自定义位置偏移
- 自定义样式（颜色、字体大小、圆角）
- 多个标签同时显示
- 复杂混合配置

运行测试示例前，请确保服务器正在运行。

## UI 控制说明

Redra 提供了直观的可视化界面来控制帧回放和场景交互。

### 帧回放控制面板

启动程序后，左上角会显示"📊 帧回放控制"面板，提供以下功能：

#### 播放控制
- **⏮ / ⏭** - 跳转到首帧/尾帧
- **◀ / ▶** - 上一帧/下一帧（逐帧浏览）
- **▶ / ⏸** - 播放/暂停（自动按设定速度推进）

#### 速度调节
- **预设档位**: 10/30/60/120 FPS 快速切换
- **自定义滑块**: 支持 1-240 FPS 无级调节

#### 帧跳转
- **时间轴滑块**: 拖动即可快速定位到任意帧
- **实时显示**: 当前帧索引 / 总帧数（如: 5/100）

#### 键盘快捷键
| 按键 | 功能 |
|------|------|
| `空格` | 播放/暂停切换 |
| `左箭头` / `右箭头` | 上一帧/下一帧 |
| `Home` / `End` | 跳转到首帧/尾帧 |
| `Alt` | 显示/隐藏所有 UI（同时锁定/释放鼠标） |
| `Tab` | 打开/关闭轮盘菜单 |

### 轮盘菜单

按 `Tab` 键可呼出径向菜单，支持：
- WASD 或方向键导航
- Enter/Space 确认选择
- 鼠标悬停高亮

### 坐标系面板

点击左侧活动栏的 "↕" 图标打开坐标系面板：

- **显示/隐藏坐标轴**: 切换场景中 X/Y/Z 坐标轴的可见性
- **手性切换**: 左手系 / 右手系
- **向上轴选择**: +X / -X / +Y / -Y / +Z / -Z 六个方向

### 实体标签编辑

选中实体后，悬浮标签会显示实体 ID 和 Tag 文本：

- 点击铅笔图标进入编辑模式
- 输入新文本后按 `Enter` 或点击 "确认" 保存
- 按 `Escape` 或点击 "取消" 丢弃修改

### UI 显示/隐藏

- 按 `Alt` 键可切换 UI 可见性
- UI 隐藏时，鼠标会被锁定并隐藏（FPS 相机模式）
- UI 显示时，鼠标自由移动，可进行界面交互

## TCMP协议 (Trailer-Command Messaging Protocol)

Redra 使用TCMP协议进行通信，该协议基于Protobuf定义消息结构，使用Trailer协议定义数据包边界。

### 协议结构

```
[Trailer][Command]
```

**Trailer结构**:
```protobuf
message Trailer {
  uint32 me = 1;    // trailer自身的大小
  uint32 next = 2;  // 后续payload的大小
}
```

**Command结构**:
```protobuf
message Command {
  oneof cmd_pack {
    target.ConceptionCMD conception = 1;
    designation.DesignCMD designation = 2;
    transform.TransCMD transform = 3;
    resource.RsrcPack resource = 4;
    shape.ShapePack shape = 5;
    pointcloud.PointCloudPack point_cloud = 6;
    MacroCmd macro = 7;  // 新增宏指令
  }
  int64 timestamp = 4;
  string command_id = 5;
}
```

**宏指令支持**:
- `ConnectionControl`: 用于连接管理
- `Heartbeat`: 用于维持连接活跃状态
- `BatchCmd`: 用于批量命令处理
- `MetaCmd`: 用于元数据传输

### 使用示例

```rust
use redra_proto::coding::encoding::{encode_command_with_trailer, create_heartbeat_command};

// 创建心跳命令
let heartbeat_cmd = create_heartbeat_command("session_123");

// 编码为TCMP格式（Trailer + Command）
let packet = encode_command_with_trailer(&heartbeat_cmd).unwrap();

// 发送数据...
```

## 贡献

欢迎提交 PR 和 Issue 来改进 Redra！