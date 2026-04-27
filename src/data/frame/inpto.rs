use bevy::transform::components::Transform;
use expto::rdmp::{ExMesh, Tag};

/// 中间表示 — 协议数据到渲染数据的转换单元
pub struct Inpto {
    pub mesh: ExMesh,
    pub material: String,
    pub transform: Transform,
    pub tag: Option<Tag>,
}

impl Inpto {
    pub fn new(mesh: ExMesh, material: String, transform: Transform) -> Self {
        Self { mesh, material, transform, tag: None }
    }

    pub fn with_tag(mut self, tag: Tag) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn material_path(&self) -> String {
        if self.material.is_empty() {
            "materials/default.toml".to_string()
        } else {
            self.material.clone()
        }
    }

    pub fn name(&self) -> &str {
        "FrameEntity"
    }
}
