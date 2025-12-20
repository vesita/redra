use prost::Message;
use socket2::Socket;
use tokio::{
    self,
    sync::{broadcast, mpsc},
};
use std::mem::MaybeUninit;

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
        
        // 创建一个channel用于处理接收到的数据包
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);
        
        // 数据接收任务
        let recv_socket = socket.try_clone().expect("socket clone 失败");
        tokio::spawn(async move {
            let mut buf = [MaybeUninit::uninit(); 1024];
            loop {
                match recv_socket.recv(&mut buf) {
                    Ok(0) => {
                        // 连接已关闭
                        println!("连接已关闭");
                        break;
                    }
                    Ok(len) => {
                        let data = unsafe {
                            std::slice::from_raw_parts(buf.as_ptr() as *const u8, len)
                        }.to_vec();
                        
                        // 发送到处理channel
                        if let Err(e) = tx.send(data).await {
                            eprintln!("发送数据到处理队列失败: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("读取数据失败: {}", e);
                        continue;
                    }
                }
            }
        });
        
        // 数据处理任务
        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                match rdr::Pack::decode(&data[..]) {
                    Ok(pack) => {
                        process_pack(pack, sender.clone());
                    }
                    Err(e) => {
                        eprintln!("解码 Pack 失败: {}", e);
                    }
                }
            }
        });
    }
}