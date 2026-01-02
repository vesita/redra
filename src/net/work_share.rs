use std::{cmp::Ordering, collections::{BinaryHeap, HashMap}};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use std::sync::Arc;


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

/// 工作分配器，用于在多个转发任务之间分配工作负载
/// 
/// 该结构体使用最小堆来跟踪最少使用的通道，并提供负载均衡
pub struct RDWosh { 
    /// 存储可用通道的最小堆，按发送计数排序
    pub channels: Arc<Mutex<BinaryHeap<AutoChannel>>>,
    /// ID到索引的映射，用于快速查找通道
    pub id_map: Arc<Mutex<HashMap<usize, usize>>>,
}

impl RDWosh {
    /// 创建一个新的工作分配器实例
    /// 
    /// # 返回值
    /// * `RDWosh` - 新创建的工作分配器实例
    pub fn new() -> RDWosh {
        RDWosh {
            channels: Arc::new(Mutex::new(BinaryHeap::new())),
            id_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 添加一个新的通道到工作分配器
    /// 
    /// # 参数
    /// * `sender` - 要添加的发送器
    /// * `id` - 通道的唯一标识ID
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

    /// 获取一个可用的通道发送器
    /// 
    /// 该方法返回最少使用的通道（按发送计数排序）
    /// 
    /// # 返回值
    /// * `Option<mpsc::Sender<Vec<u8>>>` - 如果有可用通道则返回发送器，否则返回None
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