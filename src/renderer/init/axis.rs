use std::fs;

use bevy::prelude::*;
use crate::manager::materials::MaterialManager;
use crate::renderer::helpers;


pub fn axis_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_manager: Res<MaterialManager>,
    asset_server: Res<AssetServer>,
) {
    let config_path = "assets/init/default_scene.toml";

    match fs::read_to_string(config_path) {
        Ok(content) => {
            match toml::from_str::<crate::manager::data_flow::config_adapter::SceneConfig>(&content) {
                Ok(config) => {
                    if config.global.enabled {
                        log::info!("✅ [Renderer/Startup] 从 TOML 配置文件加载 {} 个测试实体", config.entities.len());
                        log::info!("   描述: {}", config.global.description);
                        
                        // 使用 helpers API 生成实体
                        for entity_config in &config.entities {
                            // 构建 proto 数据
                            let (proto_mesh, proto_transform) = 
                                crate::manager::data_flow::config_adapter::entity_to_proto_data(entity_config);
                            
                            // 根据颜色选择材质文件路径（使用 MaterialManager 的统一方法）
                            let material_path = material_manager.select_material_by_color(&entity_config.color);
                            
                            helpers::spawn_entity(
                                &mut commands,
                                &mut meshes,
                                &asset_server,
                                &material_manager,
                                &proto_mesh,
                                &proto_transform,
                                material_path,
                                &entity_config.name,
                            );
                        }
                    } else {
                        log::info!("⚠️ [Renderer/Startup] 测试场景已禁用，仅显示基础坐标轴");
                        default_axis(commands, meshes, asset_server, material_manager);
                    }
                }
                Err(e) => {
                    log::warn!("⚠️ [Renderer/Startup] TOML 解析失败，仅显示基础坐标轴: {}", e);
                    default_axis(commands, meshes, asset_server, material_manager);

                }
            }
        }
        Err(e) => {
            log::warn!("⚠️ [Renderer/Startup] 配置文件读取失败，仅显示基础坐标轴: {}", e);
            default_axis(commands, meshes, asset_server, material_manager);
        }
    }
}

/// 创建一个3D坐标轴系统，包含X、Y、Z三个方向的轴
/// X轴为红色，Y轴为绿色，Z轴为蓝色
pub fn default_axis(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    material_manager: Res<MaterialManager>,
) {
    let scale = 3.0; // 设定固定比例
    let axis_length = scale;
    let axis_radius = scale * 0.025;
    
    // 使用 helpers API 生成带箭头的坐标轴
    helpers::spawn_axis_with_arrow(
        &mut commands,
        &mut meshes,
        &asset_server,
        &material_manager,
        Vec3::ZERO,
        Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
        axis_length,
        axis_radius,
        "axis_x",
        "X_Axis",
    );
    
    helpers::spawn_axis_with_arrow(
        &mut commands,
        &mut meshes,
        &asset_server,
        &material_manager,
        Vec3::ZERO,
        Quat::IDENTITY,
        axis_length,
        axis_radius,
        "axis_y",
        "Y_Axis",
    );
    
    helpers::spawn_axis_with_arrow(
        &mut commands,
        &mut meshes,
        &asset_server,
        &material_manager,
        Vec3::ZERO,
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        axis_length,
        axis_radius,
        "axis_z",
        "Z_Axis",
    );
}