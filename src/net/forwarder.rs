use log::{debug, info, warn};
use tokio::sync::mpsc;

use crate::module::parser::{core::RDPack, proto::process_pack};
use crate::module::parser::proto_decode::decode_pack;  // 保持别名以兼容现有代码

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
                            debug!("从链接器接收到数据，数据大小: {} 字节", data.len());
                            
                            // 直接解析protobuf Command消息
                            match decode_pack(&data) {
                                Ok(command) => {
                                    debug!("✅ 成功解码Command消息");
                                    
                                    // 打印Command类型信息用于调试
                                    if let Some(ref cmd_pack) = command.cmd_pack {
                                        use redra_proto::proto::command::command::CmdPack;
                                        match cmd_pack {
                                            CmdPack::Conception(_) => debug!("Command类型: Conception"),
                                            CmdPack::Designation(design) => {
                                                debug!("Command类型: Designation");
                                                if let Some(ref cmd) = design.cmd {
                                                    use redra_proto::proto::designation::design_cmd::Cmd;
                                                    match cmd {
                                                        Cmd::Spawn(spawn) => {
                                                            debug!("  - 操作: Spawn");
                                                            if let Some(ref data) = spawn.data {
                                                                use redra_proto::proto::designation::spawn::Data;
                                                                match data {
                                                                    Data::ShapeData(shape) => {
                                                                        debug!("  - 数据类型: ShapeData");
                                                                        if let Some(ref shape_data) = shape.data {
                                                                            use redra_proto::proto::shape::shape_pack::Data as ShapeData;
                                                                            match shape_data {
                                                                                ShapeData::Point(_) => debug!("  - 形状: Point"),
                                                                                ShapeData::Segment(_) => debug!("  - 形状: Segment"),
                                                                                ShapeData::Sphere(_) => debug!("  - 形状: Sphere"),
                                                                                ShapeData::Cube(_) => debug!("  - 形状: Cube"),
                                                                                _ => debug!("  - 形状: {:?}", shape_data),
                                                                            }
                                                                        }
                                                                    },
                                                                    Data::FormatData(_) => debug!("  - 数据类型: FormatData"),
                                                                }
                                                            }
                                                        },
                                                        Cmd::Update(_) => debug!("  - 操作: Update"),
                                                        Cmd::Delete(_) => debug!("  - 操作: Delete"),
                                                    }
                                                }
                                            },
                                            CmdPack::Transform(_) => debug!("Command类型: Transform"),
                                            CmdPack::PointCloud(pc) => {
                                                debug!("Command类型: PointCloud (帧ID: {}, 点数: {})", 
                                                       pc.frame_id, pc.points.len());
                                            },
                                        }
                                    } else {
                                        warn!("⚠️ Command消息中没有cmd_pack字段");
                                    }
                                    
                                    // 处理数据包
                                    process_pack(command, self.forward_sender.clone());
                                    debug!("数据包处理完成，当前累积数据包数量: {}", packets_processed);
                                    packets_processed += 1;
                                },
                                Err(e) => {
                                    // 如果解析失败，记录错误但不中断处理
                                    warn!("❌ 数据包解析失败: {:?}，数据长度: {} 字节", e, data.len());
                                    // 打印前32字节的十六进制用于调试
                                    let preview_len = std::cmp::min(32, data.len());
                                    warn!("数据预览 (前{}字节): {:?}", preview_len, &data[..preview_len]);
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
