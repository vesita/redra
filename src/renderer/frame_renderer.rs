use std::collections::HashMap;

use bevy::prelude::*;

use crate::{manager::{
    data::frame::{FrameManager, Inpto},
    materials::MaterialManager,
}, renderer::interaction::picking::handle_dynamic_entity_pick};
use crate::renderer::conversion;
use crate::renderer::interaction::picking::{PickableEntity, DynamicEntity};

/// 选中标记组件（用于标识用户选中的实体）
#[derive(Component, Default)]
pub struct Selected;

/// 隐藏标记组件（用于标识被用户隐藏的实体）
#[derive(Component, Default)]
pub struct Hidden;

/// 实体映射资源（用于跟踪帧数据生成的实体）
#[derive(Resource, Default)]
pub struct EntityMap {
    pub map: HashMap<u64, Entity>,
}

impl EntityMap {
    /// 清空所有实体映射
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

/// Frame渲染器插件
/// 
/// 职责：
/// - 订阅 FrameManager 的帧数据
/// - 将 KeyFrame 中的 Inpto 数据转换为 Bevy 实体
/// - 管理实体的生命周期（spawn/update/despawn）
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
    // 获取当前关键帧数据
    let Some(keyframe) = frame_manager.get_current_keyframe() else {
        log::debug!("当前无可用帧数据");
        return;
    };

    log::debug!(
        "渲染第 {} 帧，包含 {} 个实体",
        frame_manager.current_frame_index(),
        keyframe.entity_count()
    );

    // 获取当前帧中存在的实体ID集合
    let current_entity_ids: HashMap<u64, &Inpto> = keyframe.iter_entities().collect();

    // 1. 清理已销毁的实体（在上一帧存在但当前帧不存在）
    cleanup_removed_entities(&mut commands, &current_entity_ids, &mut entity_map.map);

    // 2. 渲染或更新当前帧的所有实体
    for (&entity_id, inpto) in &current_entity_ids {
        if let Some(&entity) = entity_map.map.get(&entity_id) {
            // 实体已存在，更新变换
            update_entity_transform(&mut commands, entity, &inpto.transform, &hidden_query);
            log::trace!("更新实体 {} 的变换", entity_id);
        } else {
            // 实体不存在，创建新实体
            let new_entity = spawn_entity_from_inpto(
                &mut commands,
                &mut meshes,
                &asset_server,
                &material_manager,
                inpto,
                entity_id,
            );
            entity_map.map.insert(entity_id, new_entity);
            log::info!("创建新实体 {} (名称: {})", entity_id, inpto.name());
        }
    }
    
    // 3. 验证所有具有PickableEntity的实体
    log::debug!("=== PickableEntity组件验证 ===");
    log::debug!("当前可拾取实体数量: {}", pickable_check_query.iter().count());
    for (entity, name, pickable) in pickable_check_query.iter() {
        log::debug!("实体 {:?}: {} (PickableEntity ID: {})", entity, name.as_str(), pickable.entity_id);
    }
}

/// 从 Inpto 数据生成 Bevy 实体
fn spawn_entity_from_inpto(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    asset_server: &AssetServer,
    material_manager: &MaterialManager,
    inpto: &Inpto,
    entity_id: u64,
) -> Entity {
    // 转换 Mesh
    let mesh_handle = conversion::proto_mesh_to_bevy(meshes, &inpto.mesh)
        .unwrap_or_else(|| {
            log::warn!("网格转换失败，使用备用球体 (实体 {})", entity_id);
            Mesh3d(meshes.add(Sphere::new(0.1)))
        });

    // 加载材质
    let material_handle = material_manager.load_generic_material(
        &inpto.material_path(),
        asset_server,
    );

    // 生成实体，添加拾取支持和动态实体标记
    let entity = commands
        .spawn((
            mesh_handle,
            crate::renderer::GenericMaterial3d(material_handle.clone()),
            inpto.transform,
            Name::new(format!("FrameEntity_{}", entity_id)),
            Pickable::default(), // Bevy拾取支持
            PickableEntity { entity_id }, // 业务逻辑ID
            DynamicEntity, // 标记为动态实体
        ))
        .observe(handle_dynamic_entity_pick)
        .id();

    log::info!(
        "实体 {} 生成成功 (Bevy Entity: {:?}, 材质: {})",
        entity_id,
        entity,
        inpto.material_path()
    );

    entity
}

/// 更新实体的变换组件（保留 Hidden 状态）
fn update_entity_transform(
    commands: &mut Commands,
    entity: Entity,
    transform: &Transform,
    hidden_query: &Query<(), With<Hidden>>,
) {
    // 检查实体是否有 Hidden 组件
    let has_hidden = hidden_query.get(entity).is_ok();
    
    // 更新变换
    commands.entity(entity).insert(*transform);
    
    // 如果之前是隐藏的，重新添加 Hidden 组件
    if has_hidden {
        commands.entity(entity).insert(Hidden);
    }
}

/// 清理已移除的实体
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
            log::debug!("销毁实体 {} (Bevy Entity: {:?})", entity_id, entity);
        }
    }

    for entity_id in removed_ids {
        entity_map.remove(&entity_id);
    }
}
