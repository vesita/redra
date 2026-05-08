//! 帧数据持久化模块 — 使用 bincode 进行二进制序列化

use serde::{Serialize, Deserialize};
use expto::rdmp::{ExMesh, Tag};
use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy::math::Quat;
use std::path::{Path, PathBuf};

pub mod serializable;
pub mod database;

// ============================================================================
// 可序列化的中间数据结构
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformData {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl From<Transform> for TransformData {
    fn from(transform: Transform) -> Self {
        Self { translation: transform.translation.into(), rotation: transform.rotation.into(), scale: transform.scale.into() }
    }
}

impl From<TransformData> for Transform {
    fn from(data: TransformData) -> Self {
        Self { translation: data.translation.into(), rotation: Quat::from_array(data.rotation), scale: data.scale.into() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshData {
    pub mesh_type: String,
    pub params: Vec<u8>,
}

impl From<ExMesh> for MeshData {
    fn from(mesh: ExMesh) -> Self {
        let json = serde_json::to_vec(&mesh).unwrap_or_default();
        Self { mesh_type: format!("{:?}", mesh), params: json }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityData {
    pub entity_id: u64,
    pub mesh: MeshData,
    pub material: String,
    pub transform: TransformData,
    #[serde(default)]
    pub tags: Vec<Tag>,
}

impl From<(u64, ExMesh, String, Transform)> for EntityData {
    fn from((entity_id, mesh, material, transform): (u64, ExMesh, String, Transform)) -> Self {
        Self { entity_id, mesh: MeshData::from(mesh), material, transform: TransformData::from(transform), tags: Vec::new() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableKeyFrame {
    pub timestamp: u64,
    pub entities: Vec<EntityData>,
}

// ============================================================================
// Bevy Resource 和插件
// ============================================================================

#[derive(Resource)]
pub struct FrameStorage {
    #[allow(dead_code)]
    work_dir: PathBuf,
}

impl FrameStorage {
    pub fn new(work_dir: &Path) -> Self {
        Self { work_dir: work_dir.to_path_buf() }
    }

    pub fn new_default() -> Result<Self, String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
            .parent()
            .ok_or("无法确定可执行文件所在目录".to_string())?
            .to_path_buf();
        Ok(Self::new(&exe_path))
    }

    pub fn save_to_file(
        &self,
        path: &Path,
        keyframes: &[crate::data::frame::SerializableKeyFrame],
    ) -> Result<(), String> {
        use std::fs;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
        let data = bincode::serialize(keyframes).map_err(|e| format!("序列化失败: {}", e))?;
        fs::write(path, data).map_err(|e| format!("写入文件失败: {}", e))?;
        log::info!("已保存 {} 帧到: {}", keyframes.len(), path.display());
        Ok(())
    }

    pub fn load_from_file(
        &self,
        path: &Path,
    ) -> Result<Vec<crate::data::frame::SerializableKeyFrame>, String> {
        use std::fs;
        let data = fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
        let keyframes: Vec<crate::data::frame::SerializableKeyFrame> = bincode::deserialize(&data)
            .map_err(|e| format!("反序列化失败: {}", e))?;
        log::info!("已从 {} 加载 {} 帧", path.display(), keyframes.len());
        Ok(keyframes)
    }
}

pub struct FrameStoragePlugin;

impl Plugin for FrameStoragePlugin {
    fn build(&self, app: &mut App) {
        match FrameStorage::new_default() {
            Ok(storage) => { app.insert_resource(storage); }
            Err(e) => {
                log::warn!("初始化 FrameStorage 失败: {}，使用临时目录", e);
                app.insert_resource(FrameStorage::new(std::path::Path::new(".")));
            }
        }
    }
}
