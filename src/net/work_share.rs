use std::{cmp::Ordering, collections::BinaryHeap};
use tokio::sync::{broadcast, mpsc};
use std::sync::{Arc, Mutex};

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
            false;
        }
        self.send_count += 1;
        self.sender.send(data).await;
        true
    }
    
    pub fn channel_alarms(&self) -> bool {
        if self.sender.capacity() / 2 < self.sender.max_capacity() / 3 {
            return true;
        }
        return false;
    }
}

pub struct RDWosh { 
    pub channels: ThLc<BinaryHeap<AutoChannel>>,
}

impl RDWosh {
    pub fn new() -> RDWosh {
        RDWosh {
            channels: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }

    pub fn add_channel(&self, sender: mpsc::Sender<Vec<u8>>, id: usize) {
        self.channels.lock().unwrap().push(AutoChannel {
            sender,
            id,
            send_count: 0,
        });
    }

    pub fn get_channel(&self) -> Option<mpsc::Sender<Vec<u8>>> {
        self.channels.lock().unwrap().pop().map(|link| link.sender)
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