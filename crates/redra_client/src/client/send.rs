use expto::prelude::*;
use expto::rdmp::auto::unit::generate_unit;
use expto::rdmp::{Cube, ExObject, ExMesh, Point, Cylinder, Cone, Tag, TagStyle};

use crate::client::link::get_link;

// 定义一个 trait 来扩展 Unit 的功能
pub trait AutoSend4Unit {
    async fn send(&self) -> Result<(), String>;
}

impl AutoSend4Unit for Unit { 
    async fn send(&self) -> Result<(), String> { 
        match encode(self) {
            Ok(buf) => {
                let link = get_link();
                link.send(&buf).await?;
            },
            Err(e) => return Err(format!("{}", e)),
        }
        Ok(())
    }
}


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

pub async fn send_line(
    x1: f32,
    y1: f32,
    z1: f32,
    x2: f32,
    y2: f32,
    z2: f32,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let point1: Point = (x1, y1, z1).into();
    let point2: Point = (x2, y2, z2).into();
    let line: Line = (point1, point2).into();
    let mesh: ExMesh = line.into();
    let object: ExObject = mesh.into();
    let _ = unit.set_object(object);
    unit.send().await?;
    Ok(())
}

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
    target_id: u64,
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
    target_id: u64,
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
/// 顶点顺序约定：底面 4 点逆时针 (0,1,2,3)，顶面 4 点对应 (4,5,6,7)。
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

    let cube = Cube { vertices: points };
    unit.objects.push(ExObject::from(ExMesh::from(cube)));

    // 将实体定位到 AABB 中心，使 Cuboid 渲染在正确位置
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

    // 将实体定位到 AABB 中心
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
