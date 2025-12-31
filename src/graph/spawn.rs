use bevy::prelude::*;

use crate::{graph::{MaterialManager, communicate::channels}, module::parser::core::RDPack};

pub fn general_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material_manager: ResMut<MaterialManager>,
    mut channel: ResMut<channels::RDChannel>,
) {
    // 先处理所有来自channel的数据包，避免借用冲突
    let mut packs = Vec::new();
    while let Ok(pack) = channel.receiver.try_recv() {
        debug!("接收数据包");
        packs.push(pack);
    }
    
    // 然后处理每个数据包，此时已经不再借用resources.channel
    for pack in packs {
        match pack {
            RDPack::Message(_) => todo!(),
            RDPack::SpawnShape(spw) => {
                debug!("处理SpawnShape数据包");
                debug!("{:?}", spw.transform);
                // 通过字符串标识符查找材质
                let material = material_manager.get_or_insert_material(&spw.material, &mut materials);
                commands.spawn((
                    Mesh3d(meshes.add(spw.mesh.as_ref().clone())),
                    MeshMaterial3d(material),
                    spw.transform,
                ));
            },
            RDPack::SpawnFormat(spw) => {
                
            }
        }
    }
}