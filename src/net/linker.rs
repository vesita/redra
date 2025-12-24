use std::time::Duration;

use log::{error, info, warn};
use prost::Message;
use tokio::{io::AsyncReadExt, net::TcpStream, sync::{broadcast, mpsc}, time::{sleep, timeout}};

use crate::proto::declare;

pub struct RDLinker {
    pub sender: mpsc::Sender<Vec<u8>>,
    pub receiver: mpsc::Receiver<Vec<u8>>,
    pub socket: TcpStream,
}

impl RDLinker {
    pub fn new(
        sender: mpsc::Sender<Vec<u8>>,
        receiver: mpsc::Receiver<Vec<u8>>,
        socket: TcpStream,
    ) -> RDLinker {
        RDLinker {
            sender,
            receiver,
            socket,
        }
    }
    
    pub async fn run(&mut self) {
        info!("启动TCP链接处理器");
        
        let mut total_bytes_received = 0;
        let mut packets_received = 0;

        loop {
            let mut buffer = [0; 1024];
            match self.socket.read(&mut buffer).await {
                Ok(0) => {
                    // info!("接收到0字节，连接可能已关闭");
                    let _ = sleep(Duration::from_millis(10));
                },
                Ok(len) => {
                    total_bytes_received += len;
                    packets_received += 1;
                    
                    info!("从TCP连接接收到 {} 字节数据，累计接收: {} 字节，数据包序号: {}", 
                            len, total_bytes_received, packets_received);
                    
                    let data = (&buffer[..len]).to_vec();
                    
                    // 发送到处理channel
                    if let Err(e) = self.sender.send(data).await {
                        error!("发送数据到处理队列失败: {} (数据包序号 #{})", e, packets_received);
                        break;
                    }
                },
                Err(e) => {
                    error!("从TCP连接读取数据失败: {}", e);
                    break;
                }
            }
        }
        info!("TCP链接处理器任务结束，总计处理 {} 字节，{} 个数据包", total_bytes_received, packets_received);
    }

    // async fn response(&mut self) {
    //     info!("开始发送握手请求");
        
    //     // 发送握手请求，带重试机制
    //     let mut attempts = 0;
    //     let max_attempts = 3;
    //     while attempts < max_attempts {
    //         info!("尝试发送握手请求 (尝试 {}/{})", attempts + 1, max_attempts);
    //         if let Err(e) = self.sender.send("Ready?".as_bytes().to_vec()).await {
    //             error!("发送握手请求失败 (尝试 {}/{}): {}", attempts + 1, max_attempts, e);
    //             attempts += 1;
    //             tokio::time::sleep(Duration::from_millis(100)).await;
    //             continue;
    //         }
    //         info!("握手请求已发送");
    //         break;
    //     }
        
    //     if attempts >= max_attempts {
    //         info!("达到最大重试次数，跳过握手");
    //         return;
    //     }
        
    //     info!("等待RDForwarder响应");
        
    //     // 使用超时机制避免无限等待
    //     match timeout(Duration::from_secs(5), self.receiver.recv()).await {
    //         Ok(Some(data)) => {
    //             info!("RDForwarder响应: {}", String::from_utf8_lossy(&data));
    //             if data == "Ready".as_bytes() {
    //                 info!("握手成功");
    //             } else {
    //                 warn!("握手响应错误: 期望 'Ready'，收到 '{}'", String::from_utf8_lossy(&data));
    //             }
    //         }
    //         Ok(None) => {
    //             warn!("握手响应为空");
    //         }
    //         Err(_) => {
    //             warn!("握手超时，客户端可能直接发送数据");
    //         }
    //     }
    // }
}
