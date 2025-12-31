use std::{
    collections::VecDeque,
    net::SocketAddr,
    sync::Arc,
};

use log::{error, info};
use tokio::{
    net::{TcpListener as TokioTcpListener, TcpStream},
    sync::{Mutex, broadcast, mpsc},
    task,
};

use crate::{
    ThLc,
    module::parser::core::RDPack,
    net::{forwarder::RDForwarder, linker::RDLinker, work_share::RDWosh},
};

/// 网络监听器，负责处理客户端连接、管理连接任务和转发任务
/// 
/// 该结构体管理所有TCP连接和数据转发任务，并提供ID重用机制以优化资源使用
pub struct RDListener {
    /// 存储连接任务的向量，使用Option包装以支持任务的动态创建和释放
    pub linkers: Vec<Option<tokio::task::JoinHandle<()>>>,
    /// 存储转发任务的向量，使用Option包装以支持任务的动态创建和释放
    pub forwarders: Vec<Option<tokio::task::JoinHandle<()>>>,
    /// 存储可用连接ID的有序集合，用于ID重用
    pub linker_ids: VecDeque<usize>,
    /// 存储可用转发ID的有序集合，用于ID重用
    pub forwarder_ids: VecDeque<usize>,
    /// 发送数据到引擎的通道发送器
    pub to_engine: mpsc::Sender<RDPack>,
    /// 从引擎接收广播数据的接收器
    pub engine_broadcast: broadcast::Receiver<RDPack>,
    /// 监听的网络地址
    address: String,
}

impl RDListener {
    /// 创建一个新的网络监听器实例
    /// 
    /// # 参数
    /// * `to_engine` - 用于向引擎发送数据的发送器
    /// * `engine_broadcast` - 用于从引擎接收广播的接收器
    /// 
    /// # 返回值
    /// * `RDListener` - 新创建的监听器实例
    pub fn new(
        to_engine: mpsc::Sender<RDPack>,
        engine_broadcast: broadcast::Receiver<RDPack>,
    ) -> Self {
        RDListener {
            linkers: Vec::new(),
            forwarders: Vec::new(),
            linker_ids: VecDeque::new(), // 初始化可用ID集合
            forwarder_ids: VecDeque::new(),
            to_engine,
            engine_broadcast,
            address: "0.0.0.0:8080".to_string(),
        }
    }

    /// 启动网络监听器，开始接受客户端连接
    /// 
    /// 该方法会创建必要的通道和共享资源，然后进入事件循环，
    /// 处理新连接、连接释放和任务扩展等事件
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
            }
            Err(e) => {
                error!("绑定网络地址失败: {}", e);
                return;
            }
        };

        // 创建用于处理连接释放的通道
        let (linker_rls_tx, mut release_rx) = mpsc::channel::<usize>(32); // 使用有界通道，容量为32
        let (forwarder_rls_tx, mut forwarder_rls_rx) = mpsc::channel::<usize>(32);
        // 创建用于扩展连接的通道
        let (expand_tx, mut expand_rx) = mpsc::channel::<usize>(32);

        // 创建共享的工作分配器
        let wosh = Arc::new(Mutex::new(RDWosh::new()));

        // 初始化转发器
        self.init(forwarder_rls_tx.clone(), wosh.clone()).await;

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
                    wosh.lock().await.add_channel(sender, expand_request).await;
                    self.create_forwarder(
                        receiver,
                        forwarder_rls_tx.clone(),
                    );
                }
            }
        }
    }

    /// 初始化转发器
    /// 
    /// 创建指定数量的转发器任务，并将它们与工作分配器关联
    /// 
    /// # 参数
    /// * `release` - 用于发送转发器释放通知的发送器
    /// * `wosh` - 工作分配器的共享引用
    async fn init(&mut self, release: mpsc::Sender<usize>, wosh: ThLc<RDWosh>) {
        for _ in 0..3 {
            let (sender, receiver) = mpsc::channel::<Vec<u8>>(32);
            let id = self.create_forwarder(receiver, release.clone());
            wosh.lock().await.add_channel(sender, id).await;
        }
    }

    /// 创建新的连接处理任务
    /// 
    /// 为新接受的TCP连接创建一个连接处理任务，并将其存储在管理向量中
    /// 
    /// # 参数
    /// * `socket` - TCP连接的套接字
    /// * `wosh` - 工作分配器的共享引用
    /// * `release_tx` - 用于发送连接释放通知的发送器
    /// * `expand_tx` - 用于发送扩展请求的发送器
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

    /// 创建新的数据转发任务
    /// 
    /// 创建一个转发器任务，负责将数据从接收器转发到引擎
    /// 
    /// # 参数
    /// * `sender` - 接收待转发数据的接收器
    /// * `release` - 用于发送转发器释放通知的发送器
    /// 
    /// # 返回值
    /// * `usize` - 新创建的转发器的ID
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

    /// 释放指定ID的连接任务并回收ID
    /// 
    /// 将指定ID的连接任务标记为None，并将ID添加到可用ID集合中以供重用
    /// 
    /// # 参数
    /// * `id` - 要释放的连接任务的ID
    fn release_linker(&mut self, id: usize) {
        // 检查是否是linker被释放
        self.linkers[id] = None;
        // 将ID加入可用ID集合，以便重用
        self.linker_ids.push_back(id);
    }

    /// 获取一个可用的连接ID
    /// 
    /// 如果有已释放的ID可用，则重用它；否则，扩展向量并返回新ID
    /// 
    /// # 返回值
    /// * `usize` - 可用的连接ID
    fn available_linker_id(&mut self) -> usize {
        if let Some(id) = self.linker_ids.pop_front() {
            id
        } else {
            self.linkers.resize_with(self.linkers.len() + 1, || None);
            self.linkers.len() - 1
        }
    }

    /// 释放指定ID的转发任务并回收ID
    /// 
    /// 将指定ID的转发任务标记为None，并将ID添加到可用ID集合中以供重用
    /// 
    /// # 参数
    /// * `id` - 要释放的转发任务的ID
    fn release_forwarder(&mut self, id: usize) {
        // 检查是否是forwarder被释放
        self.forwarders[id] = None;

        // 将ID加入可用ID集合，以便重用
        self.forwarder_ids.push_back(id);
    }

    /// 获取一个可用的转发ID
    /// 
    /// 如果有已释放的ID可用，则重用它；否则，扩展向量并返回新ID
    /// 
    /// # 返回值
    /// * `usize` - 可用的转发ID
    fn available_forwarder_id(&mut self) -> usize {
        if let Some(id) = self.forwarder_ids.pop_front() {
            id
        } else {
            self.forwarders
                .resize_with(self.forwarders.len() + 1, || None);
            self.forwarders.len() - 1
        }
    }
}