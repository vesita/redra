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


// Bevy的setup函数,用于初始化资源
pub fn setup_listener(
    mut command: Commands,
) {
    info!("[NetworkPlugin] 开始初始化网络监听器");
    
    let address = get_addr();
    info!("[NetworkPlugin] 目标监听地址: {}", address);
    
    let socket_addr: SocketAddr = match address.parse() {
        Ok(addr) => addr,
        Err(e) => {
            error!("[NetworkPlugin] 无效的地址格式 '{}': {}", address, e);
            panic!("无法解析地址");
        }
    };

    let (redra_sender, link_recver) = broadcast::channel::<Unit>(1024);
    let (link_sender, redra_recver) = mpsc::channel::<Unit>(1024);

    command.insert_resource(
        RDChannel {
            redra_sender,
            redra_recver,
        }
    );
    
    // 初始化网络状态
    command.insert_resource(NetworkStatus::default());

    // 使用独立线程和Tokio运行时运行整个网络监听逻辑
    info!("[NetworkPlugin] 在独立线程中启动完整网络栈...");
    let _ = std::thread::Builder::new()
        .name("redra-network-listener".to_string())
        .spawn(move || {
            info!("[listener_thread] 网络监听线程开始执行");
            
            // 在线程内部创建完整的Tokio运行时
            let rt = tokio::runtime::Runtime::new().expect("无法创建网络监听的Tokio运行时");
            
            rt.block_on(async move {
                info!("[listener_thread] 尝试绑定TCP监听器到 {}...", address);
                
                match TcpListener::bind(socket_addr).await {
                    Ok(listener) => {
                        info!("[listener_thread] 成功绑定到地址: {}", address);
                        
                        // 启动监听任务
                        listener_task(listener, link_sender, link_recver).await;
                    }
                    Err(e) => {
                        error!("[listener_thread] 绑定网络地址失败: {}", e);
                        panic!("无法绑定到地址: {}", e);
                    }
                }
            });
            
            info!("[listener_thread] 网络监听线程结束");
        })
        .expect("无法创建网络监听线程");
    
    info!("[NetworkPlugin] 网络监听线程已启动");
}


/// 启动网络监听器,开始接受客户端连接
/// 
/// 该方法会创建必要的通道和共享资源,然后进入事件循环,
/// 处理新连接、连接释放和任务扩展等事件
pub async fn listener_task(
    listener: TcpListener,
    sender: mpsc::Sender<Unit>,
    receiver: broadcast::Receiver<Unit>,
) {
    info!("[listener_task] 初始化ID池和通道");
    let mut id_pool = ShareID::new();
    let (release, mut holder) = mpsc::channel(64);
    
    info!("[listener_task] 进入主事件循环");
    loop {
        tokio::select! {
            // 接受新连接
            result = listener.accept() => {
                match result {
                    Ok((socket, addr)) => {
                        info!("[listener_task] 接受新的客户端连接: {}", addr);
                        let sender = sender.clone();
                        let receiver = receiver.resubscribe();
                        let id = id_pool.get_id();
                        let release_copy = release.clone();
                        tokio::spawn({
                            async move {
                                info!("[linker] 启动Linker任务, ID: {}", id);
                                start_linker(id, release_copy, socket, sender, receiver).await;
                                info!("[linker] Linker任务结束, ID: {}", id);
                            }
                        });
                    },
                    Err(e) => {
                        error!("[listener_task] 接受客户端连接时出错: {}", e);
                    }
                }
            },
            
            id = holder.recv() => {
                match id {
                    Some(id) => {
                        info!("[listener_task] 回收ID: {}", id);
                        id_pool.release(id);
                    },
                    None => {
                        warn!("[listener_task] ID回收通道已关闭");
                    }
                }
            },

            // 添加一个定时器,防止在没有事件时过度占用CPU
            _ = sleep(Duration::from_millis(50)) => {
                // 这个分支会在每毫秒触发一次,确保循环不会阻塞在select上
                // 在高负载情况下,其他分支会更频繁地被触发
                // 在低负载情况下,这个分支会定期触发,允许其他异步任务运行
            }
        }
    }
}