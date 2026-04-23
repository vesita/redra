//! 帧数据持久化模块 - 简化版本
//!
//! 使用 bincode 进行二进制序列化，提供简化的文件保存接口
//!
//! # 设计说明
//! - 当前版本仅支持简单的文件保存/加载功能
//! - 如需数据库元数据索引功能，可扩展 database 模块

use serde::{Serialize, Deserialize};
use expto::rdmp::ExMesh;
use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy::math::Quat;
use std::path::{Path, PathBuf};

// ============================================================================
// 可序列化的中间数据结构（与 Bevy 解耦）
// ============================================================================

/// 简化的变换数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformData {
    pub translation: [f32; 3],
    pub rotation: [f32; 4], // quaternion (x, y, z, w)
    pub scale: [f32; 3],
}

impl From<Transform> for TransformData {
    fn from(transform: Transform) -> Self {
        Self {
            translation: transform.translation.into(),
            rotation: transform.rotation.into(),
            scale: transform.scale.into(),
        }
    }
}

impl From<TransformData> for Transform {
    fn from(data: TransformData) -> Self {
        Self {
            translation: data.translation.into(),
            rotation: Quat::from_array(data.rotation),
            scale: data.scale.into(),
        }
    }
}

/// 简化的网格数据（JSON 序列化后存为字节）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshData {
    pub mesh_type: String,
    pub params: Vec<u8>,
}

impl From<ExMesh> for MeshData {
    fn from(mesh: ExMesh) -> Self {
        let json = serde_json::to_vec(&mesh).unwrap_or_default();
        Self {
            mesh_type: format!("{:?}", mesh),
            params: json,
        }
    }
}

/// 简化的实体数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityData {
    pub entity_id: u64,
    pub mesh: MeshData,
    pub material: String,
    pub transform: TransformData,
}

impl From<(u64, ExMesh, String, Transform)> for EntityData {
    fn from((entity_id, mesh, material, transform): (u64, ExMesh, String, Transform)) -> Self {
        Self {
            entity_id,
            mesh: MeshData::from(mesh),
            material,
            transform: TransformData::from(transform),
        }
    }
}

/// 可序列化的关键帧
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableKeyFrame {
    pub timestamp: u64,
    pub entities: Vec<EntityData>,
}

impl From<(u64, Vec<EntityData>)> for SerializableKeyFrame {
    fn from((timestamp, entities): (u64, Vec<EntityData>)) -> Self {
        Self {
            timestamp,
            entities,
        }
    }
}

// ============================================================================
// Bevy Resource 和插件
// ============================================================================

/// 帧存储管理器（Bevy Resource）
#[derive(Resource)]
pub struct FrameStorage {
    /// 当前工作目录（用于相对路径）
    work_dir: PathBuf,
}

impl FrameStorage {
    /// 创建新的存储实例
    pub fn new(work_dir: &Path) -> Self {
        Self {
            work_dir: work_dir.to_path_buf(),
        }
    }

    /// 使用默认工作目录创建
    pub fn new_default() -> Result<Self, String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
            .parent()
            .ok_or("无法确定可执行文件所在目录".to_string())?
            .to_path_buf();

        Ok(Self::new(&exe_path))
    }

    /// 保存所有关键帧到指定文件（同步版本，用于 UI 线程）
    pub fn save_to_file(
        &self,
        path: &Path,
        keyframes: &[crate::manager::data::frame::basic::SerializableKeyFrame],
    ) -> Result<(), String> {
        use std::fs;

        // 确保目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }

        // 序列化所有关键帧
        let data = bincode::serialize(keyframes)
            .map_err(|e| format!("序列化失败: {}", e))?;

        // 写入文件
        fs::write(path, data)
            .map_err(|e| format!("写入文件失败: {}", e))?;

        log::info!("已保存 {} 帧到: {}", keyframes.len(), path.display());
        Ok(())
    }

    /// 从文件加载关键帧（同步版本）
    pub fn load_from_file(
        &self,
        path: &Path,
    ) -> Result<Vec<crate::manager::data::frame::basic::SerializableKeyFrame>, String> {
        use std::fs;

        // 读取文件
        let data = fs::read(path)
            .map_err(|e| format!("读取文件失败: {}", e))?;

        // 反序列化（显式类型注解）
        let keyframes: Vec<crate::manager::data::frame::basic::SerializableKeyFrame> = bincode::deserialize(&data)
            .map_err(|e| format!("反序列化失败: {}", e))?;

        log::info!("已从 {} 加载 {} 帧", path.display(), keyframes.len());
        Ok(keyframes)
    }
}

/// 帧存储插件
pub struct FrameStoragePlugin;

impl Plugin for FrameStoragePlugin {
    fn build(&self, app: &mut App) {
        // 初始化 FrameStorage 资源
        match FrameStorage::new_default() {
            Ok(storage) => {
                app.insert_resource(storage);
            }
            Err(e) => {
                log::warn!("初始化 FrameStorage 失败: {}，使用临时目录", e);
                app.insert_resource(FrameStorage::new(std::path::Path::new(".")));
            }
        }
    }
}
