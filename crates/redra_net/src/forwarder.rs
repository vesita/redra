use log::{debug, error, info, warn};
use tokio::sync::mpsc;

use redra_parser::core::RDPack;
use redra_parser::proto_converter::process_command;
use redra_proto::proto::command::Command;

pub struct Forwarder {
    receiver: mpsc::Receiver<RDPack>,
    senders: Vec<mpsc::UnboundedSender<RDPack>>,
}

impl Forwarder {
    pub fn new(
        receiver: mpsc::Receiver<RDPack>,
        senders: Vec<mpsc::UnboundedSender<RDPack>>,
    ) -> Self {
        Self { receiver, senders }
    }

    pub async fn run(&mut self) {
        while let Some(pack) = self.receiver.recv().await {
            for sender in &self.senders {
                if let Err(e) = sender.send(pack.clone()) {
                    debug!("转发数据包时发生错误: {}", e);
                }
            }
        }
    }
}

/// 数据转发器，负责接收解码后的数据并将其转发到引擎
///
/// 该结构体接收来自连接处理器的数据，解析协议包并将其发送到引擎
pub struct RDForwarder {
    /// 转发器的唯一标识ID
    pub id: usize,
    /// 接收待处理数据的接收器
    pub receiver: mpsc::Receiver<Vec<u8>>,
    /// 用于向引擎发送解析后数据包的发送器
    pub sender: mpsc::Sender<RDPack>,
}

impl RDForwarder {
    /// 创建一个新的数据转发器实例
    ///
    /// # 参数
    /// * `id` - 转发器的唯一标识ID
    /// * `receiver` - 接收待处理数据的接收器
    /// * `sender` - 用于向引擎发送解析后数据包的发送器
    ///
    /// # 返回值
    /// * `RDForwarder` - 新创建的数据转发器实例
    pub fn new(
        id: usize,
        receiver: mpsc::Receiver<Vec<u8>>,
        sender: mpsc::Sender<RDPack>,
    ) -> Self {
        Self {
            id,
            receiver,
            sender,
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
        info!("启动转发任务 - 转发ID: {}", self.id);

        while let Some(data) = self.receiver.recv().await {
            // 使用redra_proto解析字节流为Command对象
            match redra_proto::decode(&data) {
                Ok(command) => {
                    // 使用proto_converter将Command转换为RDPack数组
                    match process_command(command) {
                        Ok(packs) => {
                            // 发送所有转换后的RDPack到引擎
                            for pack in packs {
                                if let Err(e) = self.sender.send(pack).await {
                                    error!("转发数据包时发生错误: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            error!("转换命令为RDPack时发生错误: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("解析协议数据时发生错误: {}", e);
                    // 仍然尝试将原始数据作为消息发送，以便调试
                    let pack = RDPack::Message(String::from_utf8_lossy(&data).to_string());
                    if let Err(e) = self.sender.send(pack).await {
                        error!("转发数据包时发生错误: {}", e);
                        break;
                    }
                }
            }
        }

        info!("转发任务结束 - 转发ID: {}", self.id);
        let _ = release.send(self.id).await;
    }

    /// 获取转发器的ID
    ///
    /// # 返回值
    /// * `usize` - 转发器的唯一标识ID
    pub fn get_id(&self) -> usize {
        self.id
    }
}