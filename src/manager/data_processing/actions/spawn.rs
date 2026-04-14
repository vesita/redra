use bevy::prelude::*;
use redra_parser::core::{RDPack, InternalShapePack};

use crate::manager::data_processing::entities::SpawnedEntity;

/// 从 channel 接收所有可用的数据包并处理
pub fn recv_and_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut channel: ResMut<redra_net::RDChannel>,
) {
    // 先处理所有来自 channel 的数据包，避免借用冲突
    let mut packs = Vec::new();
    while let Ok(pack) = channel.receiver.try_recv() {
        debug!("接收数据包");
        packs.push(pack);
    }
    
    // 然后处理每个数据包
    for pack in packs {
        handle_rd_pack(&mut commands, &mut meshes, &mut materials, pack);
    }
}

/// 处理单个 RDPack 数据包（内部辅助函数）
fn handle_rd_pack(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pack: RDPack,
) {
    match pack {
        RDPack::Message(_) => {
            // 忽略消息数据包
        },
        RDPack::SpawnShape(spw) => {
            handle_internal_shape_pack(commands, meshes, materials, &spw);
        },
        RDPack::PointCloud(_) => {
            // 点云数据由记录器处理，不在此处生成实体
            debug!("接收到点云数据包，将由记录器处理");
        }
        RDPack::SpawnFormat(_) => {
            // TODO: 处理格式包
        },
    }
}

/// 处理 InternalShapePack 结构体的函数
pub fn handle_internal_shape_pack(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    spw: &InternalShapePack,
) {
    debug!("处理 SpawnShape 数据包");
    debug!("{:?}", spw.transform);

    let material_handle = match spw.material.as_str() {
        "point_material" => {
            materials.add(StandardMaterial::from(Color::srgb(0.0, 0.0, 1.0))) // 蓝色
        }
        "sphere_material" => {
            materials.add(StandardMaterial::from(Color::srgb(1.0, 0.0, 0.0))) // 红色
        }
        "cube_material" => {
            materials.add(StandardMaterial::from(Color::srgb(0.0, 1.0, 0.0))) // 绿色
        }
        _ => {
            materials.add(StandardMaterial::from(Color::srgb(1.0, 1.0, 1.0))) // 默认白色
        }
    };

    commands.spawn((
        Mesh3d(meshes.add((*spw.mesh).clone())),
        MeshMaterial3d(material_handle),
        spw.transform,
        SpawnedEntity, // 添加标记组件
    ));
}

/// 根据当前播放帧生成实体
pub fn spawn_from_current_frame(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    playback: Res<crate::manager::data_processing::actions::record::PlaybackManager>,
    _recorder: Res<crate::manager::data_processing::actions::record::DataRecorder>,
) {
    // 检查是否有新的帧需要渲染
    if let Some(ref loaded_frame) = playback.loaded_frame {
        // 清除之前的生成实体
        // 注意：这里我们假设有另一个系统负责清理之前生成的实体
        
        for pack in loaded_frame {
            if let RDPack::SpawnShape(spw) = pack {
                handle_internal_shape_pack(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    spw,
                );
            }
        }
    }
}