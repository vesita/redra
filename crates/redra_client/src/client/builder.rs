//! 实体构建器 — 链式 API 构建并发送带标签、材质、变换的实体
//!
//! [`ShapeBuilder`] 是 redra 客户端的核心入口，支持所有 mesh 类型的构造与发送。
//!
//! # 快速开始
//!
//! ```no_run
//! use redra_client::ShapeBuilder;
//!
//! // 单个实体
//! ShapeBuilder::sphere(1.0)
//!     .id(42)
//!     .at(1.0, 2.0, 3.0)
//!     .material("red")
//!     .tag("我的球体")
//!     .send().await.unwrap();
//!
//! // 有向包围盒（OBB）
//! ShapeBuilder::cube(vec![
//!     (0.0, 0.0, 0.0), (1.0, 0.0, 0.0),
//!     (1.0, 0.0, 1.0), (0.0, 0.0, 1.0),
//!     (0.0, 1.0, 0.0), (1.0, 1.0, 0.0),
//!     (1.0, 1.0, 1.0), (0.0, 1.0, 1.0),
//! ]).material("bounding_box").send().await.unwrap();
//!
//! // 分组点云
//! ShapeBuilder::point_cloud_grouped()
//!     .group(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]], "red")
//!     .group(vec![[0.0, 1.0, 0.0]], "blue")
//!     .send().await.unwrap();
//! ```
//!
//! # 支持的 mesh 类型
//!
//! | 类型 | 构造方法 | 参数 |
//! |------|---------|------|
//! | 球体 | `sphere(radius)` | 半径 |
//! | 圆柱 | `cylinder(radius, height)` | 半径, 高度 |
//! | 圆锥 | `cone(radius, height)` | 半径, 高度 |
//! | 点 | `point(x, y, z)` | 坐标 |
//! | 线段 | `line(x1,y1,z1, x2,y2,z2)` | 起终点 |
//! | OBB | `cube(vertices)` | 8 个角点 |
//! | 分组点云 | `point_cloud_grouped()` | `.group()` 链式添加 |

use expto::prelude::*;
use expto::rdmp::auto::unit::generate_unit;
use expto::rdmp::{
    Cone, Cube, Cylinder, ExMesh, Line, Point, Sphere,
};
use nalgebra::{UnitQuaternion, Vector3};

use super::link::get_link;

// ─── 分组点云 ──────────────────────────────────────────────

/// 一组共享材质的点
pub struct PointGroup {
    pub points: Vec<[f32; 3]>,
    pub material: String,
}

// ─── ShapeBuilder ────────────────────────────────────────────

/// 实体构建器，支持链式配置 ID、位置、缩放、材质、标签后一次性发送。
///
/// 也支持分组点云（`point_cloud_grouped()`），按材质分组发送点云。
///
/// ```no_run
/// use redra_client::ShapeBuilder;
///
/// // 球体
/// ShapeBuilder::sphere(1.0)
///     .id(42)
///     .at(1.0, 2.0, 3.0)
///     .material("red")
///     .tag("我的球体")
///     .send().await.unwrap();
///
/// // 有向包围盒（OBB）
/// ShapeBuilder::cube(vec![
///     (0.0, 0.0, 0.0), (2.0, 0.0, 0.0),
///     (2.0, 0.0, 1.0), (0.0, 0.0, 1.0),
///     (0.0, 1.0, 0.0), (2.0, 1.0, 0.0),
///     (2.0, 1.0, 1.0), (0.0, 1.0, 1.0),
/// ]).material("bounding_box").send().await.unwrap();
///
/// // 分组点云
/// ShapeBuilder::point_cloud_grouped()
///     .group(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]], "red")
///     .group(vec![[0.0, 1.0, 0.0]], "blue")
///     .send().await.unwrap();
/// ```
pub struct ShapeBuilder {
    pub(crate) id: Option<u64>,
    pub(crate) mesh: ExMesh,
    pub(crate) tx: f32, pub(crate) ty: f32, pub(crate) tz: f32,
    pub(crate) rx: f32, pub(crate) ry: f32, pub(crate) rz: f32,
    pub(crate) sx: f32, pub(crate) sy: f32, pub(crate) sz: f32,
    pub(crate) material: Option<String>,
    pub(crate) tag: Option<Tag>,
    pub(crate) groups: Option<Vec<PointGroup>>,
}

impl ShapeBuilder {
    // ─── 形状构造 ─────────────────────────────────────────

    /// 分组点云 — 按材质分组，不同组渲染为不同颜色
    ///
    /// 每组通过 `.group(points, material)` 添加，最终 `.send()` 一次性发送。
    /// 适用于聚类可视化、地面/障碍物分离等场景。
    pub fn point_cloud_grouped() -> Self {
        ShapeBuilder {
            id: None,
            mesh: ExMesh { u_mesh: None },
            tx: 0.0, ty: 0.0, tz: 0.0,
            rx: 0.0, ry: 0.0, rz: 0.0,
            sx: 1.0, sy: 1.0, sz: 1.0,
            material: None, tag: None,
            groups: Some(Vec::new()),
        }
    }

    /// 添加一组共享材质的点（短名称如 `"red"` 或 `"cluster_01"`）
    pub fn group(mut self, points: Vec<[f32; 3]>, material: impl Into<String>) -> Self {
        if let Some(ref mut groups) = self.groups {
            groups.push(PointGroup { points, material: material.into() });
        }
        self
    }

    /// 球体（`radius` — 半径，必须 > 0）
    pub fn sphere(radius: f32) -> Self {
        Self::new(ExMesh::from(Sphere { location: Some(Point { x: 0.0, y: 0.0, z: 0.0 }), radius }))
    }

    /// 圆柱体（`radius` — 半径, `height` — 高度，均须 > 0）
    pub fn cylinder(radius: f32, height: f32) -> Self {
        Self::new(ExMesh::from(Cylinder::from((radius, height))))
    }

    /// 圆锥体（`radius` — 半径, `height` — 高度，均须 > 0）
    pub fn cone(radius: f32, height: f32) -> Self {
        Self::new(ExMesh::from(Cone::from((radius, height))))
    }

    /// 点（`x`, `y`, `z` — 世界坐标）
    pub fn point(x: f32, y: f32, z: f32) -> Self {
        Self::new(ExMesh::from(Point::from((x, y, z))))
    }

    /// 线段（起点 `(x1,y1,z1)` → 终点 `(x2,y2,z2)`）— 自动计算中点与朝向
    pub fn line(x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32) -> Self {
        let line_mesh = Line::from((Point { x: x1, y: y1, z: z1 }, Point { x: x2, y: y2, z: z2 }));
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dz = z2 - z1;
        let len = (dx * dx + dy * dy + dz * dz).sqrt();

        let (rx, ry, rz) = if len < 1e-6 {
            (0.0, 0.0, 0.0)
        } else {
            let dir = Vector3::new(dx / len, dy / len, dz / len);
            let q = UnitQuaternion::rotation_between(&Vector3::y(), &dir)
                .unwrap_or(UnitQuaternion::identity());
            q.euler_angles()
        };

        ShapeBuilder {
            id: None,
            mesh: ExMesh::from(line_mesh),
            tx: (x1 + x2) / 2.0, ty: (y1 + y2) / 2.0, tz: (z1 + z2) / 2.0,
            rx, ry, rz,
            sx: 1.0, sy: 1.0, sz: 1.0,
            material: None,
            tag: None,
            groups: None,
        }
    }

    /// 有向包围盒（8 个角点）— 自动计算质心作为实体位置
    ///
    /// 8 个角点可表示任意朝向的 OBB（有向包围盒），渲染端保留原始朝向。
    /// 传入轴对齐的 8 个点时等价于 AABB。
    ///
    /// **约束**：每个维度（宽/高/深）必须 > 0.001，否则渲染端会拒绝该 mesh。
    /// 对于退化包围盒（点共面/共线/单点），建议改用 `sphere()` 或 `point()`。
    pub fn cube(vertices: Vec<(f32, f32, f32)>) -> Self {
        let n = vertices.len() as f32;
        let points: Vec<Point> = vertices.iter().map(|&(x, y, z)| Point { x, y, z }).collect();

        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for p in &points {
            for i in 0..3 {
                min[i] = min[i].min([p.x, p.y, p.z][i]);
                max[i] = max[i].max([p.x, p.y, p.z][i]);
            }
        }
        let dims = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
        let min_dim = crate::defaults::mesh_constraints::MIN_CUBE_DIMENSION;
        if dims[0] < min_dim || dims[1] < min_dim || dims[2] < min_dim {
            log::warn!(
                "Cube 维度退化 (w={:.4}, h={:.4}, d={:.4})，渲染端将拒绝此 mesh。\
                 建议对退化包围盒改用 sphere() 或 point()。",
                dims[0], dims[1], dims[2]
            );
        }

        // 质心（与渲染端一致）
        let cx = points.iter().map(|p| p.x).sum::<f32>() / n;
        let cy = points.iter().map(|p| p.y).sum::<f32>() / n;
        let cz = points.iter().map(|p| p.z).sum::<f32>() / n;

        ShapeBuilder {
            id: None,
            mesh: ExMesh::from(Cube { vertices: points }),
            tx: cx, ty: cy, tz: cz,
            rx: 0.0, ry: 0.0, rz: 0.0,
            sx: 1.0, sy: 1.0, sz: 1.0,
            material: None,
            tag: None,
            groups: None,
        }
    }

    // ─── 链式配置 ─────────────────────────────────────────

    /// 设置实体 ID
    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id); self
    }

    /// 设置位置
    pub fn at(mut self, x: f32, y: f32, z: f32) -> Self {
        self.tx = x; self.ty = y; self.tz = z; self
    }

    /// 设置缩放（三个轴）
    pub fn scale(mut self, sx: f32, sy: f32, sz: f32) -> Self {
        self.sx = sx; self.sy = sy; self.sz = sz; self
    }

    /// 等比缩放
    pub fn scale_uniform(mut self, s: f32) -> Self {
        self.sx = s; self.sy = s; self.sz = s; self
    }

    /// 设置旋转（弧度制欧拉角）
    pub fn rotation(mut self, rx: f32, ry: f32, rz: f32) -> Self {
        self.rx = rx; self.ry = ry; self.rz = rz; self
    }

    /// 设置旋转（角度制欧拉角）
    pub fn rotation_deg(mut self, rx: f32, ry: f32, rz: f32) -> Self {
        let deg_to_rad = std::f32::consts::PI / 180.0;
        self.rx = rx * deg_to_rad;
        self.ry = ry * deg_to_rad;
        self.rz = rz * deg_to_rad;
        self
    }

    /// 设置材质（短名称如 "red"、"metal"，或完整 TOML 路径）
    pub fn material(mut self, id: impl Into<String>) -> Self {
        self.material = Some(id.into()); self
    }

    /// 设置标签（接受 `&str` / `String` / `Tag`）
    pub fn tag(mut self, tag: impl IntoTag) -> Self {
        self.tag = Some(tag.into_tag()); self
    }

    // ─── 发送 ─────────────────────────────────────────────

    /// 构建 Unit 并发送
    pub async fn send(self) -> Result<(), String> {
        // 分组点云模式：每组一个 Unit，共享材质
        if let Some(groups) = self.groups {
            let link = get_link().await;
            let mut id_counter: u64 = 1;
            for group in &groups {
                let mut unit = generate_unit();
                for pos in &group.points {
                    unit.objects.push(ExObject::from(id_counter));
                    id_counter += 1;
                    let p: Point = (pos[0], pos[1], pos[2]).into();
                    unit.objects.push(ExObject::from(ExMesh::from(p)));
                    unit.objects.push(ExObject::from(ExTransform {
                        x: pos[0], y: pos[1], z: pos[2],
                        rx: 0.0, ry: 0.0, rz: 0.0,
                        sx: 1.0, sy: 1.0, sz: 1.0,
                    }));
                    use expto::rdmp::ex_object::UObject;
                    unit.objects.push(ExObject { u_object: Some(UObject::MaterialId(group.material.clone())) });
                }
                let buf = encode(&unit).map_err(|e| format!("{}", e))?;
                link.send(&buf).await?;
            }
            return Ok(());
        }

        // 单实体模式
        let mut unit = generate_unit();

        if let Some(id) = self.id {
            unit.objects.push(ExObject::from(id));
        }

        unit.objects.push(ExObject::from(self.mesh));
        unit.objects.push(ExObject::from(ExTransform {
            x: self.tx, y: self.ty, z: self.tz,
            rx: self.rx, ry: self.ry, rz: self.rz,
            sx: self.sx, sy: self.sy, sz: self.sz,
        }));

        if let Some(mat) = self.material {
            use expto::rdmp::ex_object::UObject;
            unit.objects.push(ExObject { u_object: Some(UObject::MaterialId(mat)) });
        }

        if let Some(tag) = self.tag {
            unit.objects.push(ExObject::from(tag));
        }

        let link = get_link().await;
        let buf = encode(&unit).map_err(|e| format!("{}", e))?;
        link.send(&buf).await?;
        Ok(())
    }

    // ─── 内部 ─────────────────────────────────────────────

    fn new(mesh: ExMesh) -> Self {
        ShapeBuilder {
            id: None, mesh,
            tx: 0.0, ty: 0.0, tz: 0.0,
            rx: 0.0, ry: 0.0, rz: 0.0,
            sx: 1.0, sy: 1.0, sz: 1.0,
            material: None, tag: None,
            groups: None,
        }
    }
}

// ─── IntoTag 辅助 trait（绕过孤儿规则）──────────────────────

/// 将常见类型转为 Tag 的辅助 trait。
/// 避免在外部 crate 实现 `From<&str> for Tag`（违反 orphan rules）。
pub trait IntoTag {
    fn into_tag(self) -> Tag;
}

impl IntoTag for Tag {
    fn into_tag(self) -> Tag { self }
}

impl IntoTag for &str {
    fn into_tag(self) -> Tag { Tag::new(self) }
}

impl IntoTag for String {
    fn into_tag(self) -> Tag { Tag::new(self) }
}

// ─── 便捷函数 ────────────────────────────────────────────────

/// 球体（位置, 半径, 材质）
pub fn spawn_sphere(pos: [f32; 3], radius: f32, material: impl Into<String>) -> ShapeBuilder {
    ShapeBuilder::sphere(radius).at(pos[0], pos[1], pos[2]).material(material)
}

/// 圆柱体（位置, 半径, 高度, 材质）
pub fn spawn_cylinder(pos: [f32; 3], radius: f32, height: f32, material: impl Into<String>) -> ShapeBuilder {
    ShapeBuilder::cylinder(radius, height).at(pos[0], pos[1], pos[2]).material(material)
}

/// 圆锥体（位置, 半径, 高度, 材质）
pub fn spawn_cone(pos: [f32; 3], radius: f32, height: f32, material: impl Into<String>) -> ShapeBuilder {
    ShapeBuilder::cone(radius, height).at(pos[0], pos[1], pos[2]).material(material)
}

/// 包围盒（角点, 材质）— 自动计算 AABB 中心
pub fn spawn_cube(vertices: Vec<(f32, f32, f32)>, material: impl Into<String>) -> ShapeBuilder {
    ShapeBuilder::cube(vertices).material(material)
}

/// 点（位置, 材质）
pub fn spawn_point(pos: [f32; 3], material: impl Into<String>) -> ShapeBuilder {
    ShapeBuilder::point(pos[0], pos[1], pos[2]).material(material)
}

/// 线段（起点, 终点, 材质）
pub fn spawn_line(from: [f32; 3], to: [f32; 3], material: impl Into<String>) -> ShapeBuilder {
    ShapeBuilder::line(from[0], from[1], from[2], to[0], to[1], to[2]).material(material)
}

/// 发送帧结束标记
pub async fn send_frame_end() -> Result<(), String> {
    let mut unit = generate_unit();
    unit.command = Some(ExCommand { u_command: CommandType::Frameend as i32 });
    let link = get_link().await;
    let buf = encode(&unit).map_err(|e| format!("{}", e))?;
    link.send(&buf).await?;
    Ok(())
}
