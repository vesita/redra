use bevy::prelude::*;
use expto::rdmp::{CommandType, ExTransform, Unit, ex_object::UObject};

use crate::{manager::{data::frame::Inpto, materials::MaterialManager}, renderer::GenericMaterial};

// 主要是简化expto的解析语句

pub fn parse_object(unit: &Unit) -> Vec<UObject> { 
    let mut res = Vec::new();
    for object in unit.objects.clone() { 
        match object.u_object {
            Some(object) => {
                res.push(object);
            }
            None => {

            },
        }
    }
    res
}

pub fn parse_command(unit: &Unit) -> Option<CommandType> { 
    match unit.command {
        Some(cmd) => {
            CommandType::try_from(cmd.u_command).ok()
        }
        None => {
            None
        }
    }
}

pub fn parse_timestamp(unit: &Unit) -> u64 { 
     if let Some(stamp) = &unit.stamp {
        return stamp.timestamp;
     }
     0
}

pub fn e2i_transform(transform: ExTransform) -> Transform { 
    Transform {
        translation: Vec3::new(transform.x, transform.y, transform.z),
        rotation: Quat::from_euler(EulerRot::XYZ, transform.rx, transform.ry, transform.rz),
        scale: Vec3::new(transform.sx, transform.sy, transform.sz),
    }
}


/// 从 Inpto 解析材质名称
pub fn inpto_material_name(inpto: &Inpto, material_manager: &MaterialManager) -> String {
    if inpto.material.is_empty() {
        "default".to_string()
    } else {
        material_manager.resolve_material_id(&inpto.material)
            .unwrap_or(&inpto.material)
            .to_string()
    }
}


/// 从 Inpto 解析材质文件路径
pub fn inpto_material_path(inpto: &Inpto, material_manager: &MaterialManager) -> String {
    if inpto.material.is_empty() {
        "materials/default.toml".to_string()
    } else {
        // 尝试解析为 material_id，如果未找到则直接使用
        material_manager.resolve_material_id(&inpto.material)
            .unwrap_or(&inpto.material)
            .to_string()
    }
}


/// 从 Inpto 解析并加载 GenericMaterial
pub fn inpto_to_generic_material(inpto: &Inpto, material_manager: &MaterialManager, asset_server: &AssetServer) -> Handle<GenericMaterial> {
    let path = inpto_material_path(inpto, material_manager);
    material_manager.load_generic_material(&path, asset_server)
}

