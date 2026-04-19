use bevy::prelude::*;
use bevy_materialize::prelude::*;
use parser::{ProtocolParser, ParsedCommand, FrameAssembler, DefaultMaterialConfig};
use expto::rdmp::Unit;
use redra_net::RDChannel;
use std::collections::HashMap;

/// 实体ID组件
#[derive(Component, Debug, Clone)]
pub struct EntityId(pub u64);

/// Parser 管理器资源
#[derive(Resource)]
pub struct ParserManager {
    assembler: FrameAssembler,
    entities: HashMap<u64, Entity>,
    config: DefaultMaterialConfig,
}

impl ParserManager {
    pub fn new(config: DefaultMaterialConfig) -> Self {
        Self {
            assembler: FrameAssembler::new(),
            entities: HashMap::new(),
            config,
        }
    }

    /// 获取最新的帧数据
    pub fn get_latest_frame_data(&self) -> Option<&inpto::FrameData> {
        // 这里可以缓存最近的 frame_data
        None
    }
}

impl Default for ParserManager {
    fn default() -> Self {
        Self::new(DefaultMaterialConfig::default())
    }
}

/// Parser 插件
pub struct ParserManagerPlugin;

impl Plugin for ParserManagerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ParserManager>()
            .add_systems(Update, process_network_units);
    }
}

/// 处理网络 Units 的系统
fn process_network_units(
    mut rd_channel: ResMut<RDChannel>,
    mut parser_manager: ResMut<ParserManager>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut units_buffer = Vec::new();
    
    // 收集所有可用的 Units
    loop {
        match rd_channel.redra_recver.try_recv() {
            Ok(unit) => {
                units_buffer.push(unit);
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
        }
    }
    
    if units_buffer.is_empty() {
        return;
    }
    
    // 加载默认材质（首次）
    let material_handle = asset_server.load(&parser_manager.config.material_path);
    
    // 解析并处理每个 Unit
    for unit in &units_buffer {
        let parsed_cmd = ProtocolParser::parse_unit(unit);
        
        match parsed_cmd {
            ParsedCommand::Spawn { id, transform } => {
                let transform_component = if let Some(t) = transform {
                    Transform::from_translation(Vec3::from_array(t.position))
                        .with_rotation(Quat::from_euler(EulerRot::XYZ, t.rotation[0], t.rotation[1], t.rotation[2]))
                        .with_scale(Vec3::from_array(t.scale))
                } else {
                    Transform::IDENTITY
                };
                
                let entity = commands.spawn((
                    EntityId(id),
                    Mesh3d(asset_server.add(Mesh::from(Capsule3d::new(0.5, 2.0)))),
                    GenericMaterial3d(material_handle.clone()),
                    transform_component,
                )).id();
                
                parser_manager.entities.insert(id, entity);
            },
            ParsedCommand::Update { id, transform } => {
                if let Some(&entity) = parser_manager.entities.get(&id) {
                    if let Some(t) = transform {
                        let new_transform = Transform::from_translation(Vec3::from_array(t.position))
                            .with_rotation(Quat::from_euler(EulerRot::XYZ, t.rotation[0], t.rotation[1], t.rotation[2]))
                            .with_scale(Vec3::from_array(t.scale));
                        
                        commands.entity(entity).insert(new_transform);
                    }
                }
            },
            ParsedCommand::Destroy { id } => {
                if let Some(&entity) = parser_manager.entities.get(&id) {
                    commands.entity(entity).despawn();
                    parser_manager.entities.remove(&id);
                }
            },
            ParsedCommand::Unknown => {
                log::warn!("Received unknown command");
            }
        }
    }
    
    // 组装帧数据（可用于后续存储或回放）
    if let Some(stamp) = units_buffer.first().and_then(|u| u.stamp.as_ref()) {
        let _frame_data = parser_manager.assembler.assemble_frame(
            &units_buffer,
            stamp.timestamp,
        );
        // 这里可以将 frame_data 传递给 storage 或用于其他用途
    }
}
