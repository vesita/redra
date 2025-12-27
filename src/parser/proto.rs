use bevy::{ecs::spawn, transform::components::Transform};
use prost::Message;
use tokio::sync::mpsc;
use std::sync::Arc;
use log::{error, info, debug};

use crate::{parser::{core::{RDPack, SpawnPack}, interface::*}, proto::{command, designation, shape}};

/// 发送SpawnPack到Bevy
async fn send_spawn_pack(spawn_pack: SpawnPack, sender: mpsc::Sender<RDPack>) {
    let rd_pack = RDPack::Spawn(Box::new(spawn_pack));
    if let Err(e) = sender.send(rd_pack).await {
        error!("发送RDPack到Bevy失败: {}", e);
    }
}

/// 处理Pack消息
pub fn process_pack(pack: command::Command, sender: mpsc::Sender<RDPack>) {
    match pack.cmd_pack {
        Some(command::command::CmdPack::Conception(ref conception_cmd)) => {
            // 处理Conception命令
            info!("处理Conception命令: {:?}", conception_cmd);
        }
        Some(command::command::CmdPack::Designation(ref designation)) => {
            // 处理Designation命令
            if let Some(designation::design_cmd::Cmd::Spawn(spawn)) = &designation.cmd {
                // 如果是Spawn命令，处理其中的shape
                if let Some(ref shape_oneof) = spawn.shape {
                    match &shape_oneof.data {
                        Some(shape::shape_pack::Data::Point(point)) => {
                            // 处理Point数据
                            let point = point_rd(point);

                            let mesh = Arc::new(point.to_mesh());
                            let transform = point.pose();

                            let spawn_pack = SpawnPack {
                                mesh,
                                transform,
                                material: "default".to_string(),
                            };
                            tokio::spawn(send_spawn_pack(spawn_pack, sender));
                        }
                        Some(shape::shape_pack::Data::Segment(segment)) => {
                            // 处理Segment数据
                            let segment = segment_rd(segment);

                            let mesh = Arc::new(segment.to_mesh());

                            let spawn_pack = SpawnPack {
                                mesh,
                                transform: Transform::default(),
                                material: "default".to_string(),
                            };
                            tokio::spawn(send_spawn_pack(spawn_pack, sender));
                        }
                        Some(shape::shape_pack::Data::Sphere(sphere)) => {
                            let rd_sphere = sphere_rd(sphere);
                            // 创建Bevy网格和变换
                            let mesh = Arc::new(rd_sphere.to_mesh());
                            let transform = rd_sphere.pose();
                            // 创建SpawnPack并发送到Bevy
                            let spawn_pack = SpawnPack {
                                mesh,
                                transform,
                                material: "default".to_string(),
                            };
                            tokio::spawn(send_spawn_pack(spawn_pack, sender));
                        },
                        Some(shape::shape_pack::Data::Cube(cube)) => {
                            info!("创建SpawnPack");
                            debug!("Cube: {:?}", cube);
                            let rd_cube = cube_rd(cube);
                            // 创建Bevy网格和变换
                            let mesh = Arc::new(rd_cube.to_mesh());
                            let transform = rd_cube.pose();
                            
                            // 创建SpawnPack并发送到Bevy
                            let spawn_pack = SpawnPack {
                                mesh,
                                transform,
                                material: "default".to_string(),
                            };
                            
                            tokio::spawn(send_spawn_pack(spawn_pack, sender));
                        },
                        None => {
                            error!("Shape消息中没有定义任何形状");
                        }
                    }
                }
            }
        }
        Some(command::command::CmdPack::Transform(ref translation)) => {
        }
        None => {
            error!("Command消息中没有定义任何命令");
        }
    }
}