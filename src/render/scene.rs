use bevy::prelude::*;

use crate::assets::materials::MaterialManager;
use crate::data::protocol;
use crate::render::helpers;
use crate::render::interaction::picking::StaticEntity;
use crate::render::coord_system::{CoordSystem, apply_coord_system, StaticSceneEntity};

/// 静态场景初始化插件
pub struct SceneInitializerPlugin;

impl Plugin for SceneInitializerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_static_scene);
        app.add_systems(Update, rerender_static_on_handedness_change);
    }
}

fn initialize_static_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    material_manager: Res<MaterialManager>,
    handedness: Res<CoordSystem>,
) {
    let config_path = "assets/init/default_scene.toml";
    log::info!("开始加载静态场景配置...");

    match expto::config::load_static_scene_config(config_path) {
        Ok(config) => {
            if config.global.enabled {
                log::info!("从 TOML 配置文件加载 {} 个静态实体", config.entities.len());
                let mut keyframe = crate::data::frame::KeyFrame::new(0);

                for (idx, entity_config) in config.entities.iter().enumerate() {
                    let entity_id = (idx + 1) as u64;
                    let unit = expto::config::config_to_unit(entity_config, entity_id);
                    keyframe.update(&unit);
                }

                render_static_entities(&mut commands, &mut meshes, &asset_server, &material_manager, &keyframe, *handedness);
                log::info!("静态场景加载完成");
            } else {
                log::info!("静态场景已禁用，仅显示基础坐标轴");
                spawn_default_axes(&mut commands, &mut meshes, &asset_server, &material_manager, *handedness);
            }
        }
        Err(e) => {
            log::warn!("配置文件加载失败，使用默认坐标轴: {}", e);
            spawn_default_axes(&mut commands, &mut meshes, &asset_server, &material_manager, *handedness);
        }
    }
}

fn render_static_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    asset_server: &AssetServer,
    material_manager: &MaterialManager,
    keyframe: &crate::data::frame::KeyFrame,
    handedness: CoordSystem,
) {
    for (entity_id, inpto) in keyframe.iter_entities() {
        let mesh_handle = crate::render::conversion::proto_mesh_to_bevy(meshes, &inpto.mesh)
            .unwrap_or_else(|| { log::warn!("网格转换失败，使用备用球体 (实体 {})", entity_id); Mesh3d(meshes.add(Sphere::new(0.1))) });
        let material_handle = protocol::inpto_to_generic_material(inpto, material_manager, asset_server);
        let render_transform = apply_coord_system(inpto.transform, handedness);

        commands.spawn((
            mesh_handle,
            crate::render::GenericMaterial3d(material_handle),
            render_transform,
            Name::new(format!("StaticEntity_{}", entity_id)),
            StaticEntity,
            StaticSceneEntity,
        ));
    }
}

fn spawn_default_axes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    asset_server: &AssetServer,
    material_manager: &MaterialManager,
    handedness: CoordSystem,
) {
    use expto::rdmp::{ExMesh, ExTransform, mesh::ex_mesh::UMesh};

    let axis_length = 3.0;
    let axis_radius = 0.075;
    let cone_radius = 0.15;
    let cone_height = 0.3;

    // X轴
    helpers::spawn_entity(commands, meshes, asset_server, material_manager,
        &ExMesh { u_mesh: Some(UMesh::Cylinder(expto::rdmp::Cylinder { radius: axis_radius, height: axis_length })) },
        &ExTransform { x: axis_length / 2.0, y: 0.0, z: 0.0, rx: 0.0, ry: 0.0, rz: -std::f32::consts::FRAC_PI_2, sx: 1.0, sy: 1.0, sz: 1.0 },
        "materials/base/red.toml", "X_Axis_Cylinder", handedness);
    helpers::spawn_entity(commands, meshes, asset_server, material_manager,
        &ExMesh { u_mesh: Some(UMesh::Cone(expto::rdmp::Cone { radius: cone_radius, height: cone_height })) },
        &ExTransform { x: axis_length, y: 0.0, z: 0.0, rx: 0.0, ry: 0.0, rz: -std::f32::consts::FRAC_PI_2, sx: 1.0, sy: 1.0, sz: 1.0 },
        "materials/base/red.toml", "X_Axis_Cone", handedness);

    // Y轴
    helpers::spawn_entity(commands, meshes, asset_server, material_manager,
        &ExMesh { u_mesh: Some(UMesh::Cylinder(expto::rdmp::Cylinder { radius: axis_radius, height: axis_length })) },
        &ExTransform { x: 0.0, y: axis_length / 2.0, z: 0.0, rx: 0.0, ry: 0.0, rz: 0.0, sx: 1.0, sy: 1.0, sz: 1.0 },
        "materials/base/green.toml", "Y_Axis_Cylinder", handedness);
    helpers::spawn_entity(commands, meshes, asset_server, material_manager,
        &ExMesh { u_mesh: Some(UMesh::Cone(expto::rdmp::Cone { radius: cone_radius, height: cone_height })) },
        &ExTransform { x: 0.0, y: axis_length, z: 0.0, rx: 0.0, ry: 0.0, rz: 0.0, sx: 1.0, sy: 1.0, sz: 1.0 },
        "materials/base/green.toml", "Y_Axis_Cone", handedness);

    // Z轴
    helpers::spawn_entity(commands, meshes, asset_server, material_manager,
        &ExMesh { u_mesh: Some(UMesh::Cylinder(expto::rdmp::Cylinder { radius: axis_radius, height: axis_length })) },
        &ExTransform { x: 0.0, y: 0.0, z: axis_length / 2.0, rx: std::f32::consts::FRAC_PI_2, ry: 0.0, rz: 0.0, sx: 1.0, sy: 1.0, sz: 1.0 },
        "materials/base/blue.toml", "Z_Axis_Cylinder", handedness);
    helpers::spawn_entity(commands, meshes, asset_server, material_manager,
        &ExMesh { u_mesh: Some(UMesh::Cone(expto::rdmp::Cone { radius: cone_radius, height: cone_height })) },
        &ExTransform { x: 0.0, y: 0.0, z: axis_length, rx: std::f32::consts::FRAC_PI_2, ry: 0.0, rz: 0.0, sx: 1.0, sy: 1.0, sz: 1.0 },
        "materials/base/blue.toml", "Z_Axis_Cone", handedness);
}

/// 坐标系变更时重新渲染所有静态实体
fn rerender_static_on_handedness_change(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    material_manager: Res<MaterialManager>,
    handedness: Res<CoordSystem>,
    static_entities: Query<Entity, With<StaticSceneEntity>>,
    mut prev: Local<CoordSystem>,
) {
    if *prev == *handedness {
        return;
    }
    *prev = *handedness;

    log::info!("坐标系手性变更，重新渲染静态实体");

    for entity in static_entities.iter() {
        commands.entity(entity).despawn();
    }

    let config_path = "assets/init/default_scene.toml";
    match expto::config::load_static_scene_config(config_path) {
        Ok(config) => {
            if config.global.enabled {
                let mut keyframe = crate::data::frame::KeyFrame::new(0);
                for (idx, entity_config) in config.entities.iter().enumerate() {
                    let entity_id = (idx + 1) as u64;
                    let unit = expto::config::config_to_unit(entity_config, entity_id);
                    keyframe.update(&unit);
                }
                render_static_entities(&mut commands, &mut meshes, &asset_server, &material_manager, &keyframe, *handedness);
            } else {
                spawn_default_axes(&mut commands, &mut meshes, &asset_server, &material_manager, *handedness);
            }
        }
        Err(e) => {
            log::warn!("配置文件加载失败，使用默认坐标轴: {}", e);
            spawn_default_axes(&mut commands, &mut meshes, &asset_server, &material_manager, *handedness);
        }
    }
}
