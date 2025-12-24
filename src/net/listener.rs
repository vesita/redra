use std::{collections::VecDeque, net::SocketAddr, time::Duration};

use log::{error, info};
use tokio::{net::TcpListener as TokioTcpListener, sync::{broadcast, mpsc}, task, time::sleep};

use crate::{net::{forwarder::RDForwarder, linker::RDLinker, }, parser::core::RDPack};

pub struct RDListener {
    pub linkers: Vec<tokio::task::JoinHandle<()>>,
    pub forwarders: Vec<tokio::task::JoinHandle<()>>,
    pub released: VecDeque<usize>,
    pub to_engine: mpsc::Sender<RDPack>,
    pub engine_broadcast: broadcast::Receiver<RDPack>,
    address: String,
}

impl RDListener {
    pub fn new(
        to_engine: mpsc::Sender<RDPack>,
        engine_broadcast: broadcast::Receiver<RDPack>,
    ) -> Self {
        RDListener {
            linkers: vec![],
            forwarders: vec![],
            released: VecDeque::new(),
            to_engine,
            engine_broadcast,
            address: "0.0.0.0:8080".to_string(),
        }
    }

    pub async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) {
        let socket_addr: SocketAddr = match self.address.parse() {
            Ok(addr) => addr,
            Err(e) => {
                error!("无效的地址格式 '{}': {}", self.address, e);
                return;
            }
        };
        
        let listener = match TokioTcpListener::bind(socket_addr).await {
            Ok(listener) => {
                info!("开始监听网络地址: {}", self.address);
                listener
            },
            Err(e) => {
                error!("绑定网络地址失败: {}", e);
                return;
            }
        };
        
        let mut connection_id: usize = 0;
        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("接受新的客户端连接: {}", addr);
                    connection_id += 1;
                    let (link_tx, forward_rx) = mpsc::channel::<Vec<u8>>(1024);
                    let (forward_tx, link_rx) = mpsc::channel::<Vec<u8>>(1024);
                    
                    let mut linker = RDLinker::new(link_tx, link_rx, socket);
                    let mut forwarder = RDForwarder::new(
                        forward_tx,
                        forward_rx,
                        self.to_engine.clone(),
                        self.engine_broadcast.resubscribe()
                    );
                    let forwarder_task = task::spawn(async move {
                        info!("启动转发任务 - 连接ID: {}", connection_id);
                        forwarder.run().await;
                    });
                    info!("创建连接 - ID: {}", connection_id);
                    let linker_task = task::spawn(async move {
                        info!("启动链接任务 - 连接ID: {}", connection_id);
                        linker.run().await;
                    });
                    
                    self.linkers.push(linker_task);
                    self.forwarders.push(forwarder_task);
                },
                Err(e) => {
                    error!("接受客户端连接时出错: {}", e);
                }
            }
            let _ = sleep(Duration::from_millis(100));
            if let Ok(()) = shutdown.try_recv() {
                info!("收到关闭信号，正在停止所有连接任务...");
                for task in self.linkers.drain(..) {
                    task.abort();
                }
                for task in self.forwarders.drain(..) {
                    task.abort();
                }
                info!("已停止所有网络任务");
            }
        }
    }
}