use bevy::transform::components::Transform;
use expto::rdmp::{ExMesh, Tag};

/// 中间表示 — 协议数据到渲染数据的转换单元
pub struct Inpto {
    pub mesh: ExMesh,
    pub material: String,
    pub transform: Transform,
    pub tags: Vec<Tag>,
}

impl Inpto {
    pub fn new(mesh: ExMesh, material: String, transform: Transform) -> Self {
        Self { mesh, material, transform, tags: Vec::new() }
    }

    pub fn with_tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn with_tags(mut self, tags: Vec<Tag>) -> Self {
        self.tags = tags;
        self
    }

    pub fn material_path(&self) -> String {
        if !self.material.is_empty() {
            return self.material.clone();
        }
        // 未指定材质时，根据网格类型自动分配
        use expto::rdmp::mesh::ex_mesh::UMesh;
        match &self.mesh.u_mesh {
            Some(UMesh::Point(_)) => "materials/mesh_types/point.toml",
            Some(UMesh::Line(_)) => "materials/mesh_types/line.toml",
            Some(UMesh::Sphere(_)) => "materials/mesh_types/sphere.toml",
            Some(UMesh::Cylinder(_)) => "materials/mesh_types/cylinder.toml",
            Some(UMesh::Cone(_)) => "materials/mesh_types/cone.toml",
            Some(UMesh::Cube(_)) => "materials/mesh_types/cube.toml",
            None => "materials/default.toml",
        }.to_string()
    }

    pub fn name(&self) -> &str {
        "FrameEntity"
    }
}
