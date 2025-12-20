use socket2::{Socket, Domain, Type};
use std::net::SocketAddr;

use tokio::{self, sync::{broadcast, mpsc::{self}}};

use crate::{parser::core::{RDPack}, net::server::RDServer};


pub struct RDListener {
    pub runtime: tokio::runtime::Runtime,
    pub sender: mpsc::Sender<RDPack>,
    pub receiver: broadcast::Receiver<RDPack>,
}

impl RDListener {
    pub fn new(
        sender: mpsc::Sender<RDPack>,
        receiver: broadcast::Receiver<RDPack>,
    ) -> RDListener {
        RDListener {
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            sender: sender,
            receiver: receiver,
        }
    }

    pub fn listen(&mut self, addr: &str) {
        let socket_addr: SocketAddr = addr.parse().expect("无效的地址格式");
        let domain = if socket_addr.is_ipv4() {
            Domain::IPV4
        } else {
            Domain::IPV6
        };
        
        let link = match Socket::new(domain, Type::STREAM, None) {
            Ok(link) => {
                if domain == Domain::IPV6 {
                    link.set_only_v6(false).expect("无法设置套接字为 IPv6 仅模式");
                }
                link
            },
            Err(e) => panic!("创建套接字错误: {:?}", e),
        };
        
        // 解析并绑定地址
        link.bind(&socket_addr.into()).expect("绑定地址失败");
        link.listen(128).expect("监听失败");
        
        println!("开始监听地址: {}", addr);
        
        let sender = self.sender.clone();
        let receiver = self.receiver.resubscribe(); 
        let handle = self.runtime.handle().clone();
        
        // 在单独的线程中运行监听循环
        std::thread::spawn(move || {
            handle.block_on(async move {
                loop {
                    match link.accept() {
                        Ok((socket, addr)) => {
                            println!("接受连接: {:?}", addr);
                            let mut server = RDServer::new(
                                socket,
                                sender.clone(),
                                receiver.resubscribe(),
                            );
                            server.run().await;
                        },
                        Err(e) => {
                            eprintln!("接受连接时出错: {:?}", e);
                        }
                    }
                }
            });
        });
    }
}