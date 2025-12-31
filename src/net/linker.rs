
use log::{debug, error, info, warn};
use tokio::{io::AsyncReadExt, net::TcpStream, sync::mpsc};

use crate::{ThLc, module::parser::proto_decode::read_trailer};
use super::work_share::{RDWosh};

/// 连接处理器，负责处理单个TCP连接的数据读取和转发
/// 
/// 该结构体管理一个TCP连接，持续从连接中读取数据，
/// 解析数据包并将其转发到工作分配器
pub struct RDLinker {
    /// 连接的唯一标识ID
    pub id: usize,
    /// TCP连接套接字
    pub socket: TcpStream,
    /// 工作分配器的共享引用，用于分发数据到转发任务
    pub wosh: ThLc<RDWosh>,
    /// 用于请求扩展转发任务的发送器
    pub expand_request: mpsc::Sender<usize>,
}

impl RDLinker {
    /// 创建一个新的连接处理器实例
    /// 
    /// # 参数
    /// * `id` - 连接的唯一标识ID
    /// * `socket` - TCP连接套接字
    /// * `wosh` - 工作分配器的共享引用
    /// * `expand_request` - 用于请求扩展转发任务的发送器
    /// 
    /// # 返回值
    /// * `RDLinker` - 新创建的连接处理器实例
    pub fn new(
        id: usize,
        socket: TcpStream,
        wosh: ThLc<RDWosh>,
        expand_request: mpsc::Sender<usize>,
    ) -> RDLinker {
        RDLinker {
            id,
            socket,
            wosh,
            expand_request,
        }
    }
    
    /// 启动连接处理器，开始处理TCP连接数据
    /// 
    /// 该方法进入一个循环，持续从TCP连接中读取数据，
    /// 解析完整数据包并将其发送到转发任务
    /// 
    /// # 参数
    /// * `release` - 用于发送连接释放通知的发送器
    pub async fn run(&mut self, release: mpsc::Sender<usize>) {
        info!("启动TCP链接处理器 ID: {}", self.id);
        
        let mut total_bytes_received = 0;
        let mut packets_received = 0;

        // 使用更大的缓冲区来减少系统调用
        let mut buffer = [0; 1024];
        
        // 累积缓冲区，用于处理跨包数据
        let mut accum_buffer = Vec::new();

        // 用于发送数据的通道发送器，延迟获取以减少不必要的锁操作
        let mut sender = None;

        loop {
            let result = self.socket.read(&mut buffer).await;
            match result {
                Ok(0) => {
                    info!("接收到0字节，连接可能已关闭，退出链接处理器 ID: {}", self.id);
                    break;
                },
                Ok(len) => {
                    total_bytes_received += len;
                    packets_received += 1;
                    
                    debug!("从TCP连接 ID: {} 接收到 {} 字节数据，累计接收: {} 字节，数据包序号: {}", 
                            self.id, len, total_bytes_received, packets_received);
                    
                    // 将新接收的数据追加到累积缓冲区
                    accum_buffer.extend_from_slice(&buffer[..len]);
                    
                    // 处理累积缓冲区中的完整数据包
                    while let Some(packet_data) = self.extract_complete_packet(&mut accum_buffer) {
                        // 将数据发送操作移到独立任务中，避免阻塞接收循环
                        self.send_data(packet_data, &mut sender).await;
                    }
                },
                Err(e) => {
                    error!("从TCP连接ID: {} 读取数据失败: {}", self.id, e);
                    break;
                }
            }
        }
        
        // 处理连接断开前剩余的未完整数据
        if !accum_buffer.is_empty() {
            warn!("连接断开时仍有未处理的数据，长度: {}", accum_buffer.len());
            // 尝试发送剩余数据，即使它可能不是一个完整的包
            if !accum_buffer.is_empty() {
                self.send_data(accum_buffer, &mut sender).await;
            }
        }
        
        // 将当前sender放回wosh
        if let Some(s) = sender {
            self.wosh.lock().await.add_channel(s, self.id).await;
        }
        
        release.send(self.id).await.expect("释放资源失败");
        info!("TCP链接处理器任务结束，ID: {}，总计处理 {} 字节，{} 个数据包", 
              self.id, total_bytes_received, packets_received);
    }

    /// 从累积缓冲区中提取完整数据包
    /// 
    /// 使用trailer解析策略检查缓冲区中是否存在完整数据包
    /// 
    /// # 参数
    /// * `accum_buffer` - 包含累积数据的可变引用缓冲区
    /// 
    /// # 返回值
    /// * `Option<Vec<u8>>` - 如果找到完整数据包则返回其数据，否则返回None
    fn extract_complete_packet(&mut self, accum_buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
        // 需要至少4字节才能开始解析trailer
        if accum_buffer.len() < 4 {
            return None;
        }

        // 使用read_trailer函数解析预告信息
        if let Some((left, right)) = read_trailer(accum_buffer, 0) {
            if accum_buffer.len() >= right {
                // 提取完整数据包
                let packet_data = accum_buffer[left..right].to_vec();
                
                // 从累积缓冲区中移除已处理的数据
                accum_buffer.drain(..right);
                
                return Some(packet_data);
            } else {
                // 数据不足，等待更多数据
                return None;
            }
        }
        //  else {
        //     // trailer解析失败，可能是数据损坏，移除第一个字节继续尝试
        //     accum_buffer.remove(0);
        //     self.extract_complete_packet(accum_buffer).await
        // }
        None
    }

    /// 发送数据到通道的辅助函数
    /// 
    /// 尝试使用现有的发送器发送数据，如果失败则从工作分配器获取新的发送器
    /// 
    /// # 参数
    /// * `data` - 要发送的数据
    /// * `sender` - 指向发送器的可变引用
    /// 
    /// # 返回值
    /// * `bool` - 发送成功返回true，失败返回false
    async fn send_data(&mut self, data: Vec<u8>, sender: &mut Option<mpsc::Sender<Vec<u8>>>) -> bool {
            // 首先尝试使用现有sender发送数据
            if let Some(s) = sender {
                if s.send(data.clone()).await.is_ok() {
                return true;
                }
            }
            
            // 如果发送失败或sender为None，尝试获取新通道
            let channel_option = self.wosh.lock().await.get_channel().await;
            *sender = channel_option;
        
        // 尝试使用新获取的通道发送
        if let Some(s) = sender {
            if s.send(data).await.is_ok() {
                return true;
            }
        } else {
            // 没有可用通道，先尝试获取一个通道
            let new_sender_option = self.wosh.lock().await.get_channel().await;
            
            if let Some(new_sender) = new_sender_option {
                // 成功获取通道，保存并尝试发送
                *sender = Some(new_sender);
                if sender.as_ref().unwrap().send(data).await.is_ok() {
                    return true;
                }
            } else {
                // 仍然没有可用通道，请求扩容
                self.expand_request.send(self.id).await.expect("请求扩容失败");
                
                // 再次尝试获取通道并发送（扩容可能已经创建了新通道）
                let final_sender_option = self.wosh.lock().await.get_channel().await;
                
                if let Some(new_sender) = final_sender_option {
                    if new_sender.send(data).await.is_ok() {
                        *sender = Some(new_sender);
                        return true;
                    }
                }
            }
        }
        false
    }
}