use tokio::time::{sleep, Duration};
use tokio::net::TcpStream;
use log::{info, error};

use redra_client::proto::{
    command::Command,
    target::TargetId,
    designation::{self, DesignCmd, Spawn, Update},
    shape::{self, ShapePack, Cube, Sphere, Pose},
    transform::{TransCmd, Translation, Rotation, Scale, TransformOptions},
};

use redra_client::client::sender::Sender;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init(); // 初始化日志
    
    info!("Redra Rust 客户端示例启动");

    // 连接到服务器
    info!("尝试连接到服务器 127.0.0.1:8080");
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let mut sender = Sender::new(stream);
    info!("成功连接到服务器");

    // 创建立方体
    let cube_cmd = create_cube_spawn_command();
    info!("发送立方体创建命令...");
    if let Err(e) = sender.send_command(cube_cmd).await {
        error!("发送立方体命令失败: {}", e);
    }

    // 创建球体
    let sphere_cmd = create_sphere_spawn_command();
    info!("发送球体创建命令...");
    if let Err(e) = sender.send_command(sphere_cmd).await {
        error!("发送球体命令失败: {}", e);
    }

    // 更新立方体位置
    let update_cube_cmd = create_cube_update_command();
    info!("发送立方体位置更新命令...");
    if let Err(e) = sender.send_command(update_cube_cmd).await {
        error!("发送立方体更新命令失败: {}", e);
    }

    // 更新球体位置
    let update_sphere_cmd = create_sphere_update_command();
    info!("发送球体位置和旋转更新命令...");
    if let Err(e) = sender.send_command(update_sphere_cmd).await {
        error!("发送球体更新命令失败: {}", e);
    }

    info!("所有命令已发送完毕");
    info!("客户端正常退出");
    
    Ok(())
}

/// 创建立方体的Spawn命令
fn create_cube_spawn_command() -> Command {
    let timestamp = get_current_timestamp();
    Command {
        cmd_pack: Some(redra_client::proto::command::command::CmdPack::Designation(DesignCmd {
            cmd: Some(designation::design_cmd::Cmd::Spawn(Spawn {
                id: Some(TargetId {
                    has_set: true,
                    id: 1,
                    uuid: "cube-1".to_string(),
                }),
                name: "Test Cube".to_string(),
                tags: vec!["example".to_string(), "cube".to_string()],
                data: Some(designation::spawn::Data::ShapeData(ShapePack {
                    data: Some(shape::shape_pack::Data::Cube(Cube {
                        translation: Some(Translation {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        }),
                        rotation: Some(Rotation {
                            rx: 0.0,
                            ry: 0.0,
                            rz: 0.0,
                        }),
                        scale: Some(Scale {
                            sx: 1.0,
                            sy: 1.0,
                            sz: 1.0,
                        }),
                        is_wireframe: false,
                    })),
                    name: "Red Cube".to_string(),
                    tags: vec!["red".to_string(), "basic".to_string()],
                    color: Some(redra_client::proto::shape::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                })),
                initial_pose: None,
            })),
        })),
        timestamp,
        command_id: format!("cmd_spawn_cube_{}", timestamp),
    }
}

/// 创建球体的Spawn命令
fn create_sphere_spawn_command() -> Command {
    let timestamp = get_current_timestamp();
    Command {
        cmd_pack: Some(redra_client::proto::command::command::CmdPack::Designation(DesignCmd {
            cmd: Some(designation::design_cmd::Cmd::Spawn(Spawn {
                id: Some(TargetId {
                    has_set: true,
                    id: 2,
                    uuid: "sphere-1".to_string(),
                }),
                name: "Test Sphere".to_string(),
                tags: vec!["example".to_string(), "sphere".to_string()],
                data: Some(designation::spawn::Data::ShapeData(ShapePack {
                    data: Some(shape::shape_pack::Data::Sphere(Sphere {
                        pos: Some(Translation {
                            x: 2.0,
                            y: 0.0,
                            z: 0.0,
                        }),
                        radius: 1.0,
                        segments: 32,
                    })),
                    name: "Blue Sphere".to_string(),
                    tags: vec!["blue".to_string(), "basic".to_string()],
                    color: Some(redra_client::proto::shape::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                })),
                initial_pose: None,
            })),
        })),
        timestamp,
        command_id: format!("cmd_spawn_sphere_{}", timestamp),
    }
}

/// 创建立方体更新命令
fn create_cube_update_command() -> Command {
    let timestamp = get_current_timestamp();
    Command {
        cmd_pack: Some(redra_client::proto::command::command::CmdPack::Designation(DesignCmd {
            cmd: Some(designation::design_cmd::Cmd::Update(Update {
                id: Some(TargetId {
                    has_set: true,
                    id: 1,
                    uuid: "cube-1".to_string(),
                }),
                data: Some(designation::update::Data::Pose(Pose {
                    translation: Some(Translation {
                        x: 3.0,
                        y: 1.0,
                        z: 1.0,
                    }),
                    rotation: Some(Rotation {
                        rx: 0.0,
                        ry: 0.0,
                        rz: 0.0,
                    }),
                    scale: Some(Scale {
                        sx: 1.0,
                        sy: 1.0,
                        sz: 1.0,
                    }),
                })),
            })),
        })),
        timestamp,
        command_id: format!("cmd_update_cube_{}", timestamp),
    }
}

/// 创建球体更新命令
fn create_sphere_update_command() -> Command {
    let timestamp = get_current_timestamp();
    Command {
        cmd_pack: Some(redra_client::proto::command::command::CmdPack::Designation(DesignCmd {
            cmd: Some(designation::design_cmd::Cmd::Update(Update {
                id: Some(TargetId {
                    has_set: true,
                    id: 2,
                    uuid: "sphere-1".to_string(),
                }),
                data: Some(designation::update::Data::Pose(Pose {
                    translation: Some(Translation {
                        x: 0.0,
                        y: 2.0,
                        z: 0.0,
                    }),
                    rotation: Some(Rotation {
                        rx: 0.785, // 45度
                        ry: 0.785,
                        rz: 0.0,
                    }),
                    scale: Some(Scale {
                        sx: 2.0,
                        sy: 2.0,
                        sz: 2.0,
                    }),
                })),
            })),
        })),
        timestamp,
        command_id: format!("cmd_update_sphere_{}", timestamp),
    }
}

/// 获取当前时间戳
fn get_current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}