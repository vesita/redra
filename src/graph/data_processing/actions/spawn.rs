use bevy::prelude::*;

use crate::{graph::{MaterialManager, PredefinedMaterial}, module::parser::core::{RDPack, RDShapePack}};

// 为动态生成的实体定义标记组件
#[derive(Component)]
pub struct SpawnedEntity;

/// 从 channel 接收所有可用的数据包并处理
pub fn recv_and_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_manager: ResMut<MaterialManager>,
    mut channel: ResMut<crate::graph::communicate::channels::RDChannel>,
) {
    // 先处理所有来自 channel 的数据包，避免借用冲突
    let mut packs = Vec::new();
    while let Ok(pack) = channel.receiver.try_recv() {
        debug!("接收数据包");
        packs.push(pack);
    }
    
    // 然后处理每个数据包
    for pack in packs {
        handle_rd_pack(&mut commands, &mut meshes, &mut materials, &material_manager, pack);
    }
}

/// 处理单个 RDPack 数据包（内部辅助函数）
fn handle_rd_pack(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    material_manager: &MaterialManager,
    pack: RDPack,
) {
    match pack {
        RDPack::Message(_) => todo!(),
        RDPack::SpawnShape(spw) => {
            spawn_shape(commands, meshes, materials, material_manager, *spw);
        },
        RDPack::SpawnFormat(_spw) => {
            // TODO: 处理 SpawnFormat 数据包
        }
    }
}

/// 生成单个形状实体
/// 这个函数可以直接在系统内调用，接受普通引用参数
pub fn spawn_shape(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    material_manager: &MaterialManager,
    spw: RDShapePack,
) {
    debug!("处理 SpawnShape 数据包");
    debug!("{:?}", spw.transform);
    // 通过字符串标识符查找材质
    let material = if let Some(predefined) = material_manager.get_material_desc(&spw.material) {
        match predefined {
            PredefinedMaterial::Color(color) => {
                materials.add(StandardMaterial::from(*color))
            },
            PredefinedMaterial::Standard(mat) => {
                materials.add(mat.clone())
            }
        }
    } else {
        // 如果没有找到预定义材质，则使用默认材质
        materials.add(StandardMaterial::from(Color::srgb(0.8, 0.7, 0.6)))
    };
    
    commands.spawn((
        Mesh3d(meshes.add(spw.mesh.as_ref().clone())),
        MeshMaterial3d(material),
        spw.transform,
        SpawnedEntity,  // 添加标记组件
    ));
}