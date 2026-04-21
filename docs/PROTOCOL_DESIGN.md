# Redra 通信协议系统设计

## 协议架构概述

Redra 使用分层协议设计，确保数据传输的可靠性和可扩展性：

```
┌─────────────────────────────────────┐
│   Application Layer (应用层)         │
│   - PointCloud Data                 │
│   - Shape Commands                  │
│   - Transform Commands              │
└──────────────┬──────────────────────┘
               │
               │ Protobuf Encoding
               ▼
┌─────────────────────────────────────┐
│   Protocol Layer (协议层)            │
│   Command Message                   │
│   ┌──────────────────────────────┐  │
│   │ oneof cmd_pack:              │  │
│   │   - ConceptionCMD            │  │
│   │   - DesignCMD                │  │
│   │   - TransCMD                 │  │
│   │   - PointCloudPack           │  │
│   └──────────────────────────────┘  │
└──────────────┬──────────────────────┘
               │
               │ Binary Serialization
               ▼
┌─────────────────────────────────────┐
│   Transport Layer (传输层)           │
│   Trailer Protocol                  │
│   ┌──────────┬──────────┐          │
│   │ Trailer  │ Payload  │          │
│   │ (size+N) │ (data)   │          │
│   └──────────┴──────────┘          │
└──────────────┬──────────────────────┘
               │
               │ TCP Stream
               ▼
┌─────────────────────────────────────┐
│   Network Layer (网络层)             │
│   TCP Connection                    │
└─────────────────────────────────────┘
```

## 核心设计原则

### 1. **One Trailer + One Pack**

每个数据包都遵循以下格式：

```
[Trailer] [Payload]
```

- **Trailer**: 元数据，描述 payload 的大小和类型
- **Payload**: 实际的 Protobuf 编码数据

这种设计的好处：
- ✅ **自描述**: 接收方知道接下来要读取多少字节
- ✅ **流式处理**: 可以连续发送多个包，接收方能正确分割
- ✅ **错误恢复**: 如果某个包损坏，可以从下一个 trailer 重新开始

### 2. **Protobuf Command 统一封装**

所有应用层数据都被封装在 `Command` 消息中：

```protobuf
message Command {
  oneof cmd_pack {
    target.ConceptionCMD conception = 1;     // 概念命令
    designation.DesignCMD designation = 2;   // 设计命令（几何形状）
    transform.TransCMD transform = 3;        // 变换命令
    pointcloud.PointCloudPack point_cloud = 6; // 点云数据
  }
  int64 timestamp = 4;      // 时间戳
  string command_id = 5;    // 命令ID（用于追踪）
}
```

这种设计的优势：
- ✅ **类型安全**: 通过 oneof 确保只有一个命令类型
- ✅ **可扩展**: 轻松添加新的命令类型
- ✅ **向后兼容**: 旧版本客户端可以忽略未知字段

### 3. **Trailer 协议详解**

Trailer 结构定义：

```protobuf
message Trailer {
  uint32 me = 1;    // Trailer 自身的大小（字节）
  uint32 next = 2;  // 后续 Payload 的大小（字节）
}
```

**编码流程**：

```rust
// 1. 创建临时 trailer 计算大小
let temp_trailer = Trailer { me: 1, next: payload_len };
let trailer_size = temp_trailer.encoded_len() as u32;

// 2. 创建最终 trailer
let trailer = Trailer {
    me: trailer_size,
    next: payload_len,
};

// 3. 编码并发送
let mut trailer_buf = Vec::new();
trailer.encode(&mut trailer_buf)?;
stream.write_all(&trailer_buf)?;
stream.write_all(&payload)?;
```

**解码流程**：

```rust
// 1. 读取 trailer 大小（4字节）
let mut size_buf = [0u8; 4];
stream.read_exact(&mut size_buf)?;
let trailer_size = u32::from_le_bytes(size_buf);

// 2. 读取 trailer
let mut trailer_buf = vec![0u8; trailer_size as usize];
stream.read_exact(&mut trailer_buf)?;
let trailer = Trailer::decode(&mut &trailer_buf[..])?;

// 3. 读取 payload
let mut payload_buf = vec![0u8; trailer.next as usize];
stream.read_exact(&mut payload_buf)?;

// 4. 解码 payload
let command = Command::decode(&mut &payload_buf[..])?;
```

## 数据类型详细说明

### 1. 点云数据 (PointCloudPack)

**Protobuf 定义** (`proto/point_cloud.proto`):

```protobuf
message Point3D {
  float x = 1;
  float y = 2;
  float z = 3;
}

message PointCloudPack {
  uint32 frame_id = 1;              // 帧ID
  uint64 timestamp = 2;             // 时间戳（毫秒）
  repeated Point3D points = 3;      // 点列表
  string source_id = 4;             // 数据源标识
  map<string, string> metadata = 5; // 元数据
}
```

**Rust 内部表示** (`src/module/parser/core.rs`):

```rust
pub struct PointCloudPack {
    pub points: Vec<(f32, f32, f32)>,
    pub frame_id: u32,
    pub timestamp: u64,
}

pub enum RDPack {
    Message(String),
    SpawnShape(Box<RDShapePack>),
    SpawnFormat(Box<FormatPack>),
    PointCloud(PointCloudPack),  // 点云数据
}
```

**使用示例** (Rust):

```rust
use redra_proto::proto::{
    command::{self, Command},
    pointcloud::{PointCloudPack, Point3D},
};

// 创建点云数据
let pc_pack = PointCloudPack {
    frame_id: 1,
    timestamp: 1234567890,
    points: vec![
        Point3D { x: 1.0, y: 2.0, z: 3.0 },
        Point3D { x: 4.0, y: 5.0, z: 6.0 },
    ],
    source_id: "lidar_01".to_string(),
    metadata: HashMap::new(),
};

// 封装为 Command
let cmd = Command {
    cmd_pack: Some(command::command::CmdPack::PointCloud(pc_pack)),
    timestamp: 1234567890,
    command_id: "pc_001".to_string(),
};

// 编码并发送
let data = cmd.encode_to_vec();
send_with_trailer(&stream, &data).await?;
```

### 2. 几何形状命令 (DesignCMD)

**用途**: 在场景中创建、修改或删除3D对象

**示例** (使用 redra_client):

```rust
use redra_client::{send_point, send_cube};

// 发送一个点
send_point(0.0, 0.0, 0.0).await?;

// 发送一个立方体
send_cube(0.0, 0.0, 0.0, 1.0, 1.0, 1.0).await?;
```

**底层实现**:

```rust
// 1. 创建 ShapePack
let shape = ShapePack {
    shape_type: Some(shape::shape_pack::ShapeType::Point(
        shape::Point { /* ... */ }
    )),
    material: "blue".to_string(),
    // ...
};

// 2. 创建 DesignCMD
let design_cmd = DesignCMD {
    cmd: Some(designation::design_cmd::Cmd::Spawn(
        designation::SpawnCMD {
            shapes: vec![shape],
            // ...
        }
    )),
};

// 3. 封装为 Command
let cmd = Command {
    cmd_pack: Some(command::command::CmdPack::Designation(design_cmd)),
    // ...
};
```

### 3. 变换命令 (TransCMD)

**用途**: 修改现有对象的位置、旋转、缩放

**示例**:

```rust
// TODO: 提供便捷的 API
```

## 数据流完整示例

### 场景：发送一帧点云数据

```
客户端                              服务器
  |                                   |
  |  1. 生成点云数据                   |
  |     [(x,y,z), ...]                |
  |                                   |
  |  2. 构建 PointCloudPack           |
  |     protobuf encoding             |
  |                                   |
  |  3. 封装为 Command                |
  |     Command {                     |
  |       point_cloud: pc_pack,       |
  |       timestamp: ...,             |
  |       command_id: "..."           |
  |     }                             |
  |                                   |
  |  4. Protobuf 编码                 |
  |     binary_data = cmd.encode()    |
  |                                   |
  |  5. Trailer 封装                  |
  |     [trailer][binary_data]        |
  |                                   |
  |-------- TCP Send ---------------->|
  |                                   |  6. 读取 Trailer
  |                                   |     parse size
  |                                   |
  |                                   |  7. 读取 Payload
  |                                   |     read N bytes
  |                                   |
  |                                   |  8. Protobuf 解码
  |                                   |     Command::decode()
  |                                   |
  |                                   |  9. 匹配 cmd_pack
  |                                   |     CmdPack::PointCloud
  |                                   |
  |                                   | 10. 转换为 RDPack
  |                                   |     RDPack::PointCloud
  |                                   |
  |                                   | 11. 发送到 Channel
  |                                   |     channel.send(rd_pack)
  |                                   |
  |                                   | 12. 记录器接收
  |                                   |     add_point_to_frame()
  |                                   |
  |                                   | 13. 组织帧数据
  |                                   |     FrameBuilder
  |                                   |
  |                                   | 14. 持久化到 SQLite
  |                                   |     save_frame()
  |                                   |
  |                                   | 15. UI 更新
  |                                   |     显示帧列表
```

## 常见问题

### Q1: 为什么需要 Trailer 协议？

**A**: Trailer 协议解决了 TCP 流的**粘包问题**。TCP 是字节流协议，不保证消息边界。Trailer 提供了明确的帧定界：

```
Without Trailer:
[data1][data2][data3]  ← 接收方不知道如何分割

With Trailer:
[size1][data1][size2][data2][size3][data3]  ← 清晰的消息边界
```

### Q2: 为什么不直接使用 WebSocket 或其他协议？

**A**: 
- **性能**: TCP + Protobuf 比 WebSocket 更高效
- **控制**: 完全控制序列化/反序列化过程
- **简单**: 不需要额外的协议栈
- **灵活**: 可以轻松切换到 QUIC 或其他传输层

### Q3: 如何处理大数据量（如百万级点云）？

**A**: 
1. **分块传输**: 将大帧拆分为多个小包
2. **压缩**: 使用 gzip 或 zstd 压缩 payload
3. **流式处理**: 边接收边处理，不等待完整帧
4. **LOD**: 根据距离动态调整点密度

### Q4: 如何添加新的数据类型？

**A**: 三步走：

1. **定义 Protobuf 消息** (`proto/new_type.proto`):
   ```protobuf
   message NewTypePack {
     // 你的数据结构
   }
   ```

2. **添加到 Command** (`proto/cmd.proto`):
   ```protobuf
   message Command {
     oneof cmd_pack {
       // ... existing fields ...
       newtype.NewTypePack new_type = 7;
     }
   }
   ```

3. **实现 Rust 处理逻辑** (`src/module/parser/proto.rs`):
   ```rust
   Some(command::command::CmdPack::NewType(ref data)) => {
       handle_new_type(data, sender);
   }
   ```

### Q5: 如何调试协议问题？

**A**: 

1. **启用详细日志**:
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **检查 Trailer 解析**:
   ```rust
   debug!("Trailer: me={}, next={}", trailer.me, trailer.next);
   ```

3. **验证 Protobuf 解码**:
   ```rust
   match Command::decode(&mut &payload[..]) {
       Ok(cmd) => debug!("Command decoded: {:?}", cmd),
       Err(e) => error!("Decode failed: {}", e),
   }
   ```

4. **使用 Wireshark/tcpdump**:
   ```bash
   sudo tcpdump -i lo port 8080 -X
   ```

## 性能优化建议

### 1. 批量发送

```rust
// ❌ 低效：逐个发送
for point in points {
    send_point(point).await?;
}

// ✅ 高效：批量发送
let pc_pack = PointCloudPack {
    points: all_points,
    // ...
};
send_point_cloud(pc_pack).await?;
```

### 2. 连接复用

```rust
// ❌ 每次创建新连接
for _ in 0..100 {
    let client = Client::connect(addr).await?;
    client.send(data).await?;
}

// ✅ 复用连接
let client = Client::connect(addr).await?;
for _ in 0..100 {
    client.send(data).await?;
}
```

### 3. 异步并发

```rust
// 并行处理多个数据流
let tasks: Vec<_> = sensors.iter().map(|sensor| {
    tokio::spawn(process_sensor(sensor.clone()))
}).collect();

for task in tasks {
    task.await??;
}
```

## 相关资源

- **Protobuf 文档**: https://developers.google.com/protocol-buffers
- **Tokio 异步编程**: https://tokio.rs
- **Bevy 引擎**: https://bevyengine.org

## 文件索引

### Protobuf 定义
- [`proto/cmd.proto`](file:///home/vesita/coding/my/redra/proto/cmd.proto) - Command 消息定义
- [`proto/point_cloud.proto`](file:///home/vesita/coding/my/redra/proto/point_cloud.proto) - 点云数据定义
- [`proto/declare.proto`](file:///home/vesita/coding/my/redra/proto/declare.proto) - Trailer 协议定义
- [`proto/designation.proto`](file:///home/vesita/coding/my/redra/proto/designation.proto) - 几何形状命令
- [`proto/transform.proto`](file:///home/vesita/coding/my/redra/proto/transform.proto) - 变换命令

### Rust 实现
- [`crates/redra_proto/src/proto.rs`](file:///home/vesita/coding/my/redra/crates/redra_proto/src/proto.rs) - Proto 模块导出
- [`src/module/parser/core.rs`](file:///home/vesita/coding/my/redra/src/module/parser/core.rs) - RDPack 内部表示
- [`src/module/parser/proto.rs`](file:///home/vesita/coding/my/redra/src/module/parser/proto.rs) - 解析器实现
- [`src/graph/data_processing/actions/record.rs`](file:///home/vesita/coding/my/redra/src/graph/data_processing/actions/record.rs) - 数据记录器

### 客户端库
- [`crates/redra_client/src/client/sender.rs`](file:///home/vesita/coding/my/redra/crates/redra_client/src/client/sender.rs) - 发送器实现
- [`crates/redra_client/examples/point_cloud_recording_test.rs`](file:///home/vesita/coding/my/redra/crates/redra_client/examples/point_cloud_recording_test.rs) - 点云测试示例
- [`crates/redra_client/examples/simple_test.rs`](file:///home/vesita/coding/my/redra/crates/redra_client/examples/simple_test.rs) - 简单测试示例
