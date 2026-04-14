use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;
use log::info;

use redra_parser::core::RDPack;

pub struct RDWosh {
    channels: Arc<RwLock<HashMap<usize, mpsc::Sender<Vec<u8>>>>>,
}

impl RDWosh {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_channel(&mut self, sender: mpsc::Sender<Vec<u8>>, id: usize) {
        let mut channels = self.channels.write().await;
        channels.insert(id, sender);
        info!("添加通道 - ID: {}", id);
    }

    pub async fn get_channel(&self, id: usize) -> Option<mpsc::Sender<Vec<u8>>> {
        let channels = self.channels.read().await;
        channels.get(&id).cloned()
    }

    pub async fn remove_channel(&mut self, id: usize) {
        let mut channels = self.channels.write().await;
        if channels.remove(&id).is_some() {
            info!("移除通道 - ID: {}", id);
        }
    }
}

/// 用于负载均衡的自动通道包装器
/// 
/// 该结构包装了发送器并追踪发送次数，用于实现负载均衡
#[derive(Debug, Clone)]
pub struct AutoChannel {
    /// 实际的发送器
    pub sender: mpsc::Sender<Vec<u8>>,
    /// 通道的唯一标识ID
    pub id: usize,
    /// 记录发送次数，用于负载均衡
    pub send_count: usize,
}

impl AutoChannel {
    /// 创建一个新的自动通道包装器
    /// 
    /// # 参数
    /// * `sender` - 实际的发送器
    /// * `id` - 通道的唯一标识ID
    /// 
    /// # 返回值
    /// * `AutoChannel` - 新创建的自动通道包装器
    pub fn new(sender: mpsc::Sender<Vec<u8>>, id: usize) -> AutoChannel {
        AutoChannel {
            sender,
            id,
            send_count: 0,
        }
    }

    /// 发送数据到通道
    /// 
    /// # 参数
    /// * `data` - 要发送的数据
    /// 
    /// # 返回值
    /// * `bool` - 发送成功返回true，失败返回false
    pub async fn send(&mut self, data: Vec<u8>) -> bool {
        // 检查通道是否已满（背压状态）
        if self.channel_alarms() {
            return false;
        }
        self.send_count += 1;
        match self.sender.send(data).await {
            Ok(_) => true,
            Err(_) => false, // 发送失败，通道可能已关闭
        }
    }
    
    /// 检查通道是否接近满载
    /// 
    /// # 返回值
    /// * `bool` - 通道接近满载返回true，否则返回false
    pub fn channel_alarms(&self) -> bool {
        // 检查通道是否接近容量上限
        // 当剩余容量少于总容量的1/4时，认为通道接近满载
        let remaining_capacity = self.sender.capacity();
        let max_capacity = self.sender.max_capacity();
        
        if remaining_capacity < max_capacity / 4 {
            return true;
        }
        false
    }

    /// 获取通道的ID
    /// 
    /// # 返回值
    /// * `usize` - 通道的唯一标识ID
    pub fn get_id(&self) -> usize {
        self.id
    }
}