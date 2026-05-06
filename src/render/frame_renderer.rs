use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use expto::rdmp::mesh::ex_mesh::UMesh;

use crate::data::frame::{FrameManager, Inpto};
use crate::assets::materials::MaterialManager;
use crate::render::interaction::picking::PickableEntity;
use crate::render::coord_system::{Handedness, apply_handedness};

/// 隐藏标记组件
#[derive(Component, Default)]
pub struct Hidden;

/// 实体映射资源
#[derive(Resource, Default)]
pub struct EntityMap {
    pub map: HashMap<u64, Entity>,
    pub points_entity: Option<Entity>,
    pub points_mesh: Option<Handle<Mesh>>,
    /// 缓存上一帧的点云位置，用于 dirty check
    cached_positions: Vec<[f32; 3]>,
}

impl EntityMap {
    pub fn clear(&mut self) {
        self.map.clear();
        self.points_entity = None;
        self.points_mesh = None;
        self.cached_positions.clear();
    }
}

/// 帧渲染器插件
pub struct FrameRendererPlugin;

impl Plugin for FrameRendererPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityMap>()
            .add_systems(Update, render_current_frame);
    }
}

/// 渲染当前帧的所有实体
fn render_current_frame(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    material_manager: Res<MaterialManager>,
    frame_manager: Res<FrameManager>,
    handedness: Res<Handedness>,
    mut entity_map: ResMut<EntityMap>,
    pickable_check_query: Query<(Entity, &Name, &PickableEntity)>,
    hidden_query: Query<(), With<Hidden>>,
) {
    let Some(keyframe) = frame_manager.get_current_keyframe() else {
        log::debug!("当前无可用帧数据");
        return;
    };

    log::debug!("渲染第 {} 帧，包含 {} 个实体", frame_manager.current_frame_index(), keyframe.entity_count());

    // 分离 Point 和非 Point 实体
    let mut points: Vec<Vec3> = Vec::new();
    let mut non_point_ids: HashMap<u64, &Inpto> = HashMap::new();

    for (entity_id, inpto) in keyframe.iter_entities() {
        if matches!(inpto.mesh.u_mesh, Some(UMesh::Point(_))) {
            points.push(inpto.transform.translation);
        } else {
            non_point_ids.insert(entity_id, inpto);
        }
    }

    cleanup_removed_entities(&mut commands, &non_point_ids, &mut entity_map.map);

    for (&entity_id, inpto) in &non_point_ids {
        if let Some(&entity) = entity_map.map.get(&entity_id) {
            update_entity_transform(&mut commands, entity, &inpto.transform, *handedness, &hidden_query);
        } else {
            let new_entity = spawn_entity_from_inpto(&mut commands, &mut meshes, &asset_server, &material_manager, inpto, entity_id, *handedness);
            entity_map.map.insert(entity_id, new_entity);
            log::info!("创建新实体 {} (名称: {})", entity_id, inpto.name());
        }
    }

    // 聚合所有 Point 为单个 PointList mesh
    update_aggregated_points(&mut commands, &mut meshes, &asset_server, &material_manager, &mut entity_map, &points, *handedness);

    log::debug!("当前可拾取实体数量: {}", pickable_check_query.iter().count());
    for (entity, name, pickable) in pickable_check_query.iter() {
        log::debug!("实体 {:?}: {} (PickableEntity ID: {})", entity, name.as_str(), pickable.entity_id);
    }
}

fn spawn_entity_from_inpto(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    asset_server: &AssetServer,
    material_manager: &MaterialManager,
    inpto: &Inpto,
    entity_id: u64,
    handedness: Handedness,
) -> Entity {
    let mesh_handle = crate::render::conversion::proto_mesh_to_bevy(meshes, &inpto.mesh)
        .unwrap_or_else(|| { log::warn!("网格转换失败，使用备用球体 (实体 {})", entity_id); Mesh3d(meshes.add(Sphere::new(0.1))) });

    let material_handle = material_manager.load_generic_material(&inpto.material_path(), asset_server);
    let render_transform = apply_handedness(inpto.transform, handedness);

    commands.spawn((
        mesh_handle,
        crate::render::GenericMaterial3d(material_handle.clone()),
        render_transform,
        Name::new(format!("FrameEntity_{}", entity_id)),
        Pickable::default(),
        PickableEntity { entity_id },
        crate::render::interaction::picking::DynamicEntity,
    ))
    .observe(crate::render::interaction::picking::handle_dynamic_entity_pick)
    .id()
}

/// 聚合所有 Point 位置为单个 PointList mesh，1 次 draw call 渲染
fn update_aggregated_points(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    asset_server: &AssetServer,
    material_manager: &MaterialManager,
    entity_map: &mut EntityMap,
    points: &[Vec3],
    handedness: Handedness,
) {
    if points.is_empty() {
        if let Some(entity) = entity_map.points_entity.take() {
            commands.entity(entity).despawn();
        }
        entity_map.points_mesh = None;
        entity_map.cached_positions.clear();
        return;
    }

    let positions: Vec<[f32; 3]> = points.iter().map(|p| {
        let converted = apply_handedness(Transform::from_translation(*p), handedness);
        [converted.translation.x, converted.translation.y, converted.translation.z]
    }).collect();

    // dirty check：点数据未变时跳过 mesh 重建
    if positions == entity_map.cached_positions {
        return;
    }
    entity_map.cached_positions.clone_from(&positions);

    let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; positions.len()];
    let mut mesh = Mesh::new(PrimitiveTopology::PointList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    // 复用已有 mesh handle，避免每帧重新分配
    if let Some(handle) = &entity_map.points_mesh {
        if let Some(mesh_ref) = meshes.get_mut(handle) {
            *mesh_ref = mesh;
            return;
        }
    }

    let handle = meshes.add(mesh);
    let material = material_manager.load_generic_material("materials/mesh_types/point.toml", asset_server);
    let entity = commands.spawn((
        Mesh3d(handle.clone()),
        crate::render::GenericMaterial3d(material),
        Transform::default(),
        Name::new("AggregatedPoints"),
    )).id();
    entity_map.points_entity = Some(entity);
    entity_map.points_mesh = Some(handle);
}

fn update_entity_transform(
    commands: &mut Commands,
    entity: Entity,
    transform: &Transform,
    handedness: Handedness,
    hidden_query: &Query<(), With<Hidden>>,
) {
    // 实体可能在同一帧被其他系统（如文件加载）despawn，需要先检查有效性
    let Ok(mut ec) = commands.get_entity(entity) else { return };
    let render_transform = apply_handedness(*transform, handedness);
    let has_hidden = hidden_query.get(entity).is_ok();
    ec.insert(render_transform);
    if has_hidden {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.insert(Hidden);
        }
    }
}

fn cleanup_removed_entities(
    commands: &mut Commands,
    current_entity_ids: &HashMap<u64, &Inpto>,
    entity_map: &mut HashMap<u64, Entity>,
) {
    let mut removed_ids = Vec::new();
    for (&entity_id, &entity) in entity_map.iter() {
        if !current_entity_ids.contains_key(&entity_id) {
            removed_ids.push(entity_id);
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
    for entity_id in removed_ids { entity_map.remove(&entity_id); }
}
