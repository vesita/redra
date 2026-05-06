//! .rdra 文件写入器 — 离线生成 Redra 数据文件
//!
//! 与 `ShapeBuilder` 配合使用，先逐帧写入实体，最后保存为 `.rdra` 文件。
//! 生成的文件与主程序的保存格式完全兼容，可直接用 Redra 打开。
//!
//! ```no_run
//! use redra_client::*;
//!
//! let mut writer = RdraWriter::new();
//! writer.spawn(spawn_sphere([0.0, 0.0, 0.0], 1.0, "red").id(1));
//! writer.end_frame();
//! writer.save("output.rdra").unwrap();
//! ```

use std::collections::HashMap;
use std::path::Path;

use expto::prelude::*;
use serde::{Deserialize, Serialize};

use super::builder::ShapeBuilder;

// ─── 序列化格式（与主程序 keyframe.rs 兼容）────────────────

#[derive(Serialize, Deserialize)]
struct SerializableTransform {
    translation: [f32; 3],
    rotation: [f32; 4],  // 四元数 [x, y, z, w]
    scale: [f32; 3],
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerializableEntity {
    mesh: ExMesh,
    material: String,
    transform: SerializableTransform,
    tag: Option<Tag>,
}

#[derive(Serialize, Deserialize)]
struct SerializableKeyFrame {
    timestamp: u64,
    entities: Vec<(u64, SerializableEntity)>,
}

// ─── RdraWriter ──────────────────────────────────────────────

/// .rdra 文件写入器。
///
/// 逐帧积累实体数据，最终输出为 `.rdra` 文件。
/// 实体状态跨帧持续：未显式删除的实体会自动继承到下一帧。
pub struct RdraWriter {
    /// 当前帧的实体快照（id → entity）
    entities: HashMap<u64, SerializableEntity>,
    /// 已完成的帧列表
    keyframes: Vec<SerializableKeyFrame>,
    /// 自动分配 ID 的计数器
    next_auto_id: u64,
    /// 帧时间戳计数器（每帧递增 200ms，模拟 5fps）
    timestamp: u64,
}

impl RdraWriter {
    /// 创建新的写入器
    pub fn new() -> Self {
        RdraWriter {
            entities: HashMap::new(),
            keyframes: Vec::new(),
            next_auto_id: 1,
            timestamp: 0,
        }
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

        let entity = SerializableEntity {
            mesh: builder.mesh,
            material: builder.material.unwrap_or_default(),
            transform: SerializableTransform {
                translation: [builder.tx, builder.ty, builder.tz],
                rotation: [qx, qy, qz, qw],
                scale: [builder.sx, builder.sy, builder.sz],
            },
            tag: builder.tag,
        };

        self.entities.insert(id, entity);
        id
    }

    /// 从当前帧中删除指定 ID 的实体
    pub fn destroy(&mut self, id: u64) {
        self.entities.remove(&id);
    }

    /// 结束当前帧，将实体快照保存为一个 keyframe。
    ///
    /// 实体状态会持续到下一帧（不清空）。
    pub fn end_frame(&mut self) {
        let entities: Vec<(u64, SerializableEntity)> = self.entities.iter()
            .map(|(&id, e)| (id, SerializableEntity {
                mesh: e.mesh.clone(),
                material: e.material.clone(),
                transform: SerializableTransform {
                    translation: e.transform.translation,
                    rotation: e.transform.rotation,
                    scale: e.transform.scale,
                },
                tag: e.tag.clone(),
            }))
            .collect();

        self.keyframes.push(SerializableKeyFrame {
            timestamp: self.timestamp,
            entities,
        });

        self.timestamp += 200;
    }

    /// 将所有帧写入 .rdra 文件
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }
        let data = bincode::serialize(&self.keyframes)
            .map_err(|e| format!("序列化失败: {}", e))?;
        std::fs::write(path, data)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        log::info!("已保存 {} 帧到: {}", self.keyframes.len(), path.display());
        Ok(())
    }

    /// 当前帧的实体数量
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// 已积累的帧数
    pub fn frame_count(&self) -> usize {
        self.keyframes.len()
    }

    /// 清空所有帧和实体数据，释放内存
    pub fn clear(&mut self) {
        self.entities.clear();
        self.keyframes.clear();
        self.next_auto_id = 1;
        self.timestamp = 0;
    }
}

impl Default for RdraWriter {
    fn default() -> Self { Self::new() }
}
