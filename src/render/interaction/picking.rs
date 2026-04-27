use bevy::prelude::*;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::picking::pointer::PointerInteraction;
use bevy::mesh::VertexAttributeValues;

use crate::render::interaction::InteractionMessage;

/// 标记组件：可拾取的实体
#[derive(Component, Debug)]
pub struct PickableEntity {
    pub entity_id: u64,
}

/// 动态实体标记
#[derive(Component, Debug)]
pub struct DynamicEntity;

/// 静态实体标记
#[derive(Component, Debug)]
pub struct StaticEntity;

/// 选中标记
#[derive(Component, Default)]
pub struct Selected;

/// 高亮框标记
#[derive(Component)]
pub struct SelectionBox;

/// 处理动态实体拾取事件
pub fn handle_dynamic_entity_pick(
    trigger: On<Pointer<Press>>,
    mut commands: Commands,
    mut im: ResMut<InteractionMessage>,
    previously_selected: Query<Entity, With<Selected>>,
    selection_boxes: Query<Entity, With<SelectionBox>>,
    pickable_query: Query<&PickableEntity>,
    transform_query: Query<&Transform>,
    mesh3d_query: Query<&Mesh3d>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let picked_entity = trigger.entity;

    let Ok(picked_transform) = transform_query.get(picked_entity) else {
        warn!("无法获取被点击实体的 Transform: {:?}", picked_entity);
        return;
    };
    let Ok(pickable) = pickable_query.get(picked_entity) else {
        warn!("被点击实体缺少 PickableEntity 组件: {:?}", picked_entity);
        return;
    };
    let entity_id = pickable.entity_id;
    im.selected = Some(entity_id);

    let is_multi_select = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight)
        || keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);

    if !is_multi_select {
        clear_previous_selection(&mut commands, previously_selected, selection_boxes);
    }

    commands.entity(picked_entity).insert(Selected);
    create_highlight_box(&mut commands, &mut meshes, &mut materials, picked_transform, picked_entity, &mesh3d_query);
    info!("实体 {} (ID: {}) 已被选中", picked_entity, entity_id);
}

fn clear_previous_selection(
    commands: &mut Commands,
    previously_selected: Query<Entity, With<Selected>>,
    selection_boxes: Query<Entity, With<SelectionBox>>,
) {
    for entity in previously_selected.iter() {
        commands.entity(entity).remove::<Selected>();
    }
    for box_entity in selection_boxes.iter() {
        commands.entity(box_entity).despawn();
    }
}

/// 从 Mesh 顶点数据计算 AABB
fn compute_mesh_aabb(mesh: &Mesh) -> Option<(Vec3, Vec3)> {
    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?;
    match positions {
        VertexAttributeValues::Float32x3(verts) => {
            let mut min = Vec3::splat(f32::MAX);
            let mut max = Vec3::splat(f32::MIN);
            for &[x, y, z] in verts {
                min = min.min(Vec3::new(x, y, z));
                max = max.max(Vec3::new(x, y, z));
            }
            Some((min, max))
        }
        _ => None,
    }
}

/// 基于网格实际 AABB 创建 12 条边框高亮
fn create_highlight_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    entity_transform: &Transform,
    picked_entity: Entity,
    mesh3d_query: &Query<&Mesh3d>,
) {
    // 从实体实际网格计算 AABB
    let aabb = mesh3d_query
        .get(picked_entity)
        .ok()
        .and_then(|handle| meshes.get(handle))
        .and_then(|mesh| compute_mesh_aabb(mesh));

    let (local_min, local_max) = if let Some((min, max)) = aabb {
        (min, max)
    } else {
        // 无法计算 AABB 时回退为 1 单位立方体
        (Vec3::splat(-0.5), Vec3::splat(0.5))
    };

    // 8 个角点（局部空间 → 世界空间）
    let corners_local = [
        Vec3::new(local_min.x, local_min.y, local_min.z),
        Vec3::new(local_max.x, local_min.y, local_min.z),
        Vec3::new(local_max.x, local_min.y, local_max.z),
        Vec3::new(local_min.x, local_min.y, local_max.z),
        Vec3::new(local_min.x, local_max.y, local_min.z),
        Vec3::new(local_max.x, local_max.y, local_min.z),
        Vec3::new(local_max.x, local_max.y, local_max.z),
        Vec3::new(local_min.x, local_max.y, local_max.z),
    ];
    let corners: [Vec3; 8] = corners_local.map(|c| entity_transform.transform_point(c));

    // 12 条棱（角点索引对）
    const EDGES: [(usize, usize); 12] = [
        (0, 1), (1, 2), (2, 3), (3, 0), // 底面
        (4, 5), (5, 6), (6, 7), (7, 4), // 顶面
        (0, 4), (1, 5), (2, 6), (3, 7), // 竖边
    ];

    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.7, 1.0, 0.9),
        emissive: LinearRgba::rgb(0.0, 0.5, 1.0) * 40.0,
        unlit: true,
        ..default()
    });

    const EDGE_RADIUS: f32 = 0.025;

    for &(i, j) in &EDGES {
        let start = corners[i];
        let end = corners[j];
        let direction = end - start;
        let length = direction.length();
        if length < 0.001 {
            continue;
        }
        let mid = (start + end) / 2.0;
        let dir_norm = direction / length;

        let rotation = if dir_norm.normalize_or_zero().length_squared() < 0.5 {
            Quat::IDENTITY
        } else {
            Quat::from_rotation_arc(Vec3::Y, dir_norm)
        };

        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(EDGE_RADIUS, length))),
            MeshMaterial3d(edge_material.clone()),
            Transform::from_translation(mid).with_rotation(rotation),
            SelectionBox,
        ));
    }
}

/// 检测点击空白区域或静态实体，清空选中状态并清理高亮框
pub fn detect_empty_click(
    mouse: Res<ButtonInput<MouseButton>>,
    pointer_query: Query<&PointerInteraction>,
    dynamic_query: Query<&DynamicEntity>,
    mut commands: Commands,
    selection_boxes: Query<Entity, With<SelectionBox>>,
    previously_selected: Query<Entity, With<Selected>>,
    mut im: ResMut<InteractionMessage>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(pointer) = pointer_query.single() else {
        return;
    };
    let hit_dynamic = pointer
        .get_nearest_hit()
        .map(|(entity, _)| dynamic_query.get(*entity).is_ok())
        .unwrap_or(false);
    if !hit_dynamic {
        im.selected = None;
        for entity in previously_selected.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        for box_entity in selection_boxes.iter() {
            commands.entity(box_entity).despawn();
        }
    }
}
