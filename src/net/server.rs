use prost::Message;
use socket2::{Domain, Socket, Type};
use tokio::{
    self,
    sync::{broadcast, mpsc},
};
use std::mem::MaybeUninit;
use std::sync::Arc;

use crate::{parser::{core::RDPack, proto::process_pack}, proto::rdr};

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

