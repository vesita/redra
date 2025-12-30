use bevy::prelude::*;

use crate::{module::parser::core::RDPack, module::resource::RDResource};

pub fn general_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut resources: ResMut<RDResource>
) {
    // 先处理所有来自channel的数据包，避免借用冲突
    let mut packs = Vec::new();
    while let Ok(pack) = resources.channel.receiver.try_recv() {
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
                let material = if let Some(handle) = resources.materials.get(&spw.material) {
                    let handle_clone = handle.clone();
                    handle_clone
                } else {
                    // 如果没有找到指定材质，则使用默认材质
                    materials.add(StandardMaterial::from(Color::srgb(0.8, 0.7, 0.6)))
                };
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