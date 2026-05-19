//! SQLite 写入器 — 直接写入 SQLite 数据库
//!
//! 与 `ShapeBuilder` 配合使用，将实体数据直接写入 SQLite 数据库。
//! 每帧通过 `end_frame()` 立即持久化，不累积在内存中。
//!
//! ```no_run
//! use redra_client::*;
//!
//! let mut writer = SqlWriter::new("output.db").unwrap();
//! writer.spawn(spawn_sphere([0.0, 0.0, 0.0], 1.0, "red").id(1));
//! writer.end_frame().unwrap();
//! writer.save().unwrap();
//! ```

use std::collections::HashMap;
use std::path::Path;

use expto::prelude::*;
use rusqlite::{Connection, params};

use super::builder::ShapeBuilder;

// ─── 内部数据结构 ────────────────────────────────────────────

#[derive(Clone)]
struct EntityData {
    mesh: ExMesh,
    material: String,
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
    tags: Vec<Tag>,
}

// ─── SqlWriter ─────────────────────────────────────────────────

/// SQLite 写入器。
///
/// 逐帧将实体数据写入 SQLite 数据库，不累积在内存中。
/// 与 `RdraWriter` API 兼容：使用 `spawn()` / `end_frame()` / `save()`。
/// 生成的数据库与 Redra 主程序的 `FrameStorage` 格式完全兼容。
///
/// 实体状态跨帧持续：未显式删除的实体会自动继承到下一帧。
pub struct SqlWriter {
    conn: Connection,
    /// 当前帧的实体快照（id → entity）
    entities: HashMap<u64, EntityData>,
    /// 自动分配 ID 的计数器
    next_auto_id: u64,
    /// 帧时间戳计数器（每帧递增 200ms，模拟 5fps）
    timestamp: u64,
}

impl SqlWriter {
    /// 创建新的写入器，打开或创建 SQLite 数据库文件。
    ///
    /// 自动创建 `frames`、`entities`、`entity_tags` 表和索引。
    pub fn new(path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open(path.as_ref())
            .map_err(|e| format!("打开数据库失败: {}", e))?;

        let writer = SqlWriter {
            conn,
            entities: HashMap::new(),
            next_auto_id: 1,
            timestamp: 0,
        };
        writer.init_tables()?;
        Ok(writer)
    }

    /// 初始化表结构
    fn init_tables(&self) -> Result<(), String> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS frames (
                frame_id INTEGER PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS entities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_id INTEGER NOT NULL,
                frame_id INTEGER NOT NULL,
                material TEXT NOT NULL DEFAULT '',
                mesh_data BLOB NOT NULL,
                tx REAL NOT NULL DEFAULT 0,
                ty REAL NOT NULL DEFAULT 0,
                tz REAL NOT NULL DEFAULT 0,
                rx REAL NOT NULL DEFAULT 0,
                ry REAL NOT NULL DEFAULT 0,
                rz REAL NOT NULL DEFAULT 0,
                rw REAL NOT NULL DEFAULT 1,
                sx REAL NOT NULL DEFAULT 1,
                sy REAL NOT NULL DEFAULT 1,
                sz REAL NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS entity_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_id INTEGER NOT NULL,
                frame_id INTEGER NOT NULL,
                tag_index INTEGER NOT NULL,
                tag_text TEXT NOT NULL,
                tag_data BLOB NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_entities_frame ON entities(frame_id);
            CREATE INDEX IF NOT EXISTS idx_entities_entity ON entities(entity_id);
            CREATE INDEX IF NOT EXISTS idx_entities_material ON entities(material);
            CREATE INDEX IF NOT EXISTS idx_tags_text ON entity_tags(tag_text);",
        )
        .map_err(|e| format!("初始化表结构失败: {}", e))?;
        Ok(())
    }

    /// 将 ShapeBuilder 写入当前帧。
    ///
    /// 如果 builder 未设置 ID，会自动分配一个。
    /// 如果同 ID 已存在，会覆盖旧实体。
    pub fn spawn(&mut self, builder: ShapeBuilder) -> u64 {
        let id = builder.id.unwrap_or_else(|| {
            let id = self.next_auto_id;
            self.next_auto_id += 1;
            id
        });

        // 欧拉角 → 四元数
        let q = nalgebra::UnitQuaternion::from_euler_angles(builder.rx, builder.ry, builder.rz);
        let [qx, qy, qz, qw] = q.quaternion().coords.into();

        self.entities.insert(
            id,
            EntityData {
                mesh: builder.mesh,
                material: builder.material.unwrap_or_default(),
                translation: [builder.tx, builder.ty, builder.tz],
                rotation: [qx, qy, qz, qw],
                scale: [builder.sx, builder.sy, builder.sz],
                tags: builder.tag_list,
            },
        );
        id
    }

    /// 从当前帧中删除指定 ID 的实体
    pub fn destroy(&mut self, id: u64) {
        self.entities.remove(&id);
    }

    /// 修改已有实体的材质
    pub fn set_material(&mut self, id: u64, material: impl Into<String>) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.material = material.into();
        }
    }

    /// 清除当前帧中所有实体
    pub fn destroy_all(&mut self) {
        self.entities.clear();
    }

    /// 从另一个 SqlWriter 克隆当前帧的所有实体到本写入器。
    ///
    /// 用于避免重复写入相同数据（如点云）到多个文件。
    /// 仅复制实体内存状态，不涉及 SQLite I/O。
    /// 不会覆盖本写入器中已存在的 ID。
    pub fn absorb(&mut self, other: &SqlWriter) {
        for (&id, data) in &other.entities {
            self.entities.entry(id).or_insert_with(|| data.clone());
        }
    }

    /// 结束当前帧，将所有实体写入 SQLite。
    ///
    /// 在一个事务中完成帧记录、实体、标签的插入。
    /// 实体状态会持续到下一帧（不清空）。
    pub fn end_frame(&mut self) -> Result<(), String> {
        let frame_id = self.next_frame_id()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 使用事务确保原子性
        self.conn
            .execute_batch("BEGIN TRANSACTION")
            .map_err(|e| format!("开始事务失败: {}", e))?;

        if let Err(e) = self.write_frame(frame_id, now) {
            let _ = self.conn.execute_batch("ROLLBACK");
            return Err(e);
        }

        self.conn
            .execute_batch("COMMIT")
            .map_err(|e| format!("提交事务失败: {}", e))?;

        self.timestamp += 200;
        Ok(())
    }

    /// 写入一帧的数据（已在事务中调用）
    fn write_frame(&self, frame_id: i64, created_at: i64) -> Result<(), String> {
        // 插入帧记录
        self.conn
            .execute(
                "INSERT INTO frames (frame_id, timestamp, created_at) VALUES (?1, ?2, ?3)",
                params![frame_id, self.timestamp as i64, created_at],
            )
            .map_err(|e| format!("插入帧记录失败: {}", e))?;

        // 插入所有实体
        for (&entity_id, data) in &self.entities {
            let mesh_data =
                bincode::serialize(&data.mesh).map_err(|e| format!("序列化 mesh 失败: {}", e))?;

            self.conn
                .execute(
                    "INSERT INTO entities (entity_id, frame_id, material, mesh_data, \
                     tx, ty, tz, rx, ry, rz, rw, sx, sy, sz) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                    params![
                        entity_id as i64,
                        frame_id,
                        data.material,
                        mesh_data,
                        data.translation[0],
                        data.translation[1],
                        data.translation[2],
                        data.rotation[0],
                        data.rotation[1],
                        data.rotation[2],
                        data.rotation[3],
                        data.scale[0],
                        data.scale[1],
                        data.scale[2],
                    ],
                )
                .map_err(|e| format!("插入实体失败: {}", e))?;

            // 插入标签
            for (ti, tag) in data.tags.iter().enumerate() {
                let tag_data =
                    bincode::serialize(tag).map_err(|e| format!("序列化 tag 失败: {}", e))?;
                self.conn
                    .execute(
                        "INSERT INTO entity_tags (entity_id, frame_id, tag_index, tag_text, tag_data) \
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![entity_id as i64, frame_id, ti as i32, tag.text, tag_data],
                    )
                    .map_err(|e| format!("插入标签失败: {}", e))?;
            }
        }

        Ok(())
    }

    /// 获取下一个 frame_id
    fn next_frame_id(&self) -> Result<i64, String> {
        let max_id: Option<i64> = self
            .conn
            .query_row("SELECT MAX(frame_id) FROM frames", [], |row| row.get(0))
            .map_err(|e| format!("查询最大 frame_id 失败: {}", e))?;
        Ok(max_id.unwrap_or(-1) + 1)
    }

    /// VACUUM 压缩数据库文件。
    ///
    /// 写入完成后调用，减小文件体积。
    pub fn save(&self) -> Result<(), String> {
        self.conn
            .execute_batch("VACUUM")
            .map_err(|e| format!("VACUUM 失败: {}", e))?;
        Ok(())
    }

    /// 当前帧的实体数量
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// 将另一个 .rdra 数据库文件中的所有实体批量复制到本数据库。
    ///
    /// 使用 SQL 级 ATTACH + INSERT-SELECT，无 Rust 序列化开销。
    /// 实体 ID 不重叠时最安全（点云 0..N，语义 800000+）。
    /// 不复制 frames 表（目标库已有正确的帧记录）。
    pub fn merge_db(target_path: &Path, source_path: &Path) -> Result<(), String> {
        let conn = Connection::open(target_path)
            .map_err(|e| format!("打开目标数据库失败: {}", e))?;
        let source_str = source_path.to_str().ok_or_else(|| "路径包含无效 UTF-8".to_string())?;
        conn.execute("ATTACH DATABASE ? AS src", params![source_str])
            .map_err(|e| format!("ATTACH 失败: {}", e))?;
        let r = conn.execute_batch(
            "BEGIN; \
             INSERT INTO entities (entity_id, frame_id, material, mesh_data, \
              tx, ty, tz, rx, ry, rz, rw, sx, sy, sz) \
             SELECT entity_id, frame_id, material, mesh_data, \
              tx, ty, tz, rx, ry, rz, rw, sx, sy, sz FROM src.entities; \
             INSERT INTO entity_tags (entity_id, frame_id, tag_index, tag_text, tag_data) \
             SELECT entity_id, frame_id, tag_index, tag_text, tag_data FROM src.entity_tags; \
             COMMIT;",
        );
        conn.execute_batch("DETACH DATABASE src").ok();
        r.map_err(|e| format!("合并实体失败: {}", e))
    }

    /// 清空所有帧数据。
    ///
    /// 清除所有实体和帧记录，重置内部计数器。
    pub fn clear_all(&mut self) -> Result<(), String> {
        self.entities.clear();
        self.next_auto_id = 1;
        self.timestamp = 0;
        self.conn
            .execute_batch("DELETE FROM entity_tags; DELETE FROM entities; DELETE FROM frames")
            .map_err(|e| format!("清空数据失败: {}", e))?;
        Ok(())
    }
}
