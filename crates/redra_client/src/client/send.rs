//! 便捷发送函数 — 单行调用构造并发送各类实体
//!
//! 本模块提供过程式 API，适合快速原型或简单场景。
//! 对于需要链式配置 ID、材质、标签的场景，请使用 `ShapeBuilder`（见 `builder` 模块）。
//!
//! # 函数一览
//!
//! | 函数 | 说明 |
//! |------|------|
//! | `send_point` | 单个点 |
//! | `send_point_cloud` | 批量点云（单 Unit） |
//! | `send_point_cloud_grouped` | 分组点云（按材质分组） |
//! | `send_line` | 线段 |
//! | `send_sphere` | 球体 |
//! | `send_cylinder` | 圆柱体 |
//! | `send_cone` | 圆锥体 |
//! | `send_cube` / `send_cube_with_tag` | 包围盒 |
//! | `send_tag` / `send_tag_with_style` | 标签 |
//! | `send_set_material` | 更新实体材质 |
//! | `send_destroy` | 销毁实体 |

use expto::prelude::*;
use expto::rdmp::auto::unit::generate_unit;
use expto::rdmp::{Cube, ExObject, ExMesh, Point, Cylinder, Cone, Tag, TagStyle};
use nalgebra::{UnitQuaternion, Vector3};

use crate::client::link::get_link;

// 定义一个 trait 来扩展 Unit 的功能
#[allow(async_fn_in_trait)]
pub trait AutoSend4Unit {
    async fn send(&self) -> Result<(), String>;
}

impl AutoSend4Unit for Unit { 
    async fn send(&self) -> Result<(), String> { 
        match encode(self) {
            Ok(buf) => {
                let link = get_link().await;
                link.send(&buf).await?;
            },
            Err(e) => return Err(format!("{}", e)),
        }
        Ok(())
    }
}


/// 发送单个点
///
/// # 参数
/// * `x`, `y`, `z` — 世界坐标
pub async fn send_point(
    x: f32,
    y: f32,
    z: f32,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let point: Point = (x, y, z).into();
    let mesh: ExMesh = point.into();
    let object: ExObject = mesh.into();
    let _ = unit.set_object(object);

    unit.send().await?;
    Ok(())
}

/// 批量发送点云（零拷贝：不重新分配内部数据，每条消息一个 Unit）
///
/// 将所有点打包到单个 Unit 中，每个点附带一个自动生成的 ID，
/// 服务端会将每个 (Id, Mesh) 对解析为独立实体。
/// 相比循环调用 `send_point`，这种方式大幅减少网络开销。
///
/// # 参数
/// * `points` - 点云数组，每个元素为 `[x, y, z]`
///
/// # 示例
/// ```no_run
/// use redra_client::client::send::send_point_cloud;
///
/// let cloud = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
/// send_point_cloud(&cloud).await.unwrap();
/// ```
pub async fn send_point_cloud(points: &[[f32; 3]]) -> Result<(), String> {
    let mut unit = generate_unit();

    for (i, point) in points.iter().enumerate() {
        // 自动生成 ID（从 1 开始，本地唯一即可）
        unit.objects.push(ExObject::from(i as u64 + 1));

        let p: Point = (point[0], point[1], point[2]).into();
        let mesh: ExMesh = p.into();
        unit.objects.push(ExObject::from(mesh));

        // 设置位置（Point 网格渲染为小 sphere，需要 Transform 定位）
        unit.objects.push(ExObject::from(ExTransform {
            x: point[0], y: point[1], z: point[2],
            rx: 0.0, ry: 0.0, rz: 0.0,
            sx: 1.0, sy: 1.0, sz: 1.0,
        }));
    }

    unit.send().await?;
    Ok(())
}

/// 发送线段（起点 → 终点）
///
/// 自动计算中点位置与朝向。起终点距离 < 1e-6 时静默跳过。
pub async fn send_line(
    x1: f32,
    y1: f32,
    z1: f32,
    x2: f32,
    y2: f32,
    z2: f32,
) -> Result<(), String> {
    let mut unit = generate_unit();

    let dx = x2 - x1;
    let dy = y2 - y1;
    let dz = z2 - z1;
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len < 1e-6 {
        return Ok(());
    }

    // Line mesh (server renders as Cylinder along Y-axis)
    let line: Line = (Point { x: x1, y: y1, z: z1 }, Point { x: x2, y: y2, z: z2 }).into();
    unit.objects.push(ExObject::from(ExMesh::from(line)));

    // Transform: position at midpoint, rotate Y-axis to direction
    let dir = Vector3::new(dx / len, dy / len, dz / len);
    let y = Vector3::y();
    let q = UnitQuaternion::rotation_between(&y, &dir)
        .unwrap_or(UnitQuaternion::identity());
    let (rx, ry, rz) = q.euler_angles();

    unit.objects.push(ExObject::from(ExTransform {
        x: (x1 + x2) / 2.0,
        y: (y1 + y2) / 2.0,
        z: (z1 + z2) / 2.0,
        rx, ry, rz,
        sx: 1.0, sy: 1.0, sz: 1.0,
    }));

    unit.send().await?;
    Ok(())
}

/// 发送球体
///
/// # 参数
/// * `x`, `y`, `z` — 球心世界坐标
/// * `radius` — 半径（必须 > 0）
pub async fn send_sphere(
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let point: Point = (x, y, z).into();
    let sphere: Sphere = (point, radius).into();
    let mesh: ExMesh = sphere.into();
    let object: ExObject = mesh.into();
    let _ = unit.set_object(object);
    unit.send().await?;
    Ok(())
}

/// 发送圆柱体（原点处，沿 Y 轴）
///
/// # 参数
/// * `radius` — 半径（必须 > 0）
/// * `height` — 高度（必须 > 0）
pub async fn send_cylinder(
    radius: f32,
    height: f32,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let cylinder: Cylinder = (radius, height).into();
    let mesh: ExMesh = cylinder.into();
    let object: ExObject = mesh.into();
    let _ = unit.set_object(object);
    unit.send().await?;
    Ok(())
}

/// 发送圆锥体（原点处，沿 Y 轴）
///
/// # 参数
/// * `radius` — 底面半径（必须 > 0）
/// * `height` — 高度（必须 > 0）
pub async fn send_cone(
    radius: f32,
    height: f32,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let cone: Cone = (radius, height).into();
    let mesh: ExMesh = cone.into();
    let object: ExObject = mesh.into();
    let _ = unit.set_object(object);
    unit.send().await?;
    Ok(())
}

/// 发送标签到指定对象
/// 
/// # 参数
/// * `target_id` - 目标对象的 ID
/// * `text` - 标签文本内容
/// 
/// # 示例
/// ```no_run
/// use redra_client::client::send::send_tag;
/// 
/// // 发送简单标签
/// send_tag(1, "Hello World").await.unwrap();
/// ```
pub async fn send_tag(
    _target_id: u64,
    text: impl Into<String>,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let tag = Tag::new(text);
    let object: ExObject = tag.into();
    let _ = unit.set_object(object);
    unit.send().await?;
    Ok(())
}

/// 发送带样式的标签
/// 
/// # 参数
/// * `target_id` - 目标对象的 ID
/// * `text` - 标签文本内容
/// * `style` - 标签样式配置
/// 
/// # 示例
/// ```no_run
/// use redra_client::client::send::send_tag_with_style;
/// use expto::rdmp::TagStyle;
/// 
/// let style = TagStyle::default_style()
///     .with_font_size(16.0)
///     .with_bg_color(0.2, 0.3, 0.8, 0.9)
///     .with_text_color(1.0, 1.0, 1.0, 1.0);
///     
/// send_tag_with_style(1, "Important", style).await.unwrap();
/// ```
pub async fn send_tag_with_style(
    _target_id: u64,
    text: impl Into<String>,
    style: TagStyle,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let tag = Tag::new(text).with_style(style);
    let object: ExObject = tag.into();
    let _ = unit.set_object(object);
    unit.send().await?;
    Ok(())
}

/// 发送一个包围盒（8 个角点）
///
/// 用于可视化聚类（cluster）的边界框。
/// 自动计算 AABB 中心并将实体定位到正确位置。
///
/// **约束**：每个维度（宽/高/深）必须 > 0.001，否则渲染端会拒绝该 mesh。
/// 对于退化包围盒（点共面/共线/单点），建议改用 `send_sphere` 或 `send_point`。
pub async fn send_cube(
    vertices: Vec<(f32, f32, f32)>,
) -> Result<(), String> {
    let mut unit = generate_unit();

    let mut min = [f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN];
    let points: Vec<Point> = vertices.iter().map(|&(x, y, z)| {
        min[0] = min[0].min(x); min[1] = min[1].min(y); min[2] = min[2].min(z);
        max[0] = max[0].max(x); max[1] = max[1].max(y); max[2] = max[2].max(z);
        Point { x, y, z }
    }).collect();

    let w = max[0] - min[0];
    let h = max[1] - min[1];
    let d = max[2] - min[2];
    let min_dim = crate::defaults::mesh_constraints::MIN_CUBE_DIMENSION;
    if w < min_dim || h < min_dim || d < min_dim {
        log::warn!(
            "Cube 维度退化 (w={:.4}, h={:.4}, d={:.4})，渲染端将拒绝此 mesh。\
             建议对退化包围盒改用 send_sphere 或 send_point。",
            w, h, d
        );
    }

    let cube = Cube { vertices: points };
    unit.objects.push(ExObject::from(ExMesh::from(cube)));

    let cx = (min[0] + max[0]) / 2.0;
    let cy = (min[1] + max[1]) / 2.0;
    let cz = (min[2] + max[2]) / 2.0;
    unit.objects.push(ExObject::from(ExTransform {
        x: cx, y: cy, z: cz,
        rx: 0.0, ry: 0.0, rz: 0.0,
        sx: 1.0, sy: 1.0, sz: 1.0,
    }));

    unit.send().await?;
    Ok(())
}

/// 发送带标签的包围盒（聚类用）
///
/// 同时发送包围盒几何体和文本标签，便于识别聚类。
/// 自动计算 AABB 中心并将实体定位到正确位置。
pub async fn send_cube_with_tag(
    vertices: Vec<(f32, f32, f32)>,
    text: impl Into<String>,
) -> Result<(), String> {
    let mut unit = generate_unit();

    let mut min = [f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN];
    let points: Vec<Point> = vertices.iter().map(|&(x, y, z)| {
        min[0] = min[0].min(x); min[1] = min[1].min(y); min[2] = min[2].min(z);
        max[0] = max[0].max(x); max[1] = max[1].max(y); max[2] = max[2].max(z);
        Point { x, y, z }
    }).collect();

    let cube = Cube { vertices: points };
    unit.objects.push(ExObject::from(ExMesh::from(cube)));

    let cx = (min[0] + max[0]) / 2.0;
    let cy = (min[1] + max[1]) / 2.0;
    let cz = (min[2] + max[2]) / 2.0;
    unit.objects.push(ExObject::from(ExTransform {
        x: cx, y: cy, z: cz,
        rx: 0.0, ry: 0.0, rz: 0.0,
        sx: 1.0, sy: 1.0, sz: 1.0,
    }));

    // 添加标签
    let tag = Tag::new(text);
    unit.objects.push(ExObject::from(tag));

    unit.send().await?;
    Ok(())
}

/// 批量发送带材质分组的点云
///
/// 每组点共享同一材质，不同组可使用不同颜色。
/// 每组生成一个独立的 Unit 消息。
///
/// # 参数
/// * `groups` - 切片，每个元素为 `(点云数组, 材质名)`
///
/// # 示例
/// ```no_run
/// use redra_client::send_point_cloud_grouped;
///
/// let ground = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
/// let obstacle = vec![[0.5, 1.0, 0.0]];
/// send_point_cloud_grouped(&[(&ground, "red"), (&obstacle, "blue")]).await.unwrap();
/// ```
pub async fn send_point_cloud_grouped(
    groups: &[(&[[f32; 3]], &str)],
) -> Result<(), String> {
    let link = get_link().await;
    let mut id_counter: u64 = 1;
    for &(points, material) in groups {
        let mut unit = generate_unit();
        for pos in points {
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
            unit.objects.push(ExObject { u_object: Some(UObject::MaterialId(material.to_string())) });
        }
        let buf = encode(&unit).map_err(|e| format!("{}", e))?;
        link.send(&buf).await?;
    }
    Ok(())
}

// ==================== 材质 & 控制 API ====================

/// 更新已有实体的材质
///
/// 发送 `CommandType::Update` 指令，修改指定 ID 实体的材质。
/// `material_id` 可以是预定义材质名（`"red"`, `"green"`, `"glass"`, `"metal"` 等），
/// 也可以是自定义材质 TOML 文件路径。
///
/// # 参数
/// * `entity_id` - 目标实体的 ID
/// * `material_id` - 材质标识符
///
/// # 示例
/// ```no_run
/// use redra_client::client::send::send_set_material;
/// send_set_material(1, "red").await.unwrap();
/// send_set_material(2, "glass").await.unwrap();
/// ```
pub async fn send_set_material(
    entity_id: u64,
    material_id: impl Into<String>,
) -> Result<(), String> {
    let mut unit = generate_unit();
    unit.set_update().unwrap();
    unit.objects.push(ExObject::from(entity_id));
    use expto::rdmp::ex_object::UObject;
    unit.objects.push(ExObject {
        u_object: Some(UObject::MaterialId(material_id.into())),
    });
    unit.send().await?;
    Ok(())
}

/// 删除指定 ID 的实体
///
/// 发送 `CommandType::Destroy` 指令，从当前帧中移除实体。
///
/// # 参数
/// * `entity_id` - 要删除的实体 ID
///
/// # 示例
/// ```no_run
/// use redra_client::client::send::send_destroy;
/// send_destroy(1).await.unwrap();
/// ```
pub async fn send_destroy(entity_id: u64) -> Result<(), String> {
    let mut unit = generate_unit();
    unit.set_destroy().unwrap();
    unit.objects.push(ExObject::from(entity_id));
    unit.send().await?;
    Ok(())
}
