use std::{cmp::Ordering, collections::{BinaryHeap, HashMap}};
use tokio::sync::mpsc;
use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;

use crate::ThLc;

// 用于负载均衡的追踪发送器包装器
#[derive(Debug, Clone)]
pub struct AutoChannel {
    pub sender: mpsc::Sender<Vec<u8>>,
    pub id: usize,
    pub send_count: usize,  // 记录发送次数，用于负载均衡
}

impl AutoChannel {
    pub fn new(sender: mpsc::Sender<Vec<u8>>, id: usize) -> AutoChannel {
        AutoChannel {
            sender,
            id,
            send_count: 0,
        }
    }

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

    pub fn get_id(&self) -> usize {
        self.id
    }
}

pub struct RDWosh { 
    pub channels: Arc<Mutex<BinaryHeap<AutoChannel>>>,
    pub id_map: Arc<Mutex<HashMap<usize, usize>>>,
}

impl RDWosh {
    pub fn new() -> RDWosh {
        RDWosh {
            channels: Arc::new(Mutex::new(BinaryHeap::new())),
            id_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_channel(&self, sender: mpsc::Sender<Vec<u8>>, id: usize) {
        let auto_channel = AutoChannel {
            sender,
            id,
            send_count: 0,
        };
        
        let mut channels_lock = self.channels.lock().await;
        channels_lock.push(auto_channel);
        // 使用当前通道数量作为索引
        let channel_count = channels_lock.len();
        drop(channels_lock);
        
        let mut id_map_lock = self.id_map.lock().await;
        id_map_lock.insert(id, channel_count);
        drop(id_map_lock);
    }

    pub async fn get_channel(&self) -> Option<mpsc::Sender<Vec<u8>>> {
        let mut channels_lock = self.channels.lock().await;
        channels_lock.pop().map(|link| link.sender)
    }
}


// 实现基于 send_count 的排序，用于最小堆（BinaryHeap 需要最大堆，所以我们反转比较）
impl Ord for AutoChannel {
    fn cmp(&self, other: &Self) -> Ordering {
        other.send_count.cmp(&self.send_count)  // 反转以形成最小堆
    }
}

impl PartialOrd for AutoChannel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AutoChannel {
    fn eq(&self, other: &Self) -> bool {
        self.send_count == other.send_count
    }
}

impl Eq for AutoChannel {}