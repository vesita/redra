use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use expto::rdmp::mesh::ex_mesh::UMesh;

use crate::data::frame::{FrameManager, Inpto, InptoTransform};
use crate::data::tag::{TagFilter, TagRegistry, entity_passes_filter};
use crate::assets::materials::MaterialManager;
use crate::render::interaction::picking::PickableEntity;
use crate::render::coord_system::{CoordSystem, apply_coord_system};
use crate::ui::file_manager::FileOpSet;

/// 隐藏标记组件
#[derive(Component, Default)]
pub struct Hidden;

/// 单个点云组的缓存
struct PointGroupCache {
    entity: Entity,
    mesh_handle: Handle<Mesh>,
    cached_positions: Vec<[f32; 3]>,
}

/// 实体映射资源
#[derive(Resource, Default)]
pub struct EntityMap {
    pub map: HashMap<u64, Entity>,
    point_groups: HashMap<String, PointGroupCache>,
}

impl EntityMap {
    pub fn clear(&mut self) {
        self.map.clear();
        self.point_groups.clear();
    }

    /// 取出所有点云组实体（用于外部 despawn）
    pub fn drain_point_group_entities(&mut self) -> Vec<Entity> {
        self.point_groups.drain().map(|(_, cache)| cache.entity).collect()
    }
}

/// 帧渲染器插件
pub struct FrameRendererPlugin;

impl Plugin for FrameRendererPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityMap>()
            .add_systems(Update, (
                ApplyDeferred,
                render_current_frame,
            ).chain().after(FileOpSet));
    }
}

/// 渲染当前帧的所有实体
fn render_current_frame(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    material_manager: Res<MaterialManager>,
    frame_manager: Res<FrameManager>,
    tag_filter: Res<TagFilter>,
    tag_registry: Res<TagRegistry>,
    handedness: Res<CoordSystem>,
    mut entity_map: ResMut<EntityMap>,
    pickable_check_query: Query<(Entity, &Name, &PickableEntity)>,
    hidden_query: Query<(), With<Hidden>>,
) {
    let Some(keyframe) = frame_manager.get_current_keyframe() else {
        log::debug!("当前无可用帧数据");
        return;
    };

    log::debug!("渲染第 {} 帧，包含 {} 个实体", frame_manager.current_frame_index(), keyframe.entity_count());

    let mut point_groups: HashMap<String, Vec<Vec3>> = HashMap::new();
    let mut non_point_ids: HashMap<u64, &Inpto> = HashMap::new();

    for (entity_id, inpto) in keyframe.iter_entities() {
        // Tag 筛选：不通过则跳过（不渲染不创建）
        if !entity_passes_filter(&inpto.tags, &tag_filter, &tag_registry) {
            continue;
        }

        if let Some(UMesh::Point(p)) = &inpto.mesh.u_mesh {
            let material = if inpto.material.is_empty() {
                "materials/mesh_types/point.toml".to_string()
            } else {
                inpto.material_path()
            };
            let pos = apply_coord_system(Transform::from_xyz(p.x, p.y, p.z), *handedness).translation;
            point_groups.entry(material).or_default().push(Vec3::new(pos.x, pos.y, pos.z));
        } else {
            non_point_ids.insert(entity_id, inpto);
        }
    }

    let total_points: usize = point_groups.values().map(|v| v.len()).sum();
    if total_points == 0 && !non_point_ids.is_empty() {
        log::warn!("帧 {} 包含 {} 个非 Point 实体，但无 Point 实体", frame_manager.current_frame_index(), non_point_ids.len());
    } else if total_points > 0 {
        log::debug!("帧 {} 包含 {} 个 Point ({} 组) + {} 个非 Point 实体", frame_manager.current_frame_index(), total_points, point_groups.len(), non_point_ids.len());
    }

    cleanup_removed_entities(&mut commands, &non_point_ids, &mut entity_map.map);

    for (&entity_id, inpto) in &non_point_ids {
        if let Some(&entity) = entity_map.map.get(&entity_id) {
            update_entity_transform(&mut commands, entity, &inpto.transform, *handedness, &hidden_query);
        } else {
            let bevy_transform = bevy::transform::components::Transform::from(inpto.transform);
            let new_entity = spawn_entity_from_inpto(&mut commands, &mut meshes, &asset_server, &material_manager, inpto, bevy_transform, entity_id, *handedness);
            entity_map.map.insert(entity_id, new_entity);
            log::info!("创建新实体 {} (名称: {})", entity_id, inpto.name());
        }
    }

    // 按材质分组聚合 Point 为独立 PointList mesh
    update_point_groups(&mut commands, &mut meshes, &asset_server, &material_manager, &mut entity_map, &point_groups);

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
    render_transform: Transform,
    entity_id: u64,
    _handedness: CoordSystem,
) -> Entity {
    let mesh_handle = crate::render::conversion::proto_mesh_to_bevy(meshes, &inpto.mesh)
        .unwrap_or_else(|| {
            let mesh_type = match &inpto.mesh.u_mesh {
                Some(umesh) => format!("{:?}", umesh),
                None => "None".to_string(),
            };
            log::warn!(
                "网格转换失败，使用备用球体 (实体 {}, 类型: {})。\
                 支持的类型: Point, Sphere, Cylinder, Cone, Line(长度>0.001), Cube(维度>0.001)",
                entity_id, mesh_type
            );
            Mesh3d(meshes.add(Sphere::new(0.1)))
        });

    let material_handle = material_manager.load_generic_material(&inpto.material_path(), asset_server);

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

/// 按材质分组聚合 Point 为独立 PointList mesh，每组 1 次 draw call
fn update_point_groups(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    asset_server: &AssetServer,
    material_manager: &MaterialManager,
    entity_map: &mut EntityMap,
    point_groups: &HashMap<String, Vec<Vec3>>,
) {
    // 收集当前帧存在的组，清理不再存在的组
    let mut removed_keys = Vec::new();
    for (key, _cache) in &entity_map.point_groups {
        if !point_groups.contains_key(key) {
            removed_keys.push(key.clone());
        }
    }
    for key in removed_keys {
        if let Some(cache) = entity_map.point_groups.remove(&key) {
            if let Ok(mut ec) = commands.get_entity(cache.entity) {
                ec.despawn();
            }
            log::debug!("移除点云组: {}", key);
        }
    }

    for (material, positions) in point_groups {
        let n_points = positions.len();
        let coords: Vec<[f32; 3]> = positions.iter().map(|p| [p.x, p.y, p.z]).collect();

        // dirty check：点数据未变时跳过 mesh 重建
        if let Some(cache) = entity_map.point_groups.get(material) {
            if cache.cached_positions == coords {
                continue;
            }
        }

        let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; n_points];
        let mut mesh = Mesh::new(PrimitiveTopology::PointList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, coords.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

        // 复用已有 mesh handle
        if let Some(cache) = entity_map.point_groups.get_mut(material) {
            if let Some(mesh_ref) = meshes.get_mut(&cache.mesh_handle) {
                *mesh_ref = mesh;
                cache.cached_positions = coords;
                log::debug!("更新点云组 {}，{} 个点", material, n_points);
                continue;
            }
        }

        // 新建组实体
        let handle = meshes.add(mesh);
        let mat_handle = material_manager.load_generic_material(material, asset_server);
        log::info!("创建点云组 {}，包含 {} 个点", material, n_points);
        let entity = commands.spawn((
            Mesh3d(handle.clone()),
            crate::render::GenericMaterial3d(mat_handle),
            Transform::default(),
            Name::new(format!("PointGroup_{}", material)),
        )).id();
        entity_map.point_groups.insert(material.clone(), PointGroupCache {
            entity,
            mesh_handle: handle,
            cached_positions: coords,
        });
    }
}

fn update_entity_transform(
    commands: &mut Commands,
    entity: Entity,
    transform: &InptoTransform,
    handedness: CoordSystem,
    hidden_query: &Query<(), With<Hidden>>,
) {
    let Ok(mut ec) = commands.get_entity(entity) else { return };
    let bevy_t: bevy::transform::components::Transform = (*transform).into();
    let render_transform = apply_coord_system(bevy_t, handedness);
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
