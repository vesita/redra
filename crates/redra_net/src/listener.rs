use std::{
    net::SocketAddr, time::Duration
};

use expto::{ip::get_addr, rdmp::Unit};
use log::{error, info};
use tokio::{
    net::TcpListener, runtime::Runtime, sync::{broadcast, mpsc}, time::sleep
};

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use utils::ShareID;

use crate::{RDChannel, linker::start_linker};


// Bevy的setup函数，用于初始化资源
pub fn setup_listener(
    mut command: Commands,
) {
    let address = get_addr();
    let socket_addr: SocketAddr = match address.parse() {
        Ok(addr) => addr,
        Err(e) => {
            error!("无效的地址格式 '{}': {}", address, e);
            panic!("无法解析地址");
        }
    };
    // 由于Tokio监听器的创建是异步的，我们需要在另一个任务中创建它
    // 或者我们可以使用阻塞操作
    let runtime = tokio::runtime::Runtime::new().expect("无法创建Tokio运行时");
    let listener = runtime.block_on(async {
        match TcpListener::bind(socket_addr).await {
            Ok(listener) => {
                info!("开始监听网络地址: {}", address);
                listener
            }
            Err(e) => {
                error!("绑定网络地址失败: {}", e);
                panic!("无法绑定到地址");
            }
        }
    });
    let (redra_sender, link_recver) = broadcast::channel::<Unit>(1024);
    let (link_sender, redra_recver) = mpsc::channel::<Unit>(1024);

    command.insert_resource(
        RDChannel {
            redra_sender,
            redra_recver,
        }
    );

    let _ = AsyncComputeTaskPool::get()
        .spawn(async move {
            std::thread::spawn(move || {
                // 创建独立的Tokio运行时
                let rt = Runtime::new().expect("Failed to create Tokio runtime");
                rt.block_on(async move {
                    tokio::spawn(listener_task(listener, link_sender, link_recver));
                });
            });
        });
}


/// 启动网络监听器，开始接受客户端连接
/// 
/// 该方法会创建必要的通道和共享资源，然后进入事件循环，
/// 处理新连接、连接释放和任务扩展等事件
pub async fn listener_task(
    listener: TcpListener,
    sender: mpsc::Sender<Unit>,
    receiver: broadcast::Receiver<Unit>,
) {
    let mut id_pool = ShareID::new();
    let (release, mut holder) = mpsc::channel(64);
    loop {
        tokio::select! {
            // 接受新连接
            result = listener.accept() => {
                match result {
                    Ok((socket, addr)) => {
                        info!("接受新的客户端连接: {}", addr);
                        let sender = sender.clone();
                        let receiver = receiver.resubscribe();
                        let id = id_pool.get_id();
                        let release_copy = release.clone();
                        tokio::spawn({
                            async move {
                                start_linker(id, release_copy, socket, sender, receiver).await;
                            }
                        });
                    },
                    Err(e) => {
                        error!("接受客户端连接时出错: {}", e);
                    }
                }
            },
            
            id = holder.recv() => {
                match id {
                    Some(id) => {
                        id_pool.release(id);
                    },
                    None => {}
                }
            },

            // 添加一个定时器，防止在没有事件时过度占用CPU
            _ = sleep(Duration::from_millis(500)) => {
                // 这个分支会在每毫秒触发一次，确保循环不会阻塞在select上
                // 在高负载情况下，其他分支会更频繁地被触发
                // 在低负载情况下，这个分支会定期触发，允许其他异步任务运行
            }
        }
    }
}