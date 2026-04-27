use std::{
    net::SocketAddr, time::Duration
};

use expto::{ip::get_addr, rdmp::Unit};
use log::{error, info, warn};
use tokio::{
    net::TcpListener, sync::{broadcast, mpsc}, time::sleep
};

use bevy::prelude::*;
use utils::ShareID;

use crate::{RDChannel, NetworkStatus, linker::start_linker};


/// 网络监听器服务
/// 
/// 负责管理TCP连接的生命周期，包括接受新连接、分配ID和管理连接任务
pub struct NetworkListenerService {
    listener: TcpListener,
    /// 用于向Bevy引擎发送解析后的Unit数据
    to_engine_sender: mpsc::Sender<Unit>,
}

impl NetworkListenerService {
    /// 创建新的网络监听器服务
    /// 
    /// # 参数
    /// * `address` - 监听的网络地址
    /// * `to_engine_sender` - 发送到Bevy引擎的通道发送端（对应 RDChannel.redra_recver）
    pub async fn new(
        address: &str,
        to_engine_sender: mpsc::Sender<Unit>,
    ) -> Result<Self, String> {
        let socket_addr: SocketAddr = address.parse()
            .map_err(|e| format!("无效的地址格式 '{}': {}", address, e))?;
        
        let listener = TcpListener::bind(socket_addr).await
            .map_err(|e| format!("绑定网络地址失败: {}", e))?;
        
        info!("成功绑定到地址: {}", address);
        
        Ok(Self {
            listener,
            to_engine_sender,
        })
    }
    
    /// 启动监听器服务的主循环
    pub async fn run(self) {
        info!("启动监听器服务");
        
        let mut id_pool = ShareID::new();
        let (release, mut holder) = mpsc::channel(64);
        
        loop {
            tokio::select! {
                // 接受新连接
                result = self.listener.accept() => {
                    match result {
                        Ok((socket, addr)) => {
                            info!("接受新的客户端连接: {}", addr);
                            
                            let sender = self.to_engine_sender.clone();
                            let id = id_pool.get_id();
                            let release_copy = release.clone();
                            
                            tokio::spawn(async move {
                                info!("启动Linker任务, ID: {}", id);
                                // 创建一个虚拟的 broadcast receiver，因为 start_linker 需要它
                                let (dummy_sender, _) = broadcast::channel::<Unit>(1);
                                start_linker(id, release_copy, socket, sender, dummy_sender.subscribe()).await;
                                info!("Linker任务结束, ID: {}", id);
                            });
                        },
                        Err(e) => {
                            error!("接受客户端连接时出错: {}", e);
                        }
                    }
                },
                
                // 处理ID回收
                id = holder.recv() => {
                    match id {
                        Some(id) => {
                            info!("回收ID: {}", id);
                            id_pool.release(id);
                        },
                        None => {
                            warn!("ID回收通道已关闭");
                        }
                    }
                },

                // 定期yield，防止饥饿
                _ = sleep(Duration::from_millis(50)) => {
                    // 允许其他异步任务运行
                }
            }
        }
    }
}

// Bevy的setup函数,用于初始化资源
pub fn setup_listener(
    mut command: Commands,
) {
    info!("开始初始化网络监听器");
    
    let address = get_addr();
    info!("目标监听地址: {}", address);

    // 创建Bevy引擎与网络模块之间的通信通道
    let (redra_sender, _link_recver) = broadcast::channel::<Unit>(1024);
    let (link_sender, redra_recver) = mpsc::channel::<Unit>(1024);

    // 插入通道资源，供其他系统使用
    command.insert_resource(RDChannel {
        redra_sender,
        redra_recver,
    });
    
    // 初始化网络状态
    command.insert_resource(NetworkStatus::default());

    // 在独立的Tokio运行时中启动网络监听器服务
    info!("启动网络监听器服务...");
    
    let address_clone = address.clone();
    std::thread::Builder::new()
        .name("redra-network-listener".to_string())
        .spawn(move || {
            info!("网络监听线程开始执行");
            
            let rt = tokio::runtime::Runtime::new()
                .expect("无法创建网络监听的Tokio运行时");
            
            rt.block_on(async move {
                // 关键修复：传入Bevy资源的通道，而不是创建新的
                match NetworkListenerService::new(
                    &address_clone,
                    link_sender,  // 发送到 RDChannel.redra_recver
                ).await {
                    Ok(service) => {
                        info!("服务初始化成功，开始运行");
                        service.run().await;
                    },
                    Err(e) => {
                        error!("服务初始化失败: {}", e);
                        panic!("网络监听器启动失败: {}", e);
                    }
                }
            });
            
            info!("网络监听线程结束");
        })
        .expect("无法创建网络监听线程");
    
    info!("网络监听线程已启动");
}