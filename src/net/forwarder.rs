use log::{debug, info, warn};
use tokio::{
    self,
    sync::mpsc,
};


use crate::module::parser::{core::RDPack, proto::process_pack, proto_decode::decode_pack};

/// 数据转发器，负责接收解码后的数据并将其转发到引擎
/// 
/// 该结构体接收来自连接处理器的数据，解析协议包并将其发送到引擎
pub struct RDForwarder {
    /// 转发器的唯一标识ID
    pub id: usize,
    /// 接收待处理数据的接收器
    pub receiver: mpsc::Receiver<Vec<u8>>,
    /// 用于向引擎发送解析后数据包的发送器
    pub forward_sender: mpsc::Sender<RDPack>,
}

impl RDForwarder {
    /// 创建一个新的数据转发器实例
    /// 
    /// # 参数
    /// * `id` - 转发器的唯一标识ID
    /// * `receiver` - 接收待处理数据的接收器
    /// * `forward_sender` - 用于向引擎发送解析后数据包的发送器
    /// 
    /// # 返回值
    /// * `RDForwarder` - 新创建的数据转发器实例
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

    /// 启动数据转发器，开始处理接收到的数据
    /// 
    /// 该方法进入一个循环，持续接收数据，解析协议包，
    /// 然后将其发送到引擎进行进一步处理
    /// 
    /// # 参数
    /// * `release` - 用于发送转发器释放通知的发送器
    pub async fn run(&mut self, release: mpsc::Sender<usize>) {
        let mut packets_processed = 0;
        
        info!("启动数据转发任务 - ID: {}", self.id);
        
        loop {
            tokio::select! {
                // 等待接收数据
                recv_data = self.receiver.recv() => {
                    match recv_data {
                        Some(data) => {
                            debug!("从链接器接收到数据，数据大小: {}", data.len());
                            
                            // 直接解析protobuf Pack消息
                            match decode_pack(&data) {
                                Ok(pack) => {
                                    // 处理数据包
                                    process_pack(pack, self.forward_sender.clone());
                                    debug!("数据包处理完成，当前累积数据包数量: {}", packets_processed);
                                    packets_processed += 1;
                                },
                                Err(e) => {
                                    // 如果解析失败，记录错误但不中断处理
                                    warn!("数据包解析失败: {:?}，数据长度: {}", e, data.len());
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
            }
        }
        
        release.send(self.id).await.expect("释放资源失败");
        info!("数据转发任务结束，ID: {}，共处理 {} 个数据包", self.id, packets_processed);
    }

    /// 获取转发器的ID
    /// 
    /// # 返回值
    /// * `usize` - 转发器的唯一标识ID
    pub fn get_id(&self) -> usize {
        self.id
    }
}