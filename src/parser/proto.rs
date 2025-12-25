use prost::Message;
use tokio::sync::mpsc;
use std::sync::Arc;
use log::{error, info, debug};

use crate::{parser::{core::{RDPack, SpawnPack}, interface::{cube_rdr, point_rdr, position_rdr, sphere_rdr}}, proto::{rdr, shape}};

/// 发送SpawnPack到Bevy
async fn send_spawn_pack(spawn_pack: SpawnPack, sender: mpsc::Sender<RDPack>) {
    let rd_pack = RDPack::Spawn(Box::new(spawn_pack));
    if let Err(e) = sender.send(rd_pack).await {
        error!("发送RDPack到Bevy失败: {}", e);
    }
}

/// 处理Pack消息
pub fn process_pack(pack: rdr::Pack, sender: mpsc::Sender<RDPack>) {
    match pack.data_type.as_str() {
        "position" => {
            // 将pack.data解析为Position消息
            match rdr::Position::decode(&pack.data[..]) {
                Ok(position) => {
                    let _pos = position_rdr(&position);
                }
                Err(e) => error!("解析 Position 数据失败: {}", e),
            }
        }
        "rotation" => {
            // 将pack.data解析为Rotation消息
            match rdr::Rotation::decode(&pack.data[..]) {
                Ok(rotation) => {
                    // 处理Rotation数据
                    let _rot = rotation;
                }
                Err(e) => error!("解析 Rotation 数据失败: {}", e),
            }
        }
        "scale" => {
            // 将pack.data解析为Scale消息
            match rdr::Scale::decode(&pack.data[..]) {
                Ok(scale) => {
                    // 处理Scale数据
                    let _scale = scale;
                }
                Err(e) => error!("解析 Scale 数据失败: {}", e),
            }
        }
        "cube" => {
            // 将pack.data解析为Cube消息
            match shape::Cube::decode(&pack.data[..]) {
                Ok(cube) => {
                    info!("创建SpawnPack");
                    debug!("Cube: {:?}", cube);
                    let rd_cube = cube_rdr(&cube);
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
                }
                Err(e) => error!("解析 Cube 数据失败: {}", e),
            }
        },
        "sphere" => {
            // 将pack.data解析为Sphere消息
            match shape::Sphere::decode(&pack.data[..]) {
                Ok(sphere) => {
                    let rd_sphere = sphere_rdr(&sphere);
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
                Err(e) => {
                    error!("解析 Sphere 数据失败: {}", e);
                }
            }
        },
        "point" => {
            // 将pack.data解析为Point消息
            match shape::Point::decode(&pack.data[..]) {
                Ok(point) => {
                    // 处理Point数据
                    let point = point_rdr(&point);

                    let mesh = Arc::new(point.to_mesh());
                    let transform = point.pose();

                    let spawn_pack = SpawnPack {
                        mesh,
                        transform,
                        material: "default".to_string(),
                    };
                    tokio::spawn(send_spawn_pack(spawn_pack, sender));
                }
                Err(e) => error!("解析 Point 数据失败: {}", e),
            }
        },
        _ => {
            error!("未知的数据类型: {}", pack.data_type);
        }
    }
}
