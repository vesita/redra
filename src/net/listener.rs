use std::{collections::VecDeque, net::SocketAddr, sync::{Arc, Mutex}};

use log::{error, info};
use tokio::{net::{TcpListener as TokioTcpListener, TcpStream}, sync::{broadcast, mpsc}, task};

use crate::{ThLc, net::{forwarder::RDForwarder, linker::RDLinker, work_share::{RDWosh}}, module::parser::core::RDPack};

pub struct RDListener {
    pub linkers: Vec<Option<tokio::task::JoinHandle<()>>>,
    pub forwarders: Vec<Option<tokio::task::JoinHandle<()>>>,
    pub linker_ids: VecDeque<usize>,    // 存储可用ID的有序集合
    pub forwarder_ids: VecDeque<usize>,
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
            linkers: Vec::new(),
            forwarders: Vec::new(),
            linker_ids: VecDeque::new(),  // 初始化可用ID集合
            forwarder_ids: VecDeque::new(),
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
        
        let (linker_rls_tx, mut release_rx) = mpsc::channel::<usize>(32); // 使用有界通道，容量为32
        let (forwarder_rls_tx, mut forwarder_rls_rx) = mpsc::channel::<usize>(32);
        let (expand_tx, mut expand_rx) = mpsc::channel::<usize>(32);


        let wosh = Arc::new(Mutex::new(RDWosh::new()));

        self.init(forwarder_rls_tx.clone(), wosh.clone());

        loop {
            tokio::select! {
                // 接受新连接
                result = listener.accept() => {
                    match result {
                        Ok((socket, addr)) => {
                            info!("接受新的客户端连接: {}", addr);
                            self.create_linker(socket, wosh.clone(), &linker_rls_tx, expand_tx.clone());
                        },
                        Err(e) => {
                            error!("接受客户端连接时出错: {}", e);
                        }
                    }
                },
                
                // 处理连接释放
                Some(released_id) = release_rx.recv() => {
                    info!("释放连接 - ID: {}", released_id);
                    self.release_linker(released_id);
                },

                Some(released_id) = forwarder_rls_rx.recv() => {
                    info!("释放转发任务 - ID: {}", released_id);
                    self.release_forwarder(released_id);
                },
                
                Some(expand_request) = expand_rx.recv() => {
                    info!("扩展连接 - ID: {}", expand_request);
                    let (sender, receiver) = mpsc::channel::<Vec<u8>>(32);
                    wosh.lock().unwrap().add_channel(sender, expand_request);
                    self.create_forwarder(
                        receiver,
                        forwarder_rls_tx.clone(),
                    );
                }
            }
        }
    }
    
    fn init(
        &mut self,
        release: mpsc::Sender<usize>,
        wosh: ThLc<RDWosh>,
    ) {
        for _ in 0..3 {
            let (sender, receiver) = mpsc::channel::<Vec<u8>>(32);
            let id = self.create_forwarder(receiver, release.clone());
            wosh.lock().unwrap().add_channel(sender, id);
        }
    }

    fn create_linker(
        &mut self,
        socket: TcpStream,
        wosh: ThLc<RDWosh>,
        release_tx: &mpsc::Sender<usize>,
        expand_tx: mpsc::Sender<usize>,
    ) {
        let linker_id = self.available_linker_id();
        let mut linker = RDLinker::new(linker_id, socket, wosh, expand_tx);
        let release_tx = release_tx.clone();
        let linker_task = task::spawn(async move {
            info!("启动链接任务 - 链接ID: {}", linker_id);
            linker.run(release_tx).await;
        });
        self.linkers[linker_id] = Some(linker_task);
    }
    
    fn create_forwarder(
        &mut self,
        sender: mpsc::Receiver<Vec<u8>>,
        release: mpsc::Sender<usize>,
    ) -> usize {
        let forwarder_id = self.available_forwarder_id();
        let mut forwarder = RDForwarder::new(forwarder_id, sender, self.to_engine.clone());
        let forwarder_task = task::spawn(async move {
            info!("启动转发任务 - 转发ID: {}", forwarder_id);
            forwarder.run(release).await;
        });
        self.forwarders[forwarder_id] = Some(forwarder_task);
        forwarder_id
    }
    
    // 释放连接并回收ID
    fn release_linker(&mut self, id: usize) {
        // 检查是否是linker被释放
        self.linkers[id] = None;
        // 将ID加入可用ID集合，以便重用
        self.linker_ids.push_back(id);
    }
    
    fn available_linker_id(&mut self) -> usize {
        if let Some(id) = self.linker_ids.pop_front() {
            id
        } else {
            self.linkers.resize_with(self.linkers.len() + 1, || None);
            self.linkers.len() - 1
        }
    }

    fn release_forwarder(&mut self, id: usize) {
        // 检查是否是forwarder被释放
        self.forwarders[id] = None;
        
        // 将ID加入可用ID集合，以便重用
        self.forwarder_ids.push_back(id);
    }

    fn available_forwarder_id(&mut self) -> usize {
        if let Some(id) = self.forwarder_ids.pop_front() {
            id
        } else {
            self.forwarders.resize_with(self.forwarders.len() + 1, || None);
            self.forwarders.len() - 1
        }
    }
}