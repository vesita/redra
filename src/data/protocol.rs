use expto::rdmp::{CommandType, ExMesh, ExTransform, Unit, Tag, ex_object::UObject};

use crate::data::frame::InptoTransform;

// ==================== 基础对象解析 ====================

pub fn parse_object(unit: &Unit) -> Vec<UObject> {
    let mut res = Vec::new();
    for object in unit.objects.clone() {
        if let Some(object) = object.u_object {
            res.push(object);
        }
    }
    res
}

/// Trait：定义从 Unit 中提取特定类型对象的能力
pub trait ExtractFromUnit {
    fn type_name() -> &'static str;
    fn try_extract(u_object: &expto::rdmp::ex_object::UObject) -> Option<Self> where Self: Sized;
}

impl ExtractFromUnit for u64 {
    fn type_name() -> &'static str { "Id" }
    fn try_extract(u_object: &expto::rdmp::ex_object::UObject) -> Option<Self> {
        if let expto::rdmp::ex_object::UObject::Id(id) = u_object { Some(*id) } else { None }
    }
}

impl ExtractFromUnit for ExMesh {
    fn type_name() -> &'static str { "Mesh" }
    fn try_extract(u_object: &expto::rdmp::ex_object::UObject) -> Option<Self> {
        if let expto::rdmp::ex_object::UObject::Mesh(mesh) = u_object { Some(mesh.clone()) } else { None }
    }
}

impl ExtractFromUnit for ExTransform {
    fn type_name() -> &'static str { "Transform" }
    fn try_extract(u_object: &expto::rdmp::ex_object::UObject) -> Option<Self> {
        if let expto::rdmp::ex_object::UObject::Transform(transform) = u_object { Some(*transform) } else { None }
    }
}

impl ExtractFromUnit for String {
    fn type_name() -> &'static str { "MaterialId" }
    fn try_extract(u_object: &expto::rdmp::ex_object::UObject) -> Option<Self> {
        if let expto::rdmp::ex_object::UObject::MaterialId(id) = u_object { Some(id.clone()) } else { None }
    }
}

impl ExtractFromUnit for Tag {
    fn type_name() -> &'static str { "Tag" }
    fn try_extract(u_object: &expto::rdmp::ex_object::UObject) -> Option<Self> {
        if let expto::rdmp::ex_object::UObject::Tag(tag) = u_object { Some(tag.clone()) } else { None }
    }
}

pub fn extract_object<T: ExtractFromUnit>(unit: &Unit) -> Option<T> {
    let mut found = Vec::new();
    for obj in &unit.objects {
        if let Some(u_object) = &obj.u_object {
            if let Some(value) = T::try_extract(u_object) {
                found.push(value);
            }
        }
    }
    match found.len() {
        0 => None,
        1 => Some(found.into_iter().next().unwrap()),
        _ => {
            log::warn!("Unit 中包含 {} 个 {} 对象，使用第一个", found.len(), T::type_name());
            Some(found.into_iter().next().unwrap())
        }
    }
}

pub fn extract_id(unit: &Unit) -> Option<u64> { extract_object::<u64>(unit) }
pub fn extract_mesh(unit: &Unit) -> Option<ExMesh> { extract_object::<ExMesh>(unit) }
pub fn extract_transform(unit: &Unit) -> Option<ExTransform> { extract_object::<ExTransform>(unit) }
pub fn extract_material_id(unit: &Unit) -> Option<String> { extract_object::<String>(unit) }
pub fn extract_tag(unit: &Unit) -> Option<Tag> { extract_object::<Tag>(unit) }

/// 从 Unit 提取所有 Tag
pub fn extract_tags(unit: &Unit) -> Vec<Tag> {
    unit.objects.iter()
        .filter_map(|obj| {
            if let Some(expto::rdmp::ex_object::UObject::Tag(tag)) = &obj.u_object {
                Some(tag.clone())
            } else { None }
        })
        .collect()
}

pub fn parse_command(unit: &Unit) -> Option<CommandType> {
    match unit.command {
        Some(cmd) => CommandType::try_from(cmd.u_command).ok(),
        None => None,
    }
}

pub fn parse_timestamp(unit: &Unit) -> u64 {
    if let Some(stamp) = &unit.stamp { stamp.timestamp } else { 0 }
}

/// 将 ExTransform 转为内部 InptoTransform（欧拉角 → 四元数）
pub fn e2i_transform(transform: ExTransform) -> InptoTransform {
    InptoTransform::from(transform)
}

/// 将内部 InptoTransform 转为协议 ExTransform
pub fn i2e_transform(t: InptoTransform) -> ExTransform {
    // 四元数 → 欧拉角 (XYZ)
    let (rx, ry, rz) = quat_to_euler_xyz(t.rx, t.ry, t.rz, t.rw);
    ExTransform {
        x: t.tx, y: t.ty, z: t.tz,
        rx, ry, rz,
        sx: t.sx, sy: t.sy, sz: t.sz,
    }
}

/// 四元数转欧拉角 (XYZ)
fn quat_to_euler_xyz(qx: f32, qy: f32, qz: f32, qw: f32) -> (f32, f32, f32) {
    let sinr_cosp = 2.0 * (qw * qx + qy * qz);
    let cosr_cosp = 1.0 - 2.0 * (qx * qx + qy * qy);
    let rx = sinr_cosp.atan2(cosr_cosp);

    let sinp = 2.0 * (qw * qy - qz * qx);
    let ry = if sinp.abs() >= 1.0 {
        std::f32::consts::FRAC_PI_2.copysign(sinp)
    } else {
        sinp.asin()
    };

    let siny_cosp = 2.0 * (qw * qz + qx * qy);
    let cosy_cosp = 1.0 - 2.0 * (qy * qy + qz * qz);
    let rz = siny_cosp.atan2(cosy_cosp);

    (rx, ry, rz)
}

// ── 以下仅在 graph feature 下可用（依赖 bevy Transform）──

#[cfg(feature = "graph")]
use bevy::prelude::*;

#[cfg(feature = "graph")]
use crate::assets::materials::{MaterialManager, GenericMaterial};
#[cfg(feature = "graph")]
use crate::data::frame::Inpto;

/// 通过 redra_geo 管线在不同坐标轴约定间转换 bevy Transform
///
/// 注意：中间的 Transform3 使用等方缩放（f32），
/// 各向异性缩放信息会丢失（三个轴的平均值）。
#[cfg(feature = "graph")]
pub fn convert_bevy_transform(
    t: &Transform,
    from: redra_geo::axis::AxisConvention,
    to: redra_geo::axis::AxisConvention,
) -> Transform {
    let ext = i2e_transform(InptoTransform::from(*t));
    let t3 = redra_geo::convert::extransform_to_transform3(&ext);
    let converted = redra_geo::axis::convert_axis(&t3, from, to);
    let back_ext = redra_geo::convert::transform3_to_extransform(&converted);
    Transform::from(e2i_transform(back_ext))
}

#[cfg(feature = "graph")]
pub fn inpto_material_name(inpto: &Inpto, material_manager: &MaterialManager) -> String {
    if inpto.material.is_empty() {
        "default".to_string()
    } else {
        material_manager.resolve_material_id(&inpto.material)
            .unwrap_or(&inpto.material)
            .to_string()
    }
}

#[cfg(feature = "graph")]
pub fn inpto_material_path(inpto: &Inpto, material_manager: &MaterialManager) -> String {
    if inpto.material.is_empty() {
        "materials/default.toml".to_string()
    } else {
        material_manager.resolve_material_id(&inpto.material)
            .unwrap_or(&inpto.material)
            .to_string()
    }
}

#[cfg(feature = "graph")]
pub fn inpto_to_generic_material(inpto: &Inpto, material_manager: &MaterialManager, asset_server: &AssetServer) -> Handle<GenericMaterial> {
    let path = inpto_material_path(inpto, material_manager);
    material_manager.load_generic_material(&path, asset_server)
}
