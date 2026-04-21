use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::rdmp::{ExStamp};

static TIMESTAMP: AtomicU64 = AtomicU64::new(0);
static SEQUENCE_NUMBER: AtomicU32 = AtomicU32::new(0);

pub struct Stamper;

pub fn generate_stamp() -> ExStamp { 
    Stamper::generate_stamp()
}

impl Stamper {
    /// 获取当前时间戳（毫秒）
    fn get_current_timestamp() -> u64 {
        std::time::Instant::now().elapsed().as_millis() as u64
    }

    /// 生成一个新的stamp，自动填充时间戳和序列号
    pub fn generate_stamp() -> ExStamp {
        let ts = Self::get_current_timestamp();
        TIMESTAMP.store(ts, Ordering::Relaxed);
        
        let seq = SEQUENCE_NUMBER.fetch_add(1, Ordering::SeqCst);
        
        ExStamp {
            timestamp: ts,
            session_id: String::default(), // 使用默认值
            sequence_number: seq,
        }
    }

    /// 获取当前序列号
    pub fn get_sequence_number(&self) -> u32 {
        SEQUENCE_NUMBER.load(Ordering::Relaxed)
    }

    /// 获取最后记录的时间戳
    pub fn get_timestamp(&self) -> u64 {
        TIMESTAMP.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_stamp() {
        let stamp1 = Stamper::generate_stamp();
        let stamp2 = Stamper::generate_stamp();

        assert_eq!(stamp1.sequence_number + 1, stamp2.sequence_number);
        assert_eq!(stamp1.session_id, String::default()); // 验证使用了默认值
    }
}