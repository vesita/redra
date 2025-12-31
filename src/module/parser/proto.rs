use bevy::transform::components::Transform;
use log::{debug, error, info};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::{
    module::parser::{
        core::{RDPack, RDShapePack},
        interface::*,
    },
    proto::{
        command,
        designation::{self, DesignCmd},
        formats::{self, FormatPack},
        shape::{self, ShapePack},
    },
};

/// 发送SpawnPack到Bevy
/// 
/// 将创建的形状数据包通过异步通道发送到Bevy引擎进行渲染
/// 
/// # 参数
/// * `spawn_pack` - 包含形状数据的RDShapePack对象
/// * `sender` - 用于发送数据到Bevy的异步发送器
async fn send_spawn_pack(spawn_pack: RDShapePack, sender: mpsc::Sender<RDPack>) {
    let rd_pack = RDPack::SpawnShape(Box::new(spawn_pack));
    if let Err(e) = sender.send(rd_pack).await {
        error!("发送RDPack到Bevy失败: {}", e);
    }
}

/// 为指定的几何体创建SpawnPack并发送到Bevy
/// 
/// 泛型函数，将几何体转换为Mesh和Transform，并通过异步任务发送到Bevy
/// 
/// # 参数
/// * `geometry` - 输入的几何体对象
/// * `to_mesh_fn` - 将几何体转换为Mesh和Transform的函数
/// * `material` - 形状的材质名称
/// * `sender` - 用于发送数据到Bevy的异步发送器
/// 
/// # 泛型参数
/// * `T` - 几何体类型，需要实现Send和静态生命周期
/// * `F` - 转换函数类型，将几何体转换为(Mesh, Transform)
fn create_and_send_spawn_pack<T, F>(
    geometry: T,
    to_mesh_fn: F,
    material: String,
    sender: mpsc::Sender<RDPack>,
) where
    F: FnOnce(&T) -> (Arc<bevy::mesh::Mesh>, Transform) + Send + 'static,
    T: Send + 'static,
{
    let (mesh, transform) = to_mesh_fn(&geometry);
    let spawn_pack = RDShapePack {
        mesh,
        transform,
        material,
    };
    tokio::spawn(send_spawn_pack(spawn_pack, sender));
}

/// 处理Point形状
/// 
/// 将protobuf中的Point转换为内部表示，并发送到Bevy进行渲染
/// 
/// # 参数
/// * `point` - protobuf定义的Point对象
/// * `sender` - 用于发送数据到Bevy的异步发送器
fn handle_point_shape(point: &shape::Point, sender: mpsc::Sender<RDPack>) {
    let point = point_rd(point);
    create_and_send_spawn_pack(
        point,
        |p| (Arc::new(p.to_mesh()), p.pose()),
        "blue".to_string(),
        sender,
    );
}

/// 处理Segment形状
/// 
/// 将protobuf中的Segment转换为内部表示，并发送到Bevy进行渲染
/// 
/// # 参数
/// * `segment` - protobuf定义的Segment对象
/// * `sender` - 用于发送数据到Bevy的异步发送器
fn handle_segment_shape(segment: &shape::Segment, sender: mpsc::Sender<RDPack>) {
    let segment = segment_rd(segment);
    create_and_send_spawn_pack(
        segment,
        |s| (Arc::new(s.to_mesh()), Transform::default()),
        "white".to_string(),
        sender,
    );
}

/// 处理Sphere形状
/// 
/// 将protobuf中的Sphere转换为内部表示，并发送到Bevy进行渲染
/// 
/// # 参数
/// * `sphere` - protobuf定义的Sphere对象
/// * `sender` - 用于发送数据到Bevy的异步发送器
fn handle_sphere_shape(sphere: &shape::Sphere, sender: mpsc::Sender<RDPack>) {
    let rd_sphere = sphere_rd(sphere);
    create_and_send_spawn_pack(
        rd_sphere,
        |s| (Arc::new(s.to_mesh()), s.pose()),
        "default".to_string(),
        sender,
    );
}

/// 处理Cube形状
/// 
/// 将protobuf中的Cube转换为内部表示，并发送到Bevy进行渲染
/// 
/// # 参数
/// * `cube` - protobuf定义的Cube对象
/// * `sender` - 用于发送数据到Bevy的异步发送器
fn handle_cube_shape(cube: &shape::Cube, sender: mpsc::Sender<RDPack>) {
    debug!("创建SpawnPack");
    debug!("Cube: {:?}", cube);
    let rd_cube = cube_rd(cube);
    create_and_send_spawn_pack(
        rd_cube,
        |c| (Arc::new(c.to_mesh()), c.pose()),
        "default".to_string(),
        sender,
    );
}

/// 处理图像格式
/// 
/// 处理protobuf中的图像数据（待实现）
/// 
/// # 参数
/// * `image` - protobuf定义的Image对象
/// * `sender` - 用于发送数据到Bevy的异步发送器
fn handle_image_fmt(image: &formats::Image, sender: mpsc::Sender<RDPack>) {
    // todo
    debug!("处理图像数据: {:?}", image);
}

/// 处理形状数据包
/// 
/// 根据形状数据包中的类型匹配并处理相应的形状
/// 
/// # 参数
/// * `shape_pack` - protobuf定义的ShapePack对象
/// * `sender` - 用于发送数据到Bevy的异步发送器
fn match_shape_data(shape_pack: &ShapePack, sender: mpsc::Sender<RDPack>) {
    if let Some(ref data) = shape_pack.data {
        debug!("ShapeData数据包");
        match data {
            shape::shape_pack::Data::Point(point) => {
                debug!("Point数据包");
                handle_point_shape(point, sender);
            }
            shape::shape_pack::Data::Segment(segment) => {
                debug!("Segment数据包");
                handle_segment_shape(segment, sender);
            }
            shape::shape_pack::Data::Sphere(sphere) => {
                debug!("Sphere数据包");
                handle_sphere_shape(sphere, sender);
            }
            shape::shape_pack::Data::Cube(cube) => {
                debug!("Cube数据包");
                handle_cube_shape(cube, sender);
            }
        }
    } else {
        debug!("ShapePack消息中没有定义任何形状");
    }
}

/// 处理格式数据包
/// 
/// 根据格式数据包中的类型匹配并处理相应的格式
/// 
/// # 参数
/// * `format_pack` - protobuf定义的FormatPack对象
/// * `sender` - 用于发送数据到Bevy的异步发送器
fn match_format_data(format_pack: &FormatPack, sender: mpsc::Sender<RDPack>) {
    debug!("FormatData数据包");
    // 如果有需要处理的数据，可以在这里添加
    if let Some(ref data) = format_pack.data {
        match data {
            formats::format_pack::Data::Image(image) => {
                debug!("Image数据包");
                handle_image_fmt(image, sender);
            }
        }
    }
}

/// 处理Designation命令
/// 
/// 解析Designation命令并根据其内容处理相应的数据
/// 
/// # 参数
/// * `designation_cmd` - protobuf定义的DesignCmd对象
/// * `sender` - 用于发送数据到Bevy的异步发送器
fn match_designation_cmd(designation_cmd: &DesignCmd, sender: mpsc::Sender<RDPack>) {
    if let Some(designation::design_cmd::Cmd::Spawn(spawn)) = &designation_cmd.cmd {
        if let Some(ref data) = spawn.data {
            debug!("Spawn数据包");
            match data {
                designation::spawn::Data::ShapeData(shape_pack) => {
                    match_shape_data(&shape_pack, sender);
                }
                designation::spawn::Data::FormatData(format_pack) => {
                    match_format_data(format_pack, sender);
                }
            }
        } else {
            info!("Designation: 无数据包");
        }
    }
}

/// 处理Pack消息
/// 
/// 解析Command消息并根据其类型处理相应的命令
/// 
/// # 参数
/// * `pack` - protobuf定义的Command对象
/// * `sender` - 用于发送数据到Bevy的异步发送器
pub fn process_pack(pack: command::Command, sender: mpsc::Sender<RDPack>) {
    match pack.cmd_pack {
        Some(command::command::CmdPack::Conception(ref conception_cmd)) => {
            // 处理Conception命令
            debug!("处理Conception命令: {:?}", conception_cmd);
        }
        Some(command::command::CmdPack::Designation(ref designation_cmd)) => {
            match_designation_cmd(&designation_cmd, sender);
        }
        Some(command::command::CmdPack::Transform(ref translation)) => {
            // 处理Transform命令
        }
        None => {
            error!("Command消息中没有定义任何命令");
        }
    }
}