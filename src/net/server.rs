use prost::Message;
use socket2::{Domain, Socket, Type};
use tokio::{
    self,
    sync::{broadcast, mpsc},
};
use std::mem::MaybeUninit;
use std::sync::Arc;

use crate::{channel::core::{RDPack, SpawnPack}, parser::interface::cube_rdr, proto::{rdr, shape}};

pub struct RDServer {
    pub socket: Socket,
    pub sender: mpsc::Sender<RDPack>,
    pub receiver: broadcast::Receiver<RDPack>,
}

impl RDServer {
    pub fn new(
        socket: Socket,
        sender: mpsc::Sender<RDPack>,
        receiver: broadcast::Receiver<RDPack>,
    ) -> RDServer {
        RDServer {
            socket,
            sender,
            receiver,
        }
    }

    pub async fn run(&mut self) {
        let socket = self.socket.try_clone().expect("socket clone 失败");
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let mut buf = [MaybeUninit::uninit(); 1024];
            loop {
                match socket.recv(&mut buf) {
                    Ok(0) => {
                        // 连接已关闭
                        println!("连接已关闭");
                        break;
                    }
                    Ok(len) => {
                        let data = unsafe {
                            std::slice::from_raw_parts(buf.as_ptr() as *const u8, len)
                        };
                        match rdr::Pack::decode(data) {
                            Ok(pack) => {
                                process_pack(pack, sender.clone());
                            }
                            Err(e) => {
                                eprintln!("解码 Pack 失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("读取数据失败: {}", e);
                        break;
                    }
                }
            }
        });
    }
}

fn process_pack(pack: rdr::Pack, sender: mpsc::Sender<RDPack>) {
    match pack.data_type.as_str() {
        "position" => {
            // 将pack.data解析为Position消息
            match rdr::Position::decode(&pack.data[..]) {
                Ok(position) => {
                    // 处理Position数据
                    let _pos = position;
                }
                Err(e) => eprintln!("解析 Position 数据失败: {}", e),
            }
        }
        "rotation" => {
            // 将pack.data解析为Rotation消息
            match rdr::Rotation::decode(&pack.data[..]) {
                Ok(rotation) => {
                    // 处理Rotation数据
                    let _rot = rotation;
                }
                Err(e) => eprintln!("解析 Rotation 数据失败: {}", e),
            }
        }
        "scale" => {
            // 将pack.data解析为Scale消息
            match rdr::Scale::decode(&pack.data[..]) {
                Ok(scale) => {
                    // 处理Scale数据
                    let _scale = scale;
                }
                Err(e) => eprintln!("解析 Scale 数据失败: {}", e),
            }
        }
        "cube" => {
            // 将pack.data解析为Cube消息
            match shape::Cube::decode(&pack.data[..]) {
                Ok(cube) => {
                    let rdr_cube = cube_rdr(&cube);
                    // 创建Bevy网格和变换
                    let mesh = Arc::new(rdr_cube.to_mesh());
                    let transform = rdr_cube.pose();
                    
                    // 创建SpawnPack并发送到Bevy
                    let spawn_pack = SpawnPack {
                        mesh,
                        transform,
                        material: "default".to_string(),
                    };
                    
                    let rd_pack = RDPack::Spawn(Box::new(spawn_pack));
                    tokio::spawn(async move {
                        if let Err(e) = sender.send(rd_pack).await {
                            eprintln!("发送RDPack到Bevy失败: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("解析 Cube 数据失败: {}", e),
            }
        }
        _ => {
            eprintln!("未知的数据类型: {}", pack.data_type);
        }
    }
}