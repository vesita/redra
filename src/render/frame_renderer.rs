use std::collections::HashMap;

use bevy::prelude::*;

use crate::data::frame::{FrameManager, Inpto};
use crate::assets::materials::MaterialManager;
use crate::render::interaction::picking::PickableEntity;

/// 隐藏标记组件
#[derive(Component, Default)]
pub struct Hidden;

/// 实体映射资源
#[derive(Resource, Default)]
pub struct EntityMap {
    pub map: HashMap<u64, Entity>,
}

impl EntityMap {
    pub fn clear(&mut self) { self.map.clear(); }
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
    mut entity_map: ResMut<EntityMap>,
    pickable_check_query: Query<(Entity, &Name, &PickableEntity)>,
    hidden_query: Query<(), With<Hidden>>,
) {
    let Some(keyframe) = frame_manager.get_current_keyframe() else {
        log::debug!("当前无可用帧数据");
        return;
    };

    log::debug!("渲染第 {} 帧，包含 {} 个实体", frame_manager.current_frame_index(), keyframe.entity_count());

    let current_entity_ids: HashMap<u64, &Inpto> = keyframe.iter_entities().collect();
    cleanup_removed_entities(&mut commands, &current_entity_ids, &mut entity_map.map);

    for (&entity_id, inpto) in &current_entity_ids {
        if let Some(&entity) = entity_map.map.get(&entity_id) {
            update_entity_transform(&mut commands, entity, &inpto.transform, &hidden_query);
        } else {
            let new_entity = spawn_entity_from_inpto(&mut commands, &mut meshes, &asset_server, &material_manager, inpto, entity_id);
            entity_map.map.insert(entity_id, new_entity);
            log::info!("创建新实体 {} (名称: {})", entity_id, inpto.name());
        }
    }

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
) -> Entity {
    let mesh_handle = crate::render::conversion::proto_mesh_to_bevy(meshes, &inpto.mesh)
        .unwrap_or_else(|| { log::warn!("网格转换失败，使用备用球体 (实体 {})", entity_id); Mesh3d(meshes.add(Sphere::new(0.1))) });

    let material_handle = material_manager.load_generic_material(&inpto.material_path(), asset_server);

    commands.spawn((
        mesh_handle,
        crate::render::GenericMaterial3d(material_handle.clone()),
        inpto.transform,
        Name::new(format!("FrameEntity_{}", entity_id)),
        Pickable::default(),
        PickableEntity { entity_id },
        crate::render::interaction::picking::DynamicEntity,
    ))
    .observe(crate::render::interaction::picking::handle_dynamic_entity_pick)
    .id()
}

fn update_entity_transform(
    commands: &mut Commands,
    entity: Entity,
    transform: &Transform,
    hidden_query: &Query<(), With<Hidden>>,
) {
    let has_hidden = hidden_query.get(entity).is_ok();
    commands.entity(entity).insert(*transform);
    if has_hidden { commands.entity(entity).insert(Hidden); }
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
            commands.entity(entity).despawn();
        }
    }
    for entity_id in removed_ids { entity_map.remove(&entity_id); }
}
