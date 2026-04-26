use bevy::prelude::*;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::picking::prelude::*;

// 如果将这里注册成插件，则会是所有实体能够被拾取，不符合要求

/// 标记组件：可拾取的实体
#[derive(Component, Debug)]
pub struct PickableEntity {
    /// 业务逻辑中的实体 ID
    pub entity_id: u64,
}

/// 选中标记组件（用于标识用户选中的实体）
#[derive(Component, Default)]
pub struct Selected;

/// 选择框组件 - 标记这是一个选择框的子实体
#[derive(Component)]
pub struct SelectionBox;

/// 处理实体拾取事件 - 观察者系统(响应点击事件)
pub fn handle_entity_pick(
    trigger: On<Pointer<Press>>,
    mut commands: Commands,
    previously_selected: Query<Entity, With<Selected>>,
    selection_boxes: Query<Entity, With<SelectionBox>>,
    pickable_query: Query<&PickableEntity>,
    transform_query: Query<&Transform>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 通过触发器直接获取被点击的实体
    let picked_entity = trigger.entity;
    
    debug!("实体被点击: {:?}", picked_entity);

    // 获取被点击实体的 Transform
    let Ok(picked_transform) = transform_query.get(picked_entity) else {
        warn!("无法获取被点击实体的 Transform: {:?}", picked_entity);
        return;
    };

    // 获取被点击实体的业务 ID
    let Ok(pickable) = pickable_query.get(picked_entity) else {
        warn!("被点击实体缺少 PickableEntity 组件: {:?}", picked_entity);
        return;
    };
    let entity_id = pickable.entity_id;

    // 检查是否按下 Shift 或 Ctrl 键（支持多选）
    let is_multi_select = keyboard.pressed(KeyCode::ShiftLeft) 
        || keyboard.pressed(KeyCode::ShiftRight)
        || keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);

    if !is_multi_select {
        // 单选模式：清除之前的选中状态
        clear_previous_selection(&mut commands, previously_selected, selection_boxes);
    }

    // 标记新选中的实体
    commands.entity(picked_entity).insert(Selected);

    // 为新选中的实体创建选择框
    create_selection_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        picked_transform,
    );

    info!("实体 {} (ID: {}) 已被选中", picked_entity, entity_id);
}

/// 清除之前的选中状态和选择框
fn clear_previous_selection(
    commands: &mut Commands,
    previously_selected: Query<Entity, With<Selected>>,
    selection_boxes: Query<Entity, With<SelectionBox>>,
) {
    // 移除之前实体的 Selected 标记
    for entity in previously_selected.iter() {
        commands.entity(entity).remove::<Selected>();
        debug!("清除实体 {:?} 的选中状态", entity);
    }

    // 销毁所有旧的选择框
    for box_entity in selection_boxes.iter() {
        commands.entity(box_entity).despawn();
        debug!("销毁选择框实体: {:?}", box_entity);
    }
}

/// 为选中实体创建选择框（8个角点标记）
fn create_selection_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    entity_transform: &Transform,
) {
    // TODO: 根据实际网格AABB计算尺寸，这里使用默认值
    let size = Vec3::new(1.0, 1.0, 1.0);
    let half_size = size / 2.0;

    // 定义立方体的8个顶点位置（相对于实体中心）
    let corners = [
        // 前面四个角
        Vec3::new(-half_size.x, -half_size.y, half_size.z),   // 左下前
        Vec3::new(half_size.x, -half_size.y, half_size.z),    // 右下前
        Vec3::new(half_size.x, half_size.y, half_size.z),     // 右上前
        Vec3::new(-half_size.x, half_size.y, half_size.z),    // 左上前
        // 后面四个角
        Vec3::new(-half_size.x, -half_size.y, -half_size.z),  // 左下后
        Vec3::new(half_size.x, -half_size.y, -half_size.z),   // 右下后
        Vec3::new(half_size.x, half_size.y, -half_size.z),    // 右上后
        Vec3::new(-half_size.x, half_size.y, -half_size.z),   // 左上后
    ];

    // 创建角点标记的网格和材质（复用同一个资源）
    let corner_mesh = meshes.add(Cuboid::from_size(Vec3::new(0.08, 0.08, 0.08)));
    let corner_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 0.0, 1.0), // 黄色
        emissive: LinearRgba::rgb(1.0, 1.0, 0.0) * 50.0, // 发光效果
        ..default()
    });

    // 在8个角点位置创建标记
    for corner in &corners {
        let world_pos = entity_transform.translation + *corner;
        
        commands.spawn((
            Mesh3d(corner_mesh.clone()),
            MeshMaterial3d(corner_material.clone()),
            Transform::from_translation(world_pos),
            SelectionBox,
        ));
    }

    debug!("已创建 8 个选择框角点标记");
}
