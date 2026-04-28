use bevy::prelude::*;
use expto::rdmp::{CommandType, ExMesh, ExTransform, Unit, Tag, ex_object::UObject};

use crate::data::frame::Inpto;
use crate::assets::materials::MaterialManager;
use crate::assets::materials::GenericMaterial;

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

pub fn parse_command(unit: &Unit) -> Option<CommandType> {
    match unit.command {
        Some(cmd) => CommandType::try_from(cmd.u_command).ok(),
        None => None,
    }
}

pub fn parse_timestamp(unit: &Unit) -> u64 {
    if let Some(stamp) = &unit.stamp { stamp.timestamp } else { 0 }
}

pub fn e2i_transform(transform: ExTransform) -> Transform {
    Transform {
        translation: Vec3::new(transform.x, transform.y, transform.z),
        rotation: Quat::from_euler(EulerRot::XYZ, transform.rx, transform.ry, transform.rz),
        scale: Vec3::new(transform.sx, transform.sy, transform.sz),
    }
}

// ==================== Inpto 材质工具 ====================

pub fn inpto_material_name(inpto: &Inpto, material_manager: &MaterialManager) -> String {
    if inpto.material.is_empty() {
        "default".to_string()
    } else {
        material_manager.resolve_material_id(&inpto.material)
            .unwrap_or(&inpto.material)
            .to_string()
    }
}

pub fn inpto_material_path(inpto: &Inpto, material_manager: &MaterialManager) -> String {
    if inpto.material.is_empty() {
        "materials/default.toml".to_string()
    } else {
        material_manager.resolve_material_id(&inpto.material)
            .unwrap_or(&inpto.material)
            .to_string()
    }
}

pub fn inpto_to_generic_material(inpto: &Inpto, material_manager: &MaterialManager, asset_server: &AssetServer) -> Handle<GenericMaterial> {
    let path = inpto_material_path(inpto, material_manager);
    material_manager.load_generic_material(&path, asset_server)
}
