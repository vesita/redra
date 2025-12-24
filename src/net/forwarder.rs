use bevy::asset::ron::error;
use log::{debug, error, info, trace};
use prost::Message;
use std::{mem::MaybeUninit, time::Duration};
use tokio::{
    self,
    sync::{broadcast, mpsc, oneshot}, time::{sleep, timeout},
};

use crate::{
    parser::{core::RDPack, proto::process_pack},
    proto::rdr,
};

pub struct RDForwarder {
    pub sender: mpsc::Sender<Vec<u8>>,
    pub receiver: mpsc::Receiver<Vec<u8>>,
    pub forward_sender: mpsc::Sender<RDPack>,
    pub forward_receiver: broadcast::Receiver<RDPack>,
}

impl RDForwarder {
    pub fn new(
        sender: mpsc::Sender<Vec<u8>>,
        receiver: mpsc::Receiver<Vec<u8>>,
        forward_sender: mpsc::Sender<RDPack>,
        forward_receiver: broadcast::Receiver<RDPack>,
    ) -> RDForwarder {
        RDForwarder {
            sender,
            receiver,
            forward_sender,
            forward_receiver,
        }
    }

    pub async fn run(&mut self) {
        let mut packets_processed = 0;
        let mut buffer: Vec<u8> = Vec::new();
        info!("waiting for ready signal");
        self.response().await;
        info!("开始数据处理任务");
        loop {
            match self.receiver.recv().await {
                Some(data) => {
                    // if data.is_empty() {
                    //     let _ = sleep(Duration::from_millis(10));
                    //     continue;
                    // }
                    // 将新接收的数据追加到缓冲区
                    buffer.extend_from_slice(&data);
                    info!("接收到 {} 字节数据", buffer.len());
                    let mut current_consumed = 0;
                    // 解析缓冲区中的完整数据包
                    match rdr::Pack::decode(&mut &buffer[current_consumed..]) {
                        Ok(pack) => {
                            packets_processed += 1;
                            info!("处理数据包 {}", pack.data_type);
                            info!("数据包大小 {} 字节", pack.data.len());
                            current_consumed += pack.total_size as usize;
                            process_pack(pack, self.forward_sender.clone());
                        },
                        Err(e) => {
                            error!("数据包解析错误: {}", e);
                        }
                    }
                    buffer.drain(0..current_consumed);
                },
                None => {
                    let _ = sleep(Duration::from_millis(10));
                }
            }
        }
        info!("数据处理任务结束，共处理 {} 个数据包", packets_processed);
    }

    async fn response(&mut self) {
        use tokio::time::{timeout as with_timeout, sleep};
        info!("等待握手信号");
        
        // 在5秒内持续监听握手信号
        let timeout_result = with_timeout(Duration::from_secs(5), async {
            loop {
                if let Some(data) = self.receiver.recv().await {
                    info!("接收到握手数据: {}", String::from_utf8_lossy(&data));
                    if data == "Ready?".as_bytes() {
                        // 发送响应
                        if let Err(e) = self.sender.send("Ready".as_bytes().to_vec()).await {
                            error!("发送握手响应失败: {}", e);
                        } else {
                            info!("握手响应已发送");
                        }
                        break; // 成功完成握手，退出循环
                    } else {
                        info!("收到非握手数据，继续等待握手信号");
                    }
                }
            }
        }).await;
        
        match timeout_result {
            Ok(()) => {
                info!("握手成功完成");
            }
            Err(_) => {
                info!("握手超时，客户端可能直接发送数据，继续执行");
            }
        }
    }
}
