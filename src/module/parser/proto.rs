use bevy::transform::components::Transform;
use tokio::sync::mpsc;
use std::sync::Arc;
use log::{error, info, debug};

use crate::{module::parser::{core::{RDPack, ShapePack}, interface::*}, proto::{command, designation, formats, shape}};

/// 发送SpawnPack到Bevy
async fn send_spawn_pack(spawn_pack: ShapePack, sender: mpsc::Sender<RDPack>) {
    let rd_pack = RDPack::SpawnShape(Box::new(spawn_pack));
    if let Err(e) = sender.send(rd_pack).await {
        error!("发送RDPack到Bevy失败: {}", e);
    }
}

/// 为指定的几何体创建SpawnPack并发送到Bevy
fn create_and_send_spawn_pack<T, F>(
    geometry: T,
    to_mesh_fn: F,
    sender: mpsc::Sender<RDPack>,
) where
    F: FnOnce(&T) -> (Arc<bevy::mesh::Mesh>, Transform) + Send + 'static,
    T: Send + 'static,
{
    let (mesh, transform) = to_mesh_fn(&geometry);
    let spawn_pack = ShapePack {
        mesh,
        transform,
        material: "default".to_string(),
    };
    tokio::spawn(send_spawn_pack(spawn_pack, sender));
}

/// 处理Point形状
fn handle_point_shape(point: &shape::Point, sender: mpsc::Sender<RDPack>) {
    let point = point_rd(point);
    create_and_send_spawn_pack(point, |p| (Arc::new(p.to_mesh()), p.pose()), sender);
}

/// 处理Segment形状
fn handle_segment_shape(segment: &shape::Segment, sender: mpsc::Sender<RDPack>) {
    let segment = segment_rd(segment);
    create_and_send_spawn_pack(segment, |s| (Arc::new(s.to_mesh()), Transform::default()), sender);
}

/// 处理Sphere形状
fn handle_sphere_shape(sphere: &shape::Sphere, sender: mpsc::Sender<RDPack>) {
    let rd_sphere = sphere_rd(sphere);
    create_and_send_spawn_pack(rd_sphere, |s| (Arc::new(s.to_mesh()), s.pose()), sender);
}

fn handle_image_fmt(image: &formats::Image, sender: mpsc::Sender<RDPack>) {
    
}

/// 处理Cube形状
fn handle_cube_shape(cube: &shape::Cube, sender: mpsc::Sender<RDPack>) {
    info!("创建SpawnPack");
    debug!("Cube: {:?}", cube);
    let rd_cube = cube_rd(cube);
    create_and_send_spawn_pack(rd_cube, |c| (Arc::new(c.to_mesh()), c.pose()), sender);
}

/// 处理Pack消息
pub fn process_pack(pack: command::Command, sender: mpsc::Sender<RDPack>) {
    match pack.cmd_pack {
        Some(command::command::CmdPack::Conception(ref conception_cmd)) => {
            // 处理Conception命令
            debug!("处理Conception命令: {:?}", conception_cmd);
        },
        Some(command::command::CmdPack::Designation(ref designation_cmd)) => {
            match &designation_cmd.cmd {
                Some(cmd) => {
                    match cmd {
                        designation::design_cmd::Cmd::Spawn(spawn) => {
                            match &spawn.data {
                                Some(data) => {
                                    match data {
                                        designation::spawn::Data::FormatData(format_pack) => {
                                            // todo
                                            info!("FormatData数据包");
                                        },
                                        designation::spawn::Data::ShapeData(shape_pack) => {
                                            match shape_pack.data {
                                                Some(pack) => {
                                                    info!("ShapeData数据包");
                                                    match pack {
                                                        shape::shape_pack::Data::Point(point) =>{
                                                            handle_point_shape(&point, sender);
                                                        },
                                                        shape::shape_pack::Data::Segment(segment) => {
                                                            handle_segment_shape(&segment, sender);
                                                        },
                                                        shape::shape_pack::Data::Sphere(sphere) => {
                                                            handle_sphere_shape(&sphere, sender);
                                                        },
                                                        shape::shape_pack::Data::Cube(cube) => {
                                                            handle_cube_shape(&cube, sender);
                                                        },
                                                    }
                                                },
                                                None => {
                                                    info!("Designation: 无数据包");
                                                },
                                            }
                                        },
                                    }
                                },
                                None => {
                                    info!("Designation: 无数据包");
                                },
                            }
                        },
                    }
                },
                None => todo!(),
            }
            // 处理Designation命令
            // if let Some(designation::design_cmd::Cmd::Spawn(spawn)) = &designation_cmd.cmd {
            //     // 如果是Spawn命令，处理其中的shape_data
            //     if let Some(ref data) = spawn.data {
            //         debug!("Spawn数据包");
            //         match data {
            //             designation::spawn::Data::ShapeData(shape_pack) => {
            //                 if let Some(ref shape_oneof) = shape_pack.data {
            //                     match shape_oneof {
            //                         shape::shape_pack::Data::Point(point) => {
            //                             info!("Point数据包");
            //                             // 处理Point数据
            //                             handle_point_shape(&point, sender);
            //                         }
            //                         shape::shape_pack::Data::Segment(segment) => {
            //                             // 处理Segment数据
            //                             handle_segment_shape(&segment, sender);
            //                         }
            //                         shape::shape_pack::Data::Sphere(sphere) => {
            //                             handle_sphere_shape(&sphere, sender);
            //                         },
            //                         shape::shape_pack::Data::Cube(cube) => {
            //                             handle_cube_shape(&cube, sender);
            //                         },
            //                     }
            //                 } else {
            //                     error!("ShapePack消息中没有定义任何形状");
            //                 }
            //             },
            //             designation::spawn::Data::FormatData(fmt) => {
            //                 if let Some(ref data) = fmt.data {
            //                     match data {
            //                         formats::format_pack::Data::Image(image) => {

            //                         },
            //                     }
            //                 }
            //             }
            //         }
            //     }
            // }
        },
        Some(command::command::CmdPack::Transform(ref translation)) => {
        },
        None => {
            error!("Command消息中没有定义任何命令");
        }
    }
}