use bevy::prelude::*;
use crate::{graph::material::{MaterialManager, PredefinedMaterial}};

/// 在引擎启动时初始化材质资源
pub fn initialize_materials(
    mut commands: Commands,
) {
    // 创建材质管理器
    let mut material_manager = MaterialManager::new();
    
    // 添加一些额外的自定义材质
    add_custom_materials(&mut material_manager);
    
    // 将材质管理器添加到全局资源中
    commands.insert_resource(material_manager);
    
    info!("材质系统初始化完成");
}

/// 添加自定义材质到管理器
fn add_custom_materials(manager: &mut MaterialManager) {
    // 添加金属质感材质
    manager.register_material(
        &"metal".to_string(),
        PredefinedMaterial::Standard(StandardMaterial {
            base_color: Color::srgb(0.7, 0.8, 0.9),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            ..default()
        })
    );
    
    // 添加发光材质
    manager.register_material(
        &"glow".to_string(),
        PredefinedMaterial::Standard(StandardMaterial {
            base_color: Color::srgb(0.1, 0.8, 0.5),
            emissive: LinearRgba::from(Color::srgb(0.1, 0.8, 0.5)) * 3.0,
            ..default()
        })
    );
    
    // 添加半透明材质
    manager.register_material(
        &"glass".to_string(),
        PredefinedMaterial::Standard(StandardMaterial {
            base_color: Color::srgba(0.5, 0.8, 0.9, 0.7),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.1,
            metallic: 0.5,
            ..default()
        })
    );
    
    // 添加粗糙材质
    manager.register_material(
        &"rough".to_string(),
        PredefinedMaterial::Standard(StandardMaterial {
            base_color: Color::srgb(0.5, 0.4, 0.3),
            perceptual_roughness: 0.9,
            ..default()
        })
    );
    
    // 添加橡胶质感材质
    manager.register_material(
        &"rubber".to_string(),
        PredefinedMaterial::Standard(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.2),
            perceptual_roughness: 0.8,
            metallic: 0.0,
            ..default()
        })
    );
    
    info!("自定义材质注册完成");
}