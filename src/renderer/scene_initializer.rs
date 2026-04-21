use bevy::prelude::*;

use crate::manager::materials::MaterialManager;
use crate::renderer::helpers;

/// 静态场景初始化插件
/// 
/// 职责：
/// - 在 Startup 阶段从 TOML 配置文件加载静态场景
/// - 将配置转换为 Unit 协议数据
/// - 通过 FrameManager 的标准流程生成持久化的 Bevy 实体
pub struct SceneInitializerPlugin;

impl Plugin for SceneInitializerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_static_scene);
    }
}

/// 初始化静态场景
fn initialize_static_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    material_manager: Res<MaterialManager>,
) {
    let config_path = "assets/init/default_scene.toml";

    log::info!("开始加载静态场景配置...");

    match expto::config::load_static_scene_config(config_path) {
        Ok(config) => {
            if config.global.enabled {
                log::info!(
                    "从 TOML 配置文件加载 {} 个静态实体",
                    config.entities.len()
                );
                log::info!("   描述: {}", config.global.description);

                // 为每个实体生成 Unit 并转换为 Inpto
                let mut keyframe = crate::manager::data::frame::KeyFrame::new(0);
                
                for (idx, entity_config) in config.entities.iter().enumerate() {
                    // 生成唯一的 entity_id（从1开始，避免与动态实体冲突）
                    let entity_id = (idx + 1) as u64;
                    
                    // 使用 expto 的 config_to_unit 将配置转换为 Unit 协议数据
                    let unit = expto::config::config_to_unit(entity_config, entity_id);
                    
                    // 通过 KeyFrame 的标准流程将 Unit 转换为 Inpto
                    keyframe.update(&unit);
                    
                    log::debug!(
                        "转换静态实体: {} (ID: {})",
                        entity_config.name,
                        entity_id
                    );
                }

                // 渲染所有静态实体
                render_static_entities(
                    &mut commands,
                    &mut meshes,
                    &asset_server,
                    &material_manager,
                    &keyframe,
                );

                log::info!("静态场景加载完成");
            } else {
                log::info!("静态场景已禁用，仅显示基础坐标轴");
                spawn_default_axes(&mut commands, &mut meshes, &asset_server, &material_manager);
            }
        }
        Err(e) => {
            log::warn!("配置文件加载失败，使用默认坐标轴: {}", e);
            spawn_default_axes(&mut commands, &mut meshes, &asset_server, &material_manager);
        }
    }
}

/// 渲染静态实体（复用 FrameRenderer 和 parser 的逻辑）
fn render_static_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    asset_server: &AssetServer,
    material_manager: &MaterialManager,
    keyframe: &crate::manager::data::frame::KeyFrame,
) {
    use crate::manager::data_flow::parser;
    
    for (entity_id, inpto) in keyframe.iter_entities() {
        // 使用 conversion API 转换 Mesh
        let mesh_handle = crate::renderer::conversion::proto_mesh_to_bevy(meshes, &inpto.mesh)
            .unwrap_or_else(|| {
                log::warn!("网格转换失败，使用备用球体 (实体 {})", entity_id);
                Mesh3d(meshes.add(Sphere::new(0.1)))
            });

        // 使用 parser API 加载材质
        let material_handle = parser::inpto_to_generic_material(inpto, material_manager, asset_server);

        // 生成实体
        commands.spawn((
            mesh_handle,
            crate::renderer::GenericMaterial3d(material_handle),
            inpto.transform,
            Name::new(format!("StaticEntity_{}", entity_id)),
        ));

        log::debug!(
            "静态实体 {} 生成成功 (名称: {}, 材质: {})",
            entity_id,
            inpto.name(),
            parser::inpto_material_path(inpto, material_manager)
        );
    }
}

/// 生成默认的三轴坐标系（简化版）
fn spawn_default_axes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    asset_server: &AssetServer,
    material_manager: &MaterialManager,
) {
    use expto::rdmp::{ExMesh, ExTransform};
    use expto::rdmp::mesh::ex_mesh::UMesh;

    let axis_length = 3.0;
    let axis_radius = 0.075;
    let cone_radius = 0.15;
    let cone_height = 0.3;

    // ==================== X轴 ====================
    
    // X轴 - 红色圆柱体（中心在 x=1.5，沿X轴方向）
    let x_cylinder_mesh = ExMesh {
        u_mesh: Some(UMesh::Cylinder(expto::rdmp::Cylinder {
            radius: axis_radius,
            height: axis_length,
        })),
    };

    let x_cylinder_transform = ExTransform {
        x: axis_length / 2.0,  // 1.5
        y: 0.0,
        z: 0.0,
        rx: 0.0,
        ry: 0.0,
        rz: -std::f32::consts::FRAC_PI_2,  // 绕Z轴旋转-90度，使圆柱沿X轴
        sx: 1.0,
        sy: 1.0,
        sz: 1.0,
    };

    helpers::spawn_entity(
        commands,
        meshes,
        asset_server,
        material_manager,
        &x_cylinder_mesh,
        &x_cylinder_transform,
        "materials/base/red.toml",
        "X_Axis_Cylinder",
    );

    // X轴 - 红色圆锥（在 x=3.0 处，箭头指向正X方向）
    let x_cone_mesh = ExMesh {
        u_mesh: Some(UMesh::Cone(expto::rdmp::Cone {
            radius: cone_radius,
            height: cone_height,
        })),
    };

    let x_cone_transform = ExTransform {
        x: axis_length,  // 3.0
        y: 0.0,
        z: 0.0,
        rx: 0.0,
        ry: 0.0,
        rz: -std::f32::consts::FRAC_PI_2,  // 与圆柱体相同的旋转
        sx: 1.0,
        sy: 1.0,
        sz: 1.0,
    };

    helpers::spawn_entity(
        commands,
        meshes,
        asset_server,
        material_manager,
        &x_cone_mesh,
        &x_cone_transform,
        "materials/base/red.toml",
        "X_Axis_Cone",
    );

    // ==================== Y轴 ====================
    
    // Y轴 - 绿色圆柱体（中心在 y=1.5，沿Y轴方向）
    let y_cylinder_mesh = ExMesh {
        u_mesh: Some(UMesh::Cylinder(expto::rdmp::Cylinder {
            radius: axis_radius,
            height: axis_length,
        })),
    };

    let y_cylinder_transform = ExTransform {
        x: 0.0,
        y: axis_length / 2.0,  // 1.5
        z: 0.0,
        rx: 0.0,
        ry: 0.0,
        rz: 0.0,  // 无需旋转，默认沿Y轴
        sx: 1.0,
        sy: 1.0,
        sz: 1.0,
    };

    helpers::spawn_entity(
        commands,
        meshes,
        asset_server,
        material_manager,
        &y_cylinder_mesh,
        &y_cylinder_transform,
        "materials/base/green.toml",
        "Y_Axis_Cylinder",
    );

    // Y轴 - 绿色圆锥（在 y=3.0 处，箭头指向正Y方向）
    let y_cone_mesh = ExMesh {
        u_mesh: Some(UMesh::Cone(expto::rdmp::Cone {
            radius: cone_radius,
            height: cone_height,
        })),
    };

    let y_cone_transform = ExTransform {
        x: 0.0,
        y: axis_length,  // 3.0
        z: 0.0,
        rx: 0.0,
        ry: 0.0,
        rz: 0.0,  // 无需旋转
        sx: 1.0,
        sy: 1.0,
        sz: 1.0,
    };

    helpers::spawn_entity(
        commands,
        meshes,
        asset_server,
        material_manager,
        &y_cone_mesh,
        &y_cone_transform,
        "materials/base/green.toml",
        "Y_Axis_Cone",
    );

    // ==================== Z轴 ====================
    
    // Z轴 - 蓝色圆柱体（中心在 z=1.5，沿Z轴方向）
    let z_cylinder_mesh = ExMesh {
        u_mesh: Some(UMesh::Cylinder(expto::rdmp::Cylinder {
            radius: axis_radius,
            height: axis_length,
        })),
    };

    let z_cylinder_transform = ExTransform {
        x: 0.0,
        y: 0.0,
        z: axis_length / 2.0,  // 1.5
        rx: std::f32::consts::FRAC_PI_2,  // 绕X轴旋转90度，使圆柱沿Z轴
        ry: 0.0,
        rz: 0.0,
        sx: 1.0,
        sy: 1.0,
        sz: 1.0,
    };

    helpers::spawn_entity(
        commands,
        meshes,
        asset_server,
        material_manager,
        &z_cylinder_mesh,
        &z_cylinder_transform,
        "materials/base/blue.toml",
        "Z_Axis_Cylinder",
    );

    // Z轴 - 蓝色圆锥（在 z=3.0 处，箭头指向正Z方向）
    let z_cone_mesh = ExMesh {
        u_mesh: Some(UMesh::Cone(expto::rdmp::Cone {
            radius: cone_radius,
            height: cone_height,
        })),
    };

    let z_cone_transform = ExTransform {
        x: 0.0,
        y: 0.0,
        z: axis_length,  // 3.0
        rx: std::f32::consts::FRAC_PI_2,  // 与圆柱体相同的旋转
        ry: 0.0,
        rz: 0.0,
        sx: 1.0,
        sy: 1.0,
        sz: 1.0,
    };

    helpers::spawn_entity(
        commands,
        meshes,
        asset_server,
        material_manager,
        &z_cone_mesh,
        &z_cone_transform,
        "materials/base/blue.toml",
        "Z_Axis_Cone",
    );

    log::info!("✅ 默认坐标轴生成完成（包含圆柱体和圆锥箭头）");
}
