use std::{collections::{HashMap, BTreeSet}, net::SocketAddr, time::Duration};

use log::{error, info};
use tokio::{net::TcpListener as TokioTcpListener, sync::{broadcast, mpsc}, task, time::sleep};

use crate::{net::{forwarder::RDForwarder, linker::RDLinker, }, parser::core::RDPack};

pub struct RDListener {
    pub linkers: HashMap<usize, tokio::task::JoinHandle<()>>,
    pub forwarders: HashMap<usize, tokio::task::JoinHandle<()>>,
    pub available_ids: BTreeSet<usize>,    // 存储可用ID的有序集合
    pub next_id: usize,                    // 下一个新ID
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
            linkers: HashMap::new(),
            forwarders: HashMap::new(),
            available_ids: BTreeSet::new(),  // 初始化可用ID集合
            next_id: 0,                      // 初始化下一个ID
            to_engine,
            engine_broadcast,
            address: "0.0.0.0:8080".to_string(),
        }
    }

    pub async fn run(&mut self) {
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
        
        let (release_tx, mut release_rx) = mpsc::channel::<usize>(32); // 使用有界通道，容量为32

        loop {
            tokio::select! {
                // 接受新连接
                result = listener.accept() => {
                    match result {
                        Ok((socket, addr)) => {
                            info!("接受新的客户端连接: {}", addr);
                            
                            // 获取ID - 优先使用回收的ID，然后是新ID
                            let connection_id = self.get_next_id();
                            
                            let (link_tx, forward_rx) = mpsc::channel::<Vec<u8>>(1024);
                            let (forward_tx, link_rx) = mpsc::channel::<Vec<u8>>(1024);

                            let mut linker = RDLinker::new(connection_id, link_tx, link_rx, socket);
                            let mut forwarder = RDForwarder::new(
                                connection_id,
                                forward_tx,
                                forward_rx,
                                self.to_engine.clone(),
                                self.engine_broadcast.resubscribe()
                            );
                            
                            info!("创建连接 - ID: {}", connection_id);
                            
                            let linker_release = release_tx.clone();
                            let linker_task = task::spawn(async move {
                                info!("启动链接任务 - 连接ID: {}", connection_id);
                                linker.run(linker_release).await;
                            });
                            
                            let forwarder_release = release_tx.clone();
                            let forwarder_task = task::spawn(async move {
                                info!("启动转发任务 - 连接ID: {}", connection_id);
                                forwarder.run(forwarder_release).await;
                            });

                            self.linkers.insert(connection_id, linker_task);
                            self.forwarders.insert(connection_id, forwarder_task);
                        },
                        Err(e) => {
                            error!("接受客户端连接时出错: {}", e);
                        }
                    }
                },
                
                // 处理连接释放
                Some(released_id) = release_rx.recv() => {
                    info!("释放连接 - ID: {}", released_id);
                    self.release_connection(released_id);
                }
            }
        }
    }
    
    // 获取下一个可用ID，优先使用回收的ID
    fn get_next_id(&mut self) -> usize {
        if let Some(&id) = self.available_ids.iter().next() {
            // 获取最小的可用ID
            self.available_ids.remove(&id);
            id
        } else {
            // 没有可用ID，使用下一个新ID
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }
    
    // 释放连接并回收ID
    fn release_connection(&mut self, id: usize) {
        // 移除对应的任务句柄
        self.linkers.remove(&id);
        self.forwarders.remove(&id);
        
        // 将ID加入可用ID集合
        self.available_ids.insert(id);
    }
}