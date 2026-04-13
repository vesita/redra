# 点云数据录制测试指南

## 问题背景

之前遇到的问题是：
- ❌ 测试工具发送原始二进制数据，服务器无法解析
- ❌ 错误信息："Command消息中没有定义任何命令"
- ❌ 帧列表没有任何数据

**根本原因**：Redra 系统使用 **Protobuf Command 协议 + Trailer 封装**进行通信，而之前的测试工具直接发送原始二进制数据，导致解析失败。

## 解决方案概述

已完成以下修改：

### 1. 扩展 Protobuf 协议
- ✅ 创建 [`point_cloud.proto`](file:///home/vesita/coding/my/redra/proto/point_cloud.proto) - 定义点云数据结构
- ✅ 修改 [`cmd.proto`](file:///home/vesita/coding/my/redra/proto/cmd.proto) - 添加 `point_cloud` 字段到 Command 消息

### 2. 更新解析器
- ✅ 在 [`proto.rs`](file:///home/vesita/coding/my/redra/src/module/parser/proto.rs) 中添加 `handle_point_cloud()` 函数
- ✅ 扩展 `process_pack()` 以处理 PointCloud 命令
- ✅ 将点云数据转换为 `RDPack::PointCloud` 并发送到记录器

### 3. 修复记录器
- ✅ 在 [`core.rs`](file:///home/vesita/coding/my/redra/src/module/parser/core.rs) 中添加 `PointCloudPack` 结构
- ✅ 在 [`record.rs`](file:///home/vesita/coding/my/redra/src/graph/data_processing/actions/record.rs) 中实现点云数据处理逻辑
- ✅ 支持自动帧边界检测和 SQLite 持久化

### 4. 创建正确的测试工具
- ✅ 更新 [`point_cloud_recording_test.rs`](file:///home/vesita/coding/my/redra/crates/redra_client/examples/point_cloud_recording_test.rs)
- ✅ 使用 Protobuf Command 编码
- ✅ 使用 Trailer 协议封装

## 数据流架构

```
┌─────────────────┐
│  Python/Rust    │
│   客户端         │
└────────┬────────┘
         │
         │ 1. 生成点云数据
         │ 2. 编码为 Protobuf Command
         │ 3. 使用 Trailer 封装
         ▼
┌─────────────────┐
│   TCP 连接       │
│  (8080端口)      │
└────────┬────────┘
         │
         │ 4. 接收原始字节
         │ 5. 解析 Trailer
         │ 6. 解码 Protobuf Command
         ▼
┌─────────────────┐
│  Parser Module  │
│  process_pack() │
└────────┬────────┘
         │
         │ 7. 识别 CmdPack::PointCloud
         │ 8. 转换为 RDPack::PointCloud
         ▼
┌─────────────────┐
│   RDChannel     │
│  (MPSC Channel) │
└────────┬────────┘
         │
         │ 9. 接收 RDPack
         ▼
┌─────────────────┐
│ DataRecorder    │
│ add_point_to_   │
│ frame()         │
└────────┬────────┘
         │
         │ 10. 组织帧数据
         │ 11. 持久化到 SQLite
         ▼
┌─────────────────┐
│  SQLite Database│
│  frames.db      │
└─────────────────┘
```

## 使用方法

### 步骤 1: 启动 Redra 服务器

```bash
cd /home/vesita/coding/my/redra
cargo run
```

等待看到以下日志：
```
✅ SQLite存储初始化成功
📊 数据库统计: X 帧, Y 总点数
```

### 步骤 2: 运行点云测试工具

在新的终端中：

```bash
cd /home/vesita/coding/my/redra
cargo run --example point_cloud_recording_test
```

你会看到：
```
=== 点云数据录制测试 ===

正在连接到服务器 127.0.0.1:8080...
✓ 连接成功

发送第 1 帧点云数据 (100 个点)...
  ✓ 第 1 帧发送完成
发送第 2 帧点云数据 (100 个点)...
  ✓ 第 2 帧发送完成
...
✅ 测试完成！共发送 5 帧点云数据
提示：在 UI 中查看帧列表，应该能看到 5 条记录
```

### 步骤 3: 验证录制结果

在 Redra UI 中：

1. **查看总帧数**：
   - "数据回放控制"窗口中的"总帧数"应该显示为 5（或更多）

2. **查看帧列表**：
   - 点击"📋 显示帧列表"按钮
   - 应该能看到 5 条记录，每条显示：
     - 帧 ID
     - 点数（100）
     - 时间戳
     - 帧类型

3. **检查 SQLite 数据库**（可选）：
   ```bash
   sqlite3 ~/.redra/frames/frames.db
   SELECT * FROM frames ORDER BY frame_id;
   ```

## 技术细节

### Protobuf 消息结构

**Point3D** (`point_cloud.proto`):
```protobuf
message Point3D {
  float x = 1;
  float y = 2;
  float z = 3;
}
```

**PointCloudPack** (`point_cloud.proto`):
```protobuf
message PointCloudPack {
  uint32 frame_id = 1;              // 帧ID
  uint64 timestamp = 2;             // 时间戳（毫秒）
  repeated Point3D points = 3;      // 点列表
  string source_id = 4;             // 数据源标识
  map<string, string> metadata = 5; // 元数据
}
```

**Command** (`cmd.proto`):
```protobuf
message Command {
  oneof cmd_pack {
    target.ConceptionCMD conception = 1;
    designation.DesignCMD designation = 2;
    transform.TransCMD transform = 3;
    pointcloud.PointCloudPack point_cloud = 6;  // 新增
  }
  int64 timestamp = 4;
  string command_id = 5;
}
```

### Trailer 协议

所有通过网络发送的数据都使用 Trailer 协议封装：

```
[trailer_size (4 bytes)] [trailer_data] [payload_data]
```

Trailer 结构：
```protobuf
message Trailer {
  uint32 me = 1;    // trailer 自身的大小
  uint32 next = 2;  // 后续 payload 的大小
}
```

### Rust 内部数据结构

**PointCloudPack** ([`core.rs`](file:///home/vesita/coding/my/redra/src/module/parser/core.rs)):
```rust
pub struct PointCloudPack {
    pub points: Vec<(f32, f32, f32)>,  // 点坐标列表
    pub frame_id: u32,                  // 帧ID
    pub timestamp: u64,                 // 时间戳
}
```

**RDPack** ([`core.rs`](file:///home/vesita/coding/my/redra/src/module/parser/core.rs)):
```rust
pub enum RDPack {
    Message(String),
    SpawnShape(Box<RDShapePack>),
    SpawnFormat(Box<FormatPack>),
    PointCloud(PointCloudPack),  // 新增
}
```

## 常见问题

### Q1: 为什么之前会显示 "Command消息中没有定义任何命令"？

**A**: 因为测试工具发送的是原始二进制数据，服务器尝试将其解码为 `command::Command`，但数据格式不匹配，导致 `cmd_pack` 字段为 `None`。

### Q2: 可以同时发送几何形状和点云数据吗？

**A**: 可以！系统会正确处理两种数据类型：
- `CmdPack::Designation` → 创建3D对象（SpawnShape）
- `CmdPack::PointCloud` → 录制点云数据（PointCloud）

### Q3: 如何从 Python 客户端发送点云数据？

**A**: 需要使用 `redra_proto` 的 Python 绑定。示例代码：

```python
import redra_proto.proto.cmd_pb2 as cmd_pb2
import redra_proto.proto.point_cloud_pb2 as pc_pb2
import struct

# 创建点云数据
pc_pack = pc_pb2.PointCloudPack()
pc_pack.frame_id = 1
pc_pack.timestamp = int(time.time() * 1000)

for x, y, z in points:
    point = pc_pack.points.add()
    point.x, point.y, point.z = x, y, z

# 创建 Command
command = cmd_pb2.Command()
command.point_cloud.CopyFrom(pc_pack)
command.command_id = "py_test_1"

# 编码并发送
data = command.SerializeToString()
# ... 使用 Trailer 协议发送
```

### Q4: 点云数据会被实时渲染吗？

**A**: 目前不会。点云数据仅被录制到 SQLite 数据库。如果需要实时渲染，需要在 `spawn.rs` 中添加相应的处理逻辑，将点云转换为可视化的3D对象。

### Q5: 如何调整每帧的点数？

**A**: 修改测试工具中的 `points_per_frame` 变量：

```rust
let points_per_frame = 100;  // 改为其他值，如 1000
```

## 下一步改进方向

1. **实时可视化**：在接收到点云时实时渲染到3D场景
2. **回放功能**：从 SQLite 加载历史帧并回放
3. **压缩优化**：对点云数据进行压缩以减少网络带宽
4. **LOD 支持**：根据距离动态调整点云密度
5. **多源支持**：同时处理多个传感器的点云数据流
6. **Python 客户端**：提供完整的 Python 示例代码

## 相关文件

- Protobuf 定义：
  - [`proto/point_cloud.proto`](file:///home/vesita/coding/my/redra/proto/point_cloud.proto)
  - [`proto/cmd.proto`](file:///home/vesita/coding/my/redra/proto/cmd.proto)

- Rust 实现：
  - [`src/module/parser/core.rs`](file:///home/vesita/coding/my/redra/src/module/parser/core.rs) - RDPack 定义
  - [`src/module/parser/proto.rs`](file:///home/vesita/coding/my/redra/src/module/parser/proto.rs) - 解析器
  - [`src/graph/data_processing/actions/record.rs`](file:///home/vesita/coding/my/redra/src/graph/data_processing/actions/record.rs) - 记录器

- 测试工具：
  - [`crates/redra_client/examples/point_cloud_recording_test.rs`](file:///home/vesita/coding/my/redra/crates/redra_client/examples/point_cloud_recording_test.rs)
