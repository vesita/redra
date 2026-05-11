use expto::rdmp::{ExMesh, Tag};

/// 与 bevy Transform 无关的内部变换表示
#[derive(Clone, Copy, Debug)]
pub struct InptoTransform {
    pub tx: f32,
    pub ty: f32,
    pub tz: f32,
    pub rx: f32,
    pub ry: f32,
    pub rz: f32,
    pub rw: f32,
    pub sx: f32,
    pub sy: f32,
    pub sz: f32,
}

impl InptoTransform {
    pub fn identity() -> Self {
        Self {
            tx: 0.0, ty: 0.0, tz: 0.0,
            rx: 0.0, ry: 0.0, rz: 0.0, rw: 1.0,
            sx: 1.0, sy: 1.0, sz: 1.0,
        }
    }
}

impl Default for InptoTransform {
    fn default() -> Self {
        Self::identity()
    }
}

// ── From ExTransform ──

impl From<expto::rdmp::ExTransform> for InptoTransform {
    fn from(t: expto::rdmp::ExTransform) -> Self {
        // 欧拉角转四元数（与协议一致）
        let (sx_half, cx_half) = (t.rx * 0.5).sin_cos();
        let (sy_half, cy_half) = (t.ry * 0.5).sin_cos();
        let (sz_half, cz_half) = (t.rz * 0.5).sin_cos();
        Self {
            tx: t.x, ty: t.y, tz: t.z,
            rx: sx_half * cy_half * cz_half + cx_half * sy_half * sz_half,
            ry: cx_half * sy_half * cz_half - sx_half * cy_half * sz_half,
            rz: cx_half * cy_half * sz_half + sx_half * sy_half * cz_half,
            rw: cx_half * cy_half * cz_half - sx_half * sy_half * sz_half,
            sx: t.sx, sy: t.sy, sz: t.sz,
        }
    }
}

/// 中间表示 — 协议数据到渲染数据的转换单元
pub struct Inpto {
    pub mesh: ExMesh,
    pub material: String,
    pub transform: InptoTransform,
    pub tags: Vec<Tag>,
}

impl Inpto {
    pub fn new(mesh: ExMesh, material: String, transform: InptoTransform) -> Self {
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

// ── bevy Transform 转换 (仅 graph feature) ──

#[cfg(feature = "graph")]
impl From<InptoTransform> for bevy::transform::components::Transform {
    fn from(t: InptoTransform) -> Self {
        Self {
            translation: bevy::math::Vec3::new(t.tx, t.ty, t.tz),
            rotation: bevy::math::Quat::from_xyzw(t.rx, t.ry, t.rz, t.rw),
            scale: bevy::math::Vec3::new(t.sx, t.sy, t.sz),
        }
    }
}

#[cfg(feature = "graph")]
impl From<bevy::transform::components::Transform> for InptoTransform {
    fn from(t: bevy::transform::components::Transform) -> Self {
        Self {
            tx: t.translation.x, ty: t.translation.y, tz: t.translation.z,
            rx: t.rotation.x, ry: t.rotation.y, rz: t.rotation.z, rw: t.rotation.w,
            sx: t.scale.x, sy: t.scale.y, sz: t.scale.z,
        }
    }
}
