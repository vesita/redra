# Parser Crate

协议解析核心逻辑库，提供纯粹的协议解析功能，**不包含任何 Bevy 依赖**。

## 架构设计

```
expto (协议层) → parser (纯逻辑解析) → inpto (内存层)
                      ↓
              redra/src/manager (Bevy 集成层)
```

- **expto**: 定义网络传输的 Protobuf 消息结构
- **parser**: 纯逻辑库，提供无状态的解析工具函数
- **inpto**: 提供内存层面的标准数据结构
- **redra/manager**: Bevy 集成层，负责将解析结果应用到 ECS

## 核心特性

### 1. 纯逻辑库

本 crate **没有任何 Bevy 依赖**，可以在任何 Rust 项目中使用，包括：
- 无头服务器
- 单元测试
- 其他渲染引擎
- 命令行工具

### 2. 清晰的职责分离

- **parser crate**: 只负责"如何解析协议"
- **redra manager**: 负责"如何使用解析结果"

### 3. 易于测试

所有解析逻辑都是纯函数，无需模拟 Bevy 环境即可测试。

## 公共 API

### ProtocolParser - 协议解析器

```rust
use parser::{ProtocolParser, ParsedCommand};
use expto::rdmp::Unit;

// 解析单个 Unit
let unit: Unit = /* ... */;
let cmd = ProtocolParser::parse_unit(&unit);

match cmd {
    ParsedCommand::Spawn { id, transform } => {
        println!("Spawn entity {} at {:?}", id, transform);
    },
    ParsedCommand::Update { id, transform } => {
        println!("Update entity {}", id);
    },
    ParsedCommand::Destroy { id } => {
        println!("Destroy entity {}", id);
    },
    ParsedCommand::Unknown => {
        println!("Unknown command");
    }
}

// 批量解析
let units: Vec<Unit> = /* ... */;
let commands = ProtocolParser::parse_units(&units);
```

### FrameAssembler - 帧组装器

```rust
use parser::FrameAssembler;
use expto::rdmp::Unit;

let mut assembler = FrameAssembler::new();

// 将 Units 组装为 FrameData
let units: Vec<Unit> = /* ... */;
let timestamp_ms = 1000; // 毫秒时间戳
let frame_data = assembler.assemble_frame(&units, timestamp_ms);

println!("Frame {}: {} units", frame_data.id, frame_data.units.len());
```

### DefaultMaterialConfig - 默认材质配置

```rust
use parser::DefaultMaterialConfig;

// 使用默认配置
let config = DefaultMaterialConfig::default();
// material_path: "materials/default.toml"

// 自定义配置
let config = DefaultMaterialConfig::new("materials/custom.toml");
```

## 在 Bevy 中使用

在 `redra` 主应用中，`ParserManagerPlugin` 已经集成了 parser：

```rust
use bevy::prelude::*;
use redra::RedraPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RedraPlugin)  // 已包含 ParserManagerPlugin
        .run();
}
```

如需自定义解析行为，可以在自己的系统中调用 parser 的 API：

```rust
use parser::{ProtocolParser, ParsedCommand};

fn custom_parse_system(
    mut commands: Commands,
    units: Res<Vec<Unit>>,  // 假设你有 Units 资源
) {
    for unit in units.iter() {
        let cmd = ProtocolParser::parse_unit(unit);
        
        match cmd {
            ParsedCommand::Spawn { id, transform } => {
                // 创建实体...
            },
            // ...
        }
    }
}
```

## 依赖关系

- **expto**: 协议定义（Protobuf 生成的类型）
- **inpto**: 内存数据结构（FrameData 等）
- **log**: 日志记录

**注意**: 本 crate **不依赖** bevy、bevy_materialize 或其他图形相关库。

## 测试

```bash
cargo test -p parser
```

所有测试都是纯单元测试，无需 Bevy 运行时。

## 设计原则

1. **单一职责**: parser 只做解析，不做渲染
2. **无状态**: 所有解析函数都是纯函数或仅依赖简单状态
3. **可组合**: 可以轻松与其他系统集成
4. **易测试**: 无需模拟复杂环境
