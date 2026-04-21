use bevy::transform::components::Transform;
use expto::rdmp::{ExMesh};

use crate::manager::data::frame::Inpto;



impl Inpto {
    pub fn new(mesh: ExMesh, material: String, transform: Transform) -> Self {
        Self {
            mesh: mesh,
            material,
            transform: transform,
        }
    }

    // ==================== 数据访问接口（供 FrameRenderer 使用）====================

    /// 获取材质文件路径
    /// 如果 material 为空，返回默认材质路径
    pub fn material_path(&self) -> String {
        if self.material.is_empty() {
            "materials/default.toml".to_string()
        } else {
            self.material.clone()
        }
    }

    /// 获取实体名称（用于调试日志）
    pub fn name(&self) -> &str {
        "FrameEntity"
    }
}