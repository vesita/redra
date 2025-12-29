use log::{info, trace};
use std::time::Duration;
use tokio::{
    self,
    sync::mpsc, 
    time::sleep,
};


use crate::{
    module::parser::{core::RDPack, proto::process_pack},
    utils::proto_decode::decode_pack,  // 修改导入
};

pub struct RDForwarder {
    pub id: usize,
    pub receiver: mpsc::Receiver<Vec<u8>>,
    pub forward_sender: mpsc::Sender<RDPack>,
}

impl RDForwarder {
    pub fn new(
        id: usize,
        receiver: mpsc::Receiver<Vec<u8>>,
        forward_sender: mpsc::Sender<RDPack>,
    ) -> RDForwarder {
        RDForwarder {
            id,
            receiver,
            forward_sender,
        }
    }

    pub async fn run(&mut self, release: mpsc::Sender<usize>) {
        let mut packets_processed = 0;
        let mut buffer: Vec<u8> = Vec::new(); // 使用单个缓冲区累积数据
        
        info!("启动数据转发任务 - ID: {}", self.id);
        
        loop {
            tokio::select! {
                // 等待接收数据
                recv_data = self.receiver.recv() => {
                    match recv_data {
                        Some(data) => {
                            // 将新接收的数据追加到累积缓冲区
                            buffer.extend(data);
                            trace!("从链接器接收到数据，累积缓冲区大小: {}", buffer.len());
                            
                            // 直接解析protobuf Pack消息
                            match decode_pack(&buffer) {
                                Ok(pack) => {
                                    // 处理数据包
                                    process_pack(pack, self.forward_sender.clone());
                                    packets_processed += 1;
                                    // 清空已处理的数据
                                    buffer.clear();
                                },
                                Err(_) => {
                                    // 如果解析失败，继续累积更多数据
                                    trace!("当前累积数据无法解析，继续等待更多数据");
                                }
                            }
                        },
                        None => {
                            // 接收器已关闭，退出循环
                            info!("接收器已关闭，退出转发器 ID: {}", self.id);
                            break;
                        }
                    }
                }
                // 添加一个间隔以避免繁忙等待
                _ = sleep(Duration::from_millis(10)) => {
                    // 检查是否有剩余数据可以解析
                    if !buffer.is_empty() {
                        match decode_pack(&buffer) {
                            Ok(pack) => {
                                process_pack(pack, self.forward_sender.clone());
                                packets_processed += 1;
                                buffer.clear();
                            },
                            Err(_) => {
                                // 解析失败，继续等待更多数据
                            }
                        }
                    }
                }
            }
        }
        
        release.send(self.id).await.expect("释放资源失败");
        info!("数据转发任务结束，ID: {}，共处理 {} 个数据包", self.id, packets_processed);
    }
}