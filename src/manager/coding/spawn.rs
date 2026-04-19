use bevy::prelude::*;
use expto::rdmp::{Unit, CommandType, object::object::AObject};

// SpawnedEntity 组件标记由spawn系统创建的实体
#[derive(Component)]
pub struct SpawnedEntity;

/// 从 channel 接收所有可用的数据包并处理
pub fn recv_and_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut channel: ResMut<redra_net::RDChannel>,
) {
    // 先处理所有来自 channel 的数据包，避免借用冲突
    let mut units = Vec::new();
    while let Ok(unit) = channel.redra_recver.try_recv() {
        log::debug!("接收Unit数据包");
        units.push(unit);
    }
    
    // 然后处理每个Unit
    for unit in units {
        handle_unit(&mut commands, &mut meshes, &mut materials, unit);
    }
}

/// 处理单个Unit数据包（内部辅助函数）
fn handle_unit(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    unit: Unit,
) {
    if let Some(command) = &unit.command {
        match command.a_command() {
            CommandType::Spawn => {
                // 解析对象信息并创建实体
                handle_spawn_command(commands, meshes, materials, &unit);
            },
            CommandType::Update => {
                // 更新现有实体
                handle_update_command(commands, &unit);
            },
            CommandType::Destroy => {
                // 销毁实体
                handle_destroy_command(commands, &unit);
            },
            _ => {
                // 其他命令类型，暂时忽略
            }
        }
    }
}

/// 处理Spawn命令
fn handle_spawn_command(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    unit: &Unit,
) {
    // 遍历Unit中的对象，查找ID和变换信息
    let mut entity_id: Option<u64> = None;
    let mut position = [0.0, 0.0, 0.0];
    let mut scale = [1.0, 1.0, 1.0];
    let mut color = [1.0, 1.0, 1.0, 1.0]; // RGBA

    for obj in &unit.objects {
        if let Some(AObject::Id(id)) = obj.a_object {
            entity_id = Some(id);
        } else if let Some(AObject::Transform(transform)) = &obj.a_object {
            position = [transform.x, transform.y, transform.z];
            scale = [transform.sx, transform.sy, transform.sz];
        }
    }

    if let Some(id) = entity_id {
        create_entity(commands, meshes, materials, id, position, scale, color);
    }
}

/// 处理Update命令
fn handle_update_command(
    commands: &mut Commands,
    unit: &Unit,
) {
    // 解析对象信息并更新实体
    let mut entity_id: Option<u64> = None;
    let mut position = None;
    let mut scale = None;

    for obj in &unit.objects {
        if let Some(AObject::Id(id)) = obj.a_object {
            entity_id = Some(id);
        } else if let Some(AObject::Transform(transform)) = &obj.a_object {
            position = Some([transform.x, transform.y, transform.z]);
            scale = Some([transform.sx, transform.sy, transform.sz]);
        }
    }

    if let Some(id) = entity_id {
        update_entity(commands, id, position, scale);
    }
}

/// 处理Destroy命令
fn handle_destroy_command(
    commands: &mut Commands,
    unit: &Unit,
) {
    // 解析对象信息并销毁实体
    for obj in &unit.objects {
        if let Some(AObject::Id(id)) = obj.a_object {
            delete_entity(commands, id);
        }
    }
}

/// 创建实体
fn create_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    entity_id: u64,
    position: [f32; 3],
    scale: [f32; 3],
    color: [f32; 4],
) {
    // 使用默认立方体
    let mesh = meshes.add(Cuboid::new(scale[0] * 2.0, scale[1] * 2.0, scale[2] * 2.0));
    let material = materials.add(StandardMaterial::from(Color::srgba(color[0], color[1], color[2], color[3])));

    let transform = Transform::from_xyz(position[0], position[1], position[2])
        .with_scale(Vec3::new(scale[0], scale[1], scale[2]));

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        transform,
        SpawnedEntity, // 添加标记组件
    ));
}

/// 更新实体
fn update_entity(
    commands: &mut Commands,
    entity_id: u64,
    position: Option<[f32; 3]>,
    scale: Option<[f32; 3]>,
) {
    // 实体更新需要根据ID找到对应的实体，这里简化处理
    // 在实际应用中，需要维护实体ID到Bevy实体的映射
    log::debug!("更新实体 {}: pos={:?}, scale={:?}", entity_id, position, scale);
}

/// 删除实体
fn delete_entity(
    commands: &mut Commands,
    entity_id: u64,
) {
    // 实体删除需要根据ID找到对应的实体，这里简化处理
    log::debug!("删除实体: {}", entity_id);
}