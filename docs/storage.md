# 数据存储模块

Redra 使用 SQLite（通过 sea-orm）进行帧数据的持久化存储，替代了旧版基于 bincode 的 `.rdra` 文件格式。

## 架构

```
src/data/frame/storage/
  ├── mod.rs    ← 模块入口，重导出 sql::FrameStorage、定义 FrameStoragePlugin
  └── sql.rs    ← sea-orm 实体定义 + FrameStorage 实现
```

存储模块是 `data` 层的一部分，通过 `FrameStoragePlugin` 注册为 Bevy Resource：

```
ControlPlugin
  └── FrameStoragePlugin
        └── FrameStorage (Resource)
              ├── conn: DatabaseConnection ← sea-orm SQLite
              ├── db_path: PathBuf         ← storage.db 路径
              └── rt: tokio::Runtime       ← 桥接同步/异步
```

## Schema

### frames 表

| 列 | 类型 | 说明 |
|---|---|---|
| `frame_id` | INTEGER PK | 帧序号 |
| `timestamp` | INTEGER | 时间戳 |
| `created_at` | INTEGER | 创建时间 |

### entities 表

每个实体存储为独立行，mesh 数据以 bincode BLOB 存储：

| 列 | 类型 | 说明 |
|---|---|---|
| `id` | INTEGER PK AUTO | 自增 ID |
| `entity_id` | INTEGER | 实体逻辑 ID |
| `frame_id` | INTEGER FK | 所属帧 |
| `material` | TEXT | 材质名 |
| `mesh_data` | BLOB | bincode 序列化的 ExMesh |
| `tx, ty, tz` | REAL | 平移 |
| `rx, ry, rz, rw` | REAL | 旋转（四元数） |
| `sx, sy, sz` | REAL | 缩放 |

### entity_tags 表

每个实体的每个标签存储为独立行：

| 列 | 类型 | 说明 |
|---|---|---|
| `id` | INTEGER PK AUTO | 自增 ID |
| `entity_id` | INTEGER | 实体逻辑 ID |
| `frame_id` | INTEGER | 所属帧 |
| `tag_index` | INTEGER | 标签序号 |
| `tag_text` | TEXT | 标签文本（可查询） |
| `tag_data` | BLOB | bincode 序列化的完整 Tag 结构体 |

索引：`entities(frame_id)`、`entities(entity_id)`、`entities(material)`、`entity_tags(tag_text)`

## 核心 API

### 实例化

```rust
// 默认位置（exe 同目录下 storage.db）
let storage = FrameStorage::new_default()?;

// 指定路径
let storage = FrameStorage::new(Path::new("path/to/storage.db"))?;
```

首次创建时自动执行 `CREATE TABLE IF NOT EXISTS` 和 `CREATE INDEX IF NOT EXISTS`。

### 流式写入

```rust
// 追加一帧，立即持久化，不占内存
let frame_id = storage.append_frame(&keyframe)?;

// 批量写入
storage.append_frames(&keyframes)?;
```

`append_frame` 在单个事务中完成帧记录、实体、标签的插入。写入后内存中的 `KeyFrame` 可以安全丢弃。

### 读取

```rust
// 加载单帧
let kf = storage.load_frame(frame_id)?;

// 加载全部帧
let all_frames = storage.load_all_frames()?;
```

### 查询

```rust
// 按材质查询
let results = storage.query_by_material("red")?;  // Vec<(frame_id, entity_id)>

// 按标签文本查询
let results = storage.query_by_tag("标签内容")?;
```

### 统计

```rust
let frames = storage.frame_count()?;  // u64
let entities = storage.entity_count()?;  // u64
let ids = storage.get_all_frame_ids()?;  // Vec<i32>
```

### 导入/导出

```rust
// 导出当前数据库为 .db 文件（先 VACUUM 再文件复制）
storage.export_db(Path::new("backup.db"))?;

// 导入旧 .rdra 文件到当前数据库
storage.import_rdra(Path::new("old_data.rdra"))?;

// 向后兼容：读取/写入 .rdra 文件（bincode 格式）
let frames = storage.load_from_file(Path::new("data.rdra"))?;
storage.save_to_file(Path::new("export.rdra"), &frames)?;
```

### 维护

```rust
storage.clear_all()?;  // 清空所有帧、实体、标签
storage.vacuum()?;      // 压缩数据库文件
```

## 与旧 .rdra 格式的兼容性

- **加载旧文件**：`load_from_file()` 使用 bincode 反序列化旧版 `.rdra` 文件（`Vec<keyframe::SerializableKeyFrame>`）
- **导入到 SQL**：`import_rdra()` 读取旧文件后将所有帧逐帧追加到当前 SQL 数据库
- **导出为旧格式**：`save_to_file()` 将帧数据序列化为 bincode `.rdra` 文件

## 同步/异步桥接

Bevy 系统是同步的，而 sea-orm 是异步的。`FrameStorage` 在构造函数中创建一个 `tokio::runtime::Runtime`（单线程、仅 I/O），所有公共方法使用 `self.rt.block_on(async { ... })` 在同步上下文中执行异步 sea-orm 操作。SQLite 操作耗时在微秒到毫秒级，block_on 不会对 UI 造成可感知的阻塞。

## 旧版存储（已删除）

旧版存储模块包含以下已删除的组件：

- `storage/database.rs` — 使用 sea-orm 但仅将实体序列化为 `frame_N.bin` 文件，SQL 表只存文件路径。从未被功能路径使用。
- `storage/serializable.rs` — 为 `database.rs` 提供 `From<&KeyFrame>` 转换。从未被功能路径使用。
- `storage.rs` 中的 `TransformData`、`MeshData`、`EntityData`、`SerializableKeyFrame` — 与 `keyframe.rs` 中的同名类型重复，从未被功能路径使用。

SQLite 支持于 2026-05 重构，废弃了 .rdra 作为原生格式。
