use bevy::asset::ron::error;
use log::{debug, error, info, trace};
use prost::Message;
use std::{mem::MaybeUninit, time::Duration};
use tokio::{
    self,
    sync::{broadcast, mpsc, oneshot}, time::{sleep, timeout},
};

use std::collections::VecDeque;

use crate::{
    parser::{core::RDPack, proto::process_pack},
    proto::rdr::{self, Pack}, utils::proto_decode::auto_decode,
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
        let mut buffer: VecDeque<Vec<u8>> = VecDeque::new();
        loop {
            tokio::select! {
                // 等待接收数据
                recv_data = self.receiver.recv() => {
                    match recv_data {
                        Some(data) => {
                            // 将新接收的数据追加到缓冲区
                            buffer.push_back(data);
                            info!("从链接器接收到数据，缓冲区大小: {}", buffer.len());
                        },
                        None => {
                            // 接收器已关闭，退出循环
                            info!("接收器已关闭，退出转发器");
                            break;
                        }
                    }
                    if let Some(buf) = buffer.pop_front() {
                        if let Ok(packs) = auto_decode(&buf) {
                            for pack in packs {
                                // 处理数据包
                                info!("处理接收到的协议数据包，类型: {:?}", pack.data_type);
                                process_pack(pack, self.forward_sender.clone());
                                packets_processed += 1;
                            }
                        } else {
                            error!("协议数据解码失败，数据长度: {} 字节", buf.len());
                        }
                    }
                }
                // 添加一个间隔以避免繁忙等待
                _ = sleep(Duration::from_millis(10)) => {
                    // 这里可以添加其他定期任务
                }
            }
        }
        info!("数据转发任务结束，共处理 {} 个数据包", packets_processed);
    }


    // async fn response(&mut self) {
    //     use tokio::time::{timeout as with_timeout, sleep};
    //     info!("等待握手信号");
        
    //     // 在5秒内持续监听握手信号
    //     let timeout_result = with_timeout(Duration::from_secs(5), async {
    //         loop {
    //             if let Some(data) = self.receiver.recv().await {
    //                 info!("接收到握手数据: {}", String::from_utf8_lossy(&data));
    //                 if data == "Ready?".as_bytes() {
    //                     // 发送响应
    //                     if let Err(e) = self.sender.send("Ready".as_bytes().to_vec()).await {
    //                         error!("发送握手响应失败: {}", e);
    //                     } else {
    //                         info!("握手响应已发送");
    //                     }
    //                     break; // 成功完成握手，退出循环
    //                 } else {
    //                     info!("收到非握手数据，继续等待握手信号");
    //                 }
    //             }
    //         }
    //     }).await;
        
    //     match timeout_result {
    //         Ok(()) => {
    //             info!("握手成功完成");
    //         }
    //         Err(_) => {
    //             info!("握手超时，客户端可能直接发送数据，继续执行");
    //         }
    //     }
    // }
}