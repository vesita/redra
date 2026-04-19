use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};


/// 全局ID生成器实例
pub static GLOBAL_ID_GENERATOR: LazyLock<IdGenerator> = LazyLock::new(|| {
    IdGenerator::new(Some("global_session".to_string()))
});

pub fn id_generator() -> &'static IdGenerator {
    &GLOBAL_ID_GENERATOR
}


/// ID生成器，用于自动生成时间戳和唯一ID
pub struct IdGenerator {
    /// 用于生成唯一ID的原子计数器
    counter: AtomicU64,
    /// 会话ID，用于区分不同客户端的请求
    session_id: String,
}

impl IdGenerator {
    /// 创建新的ID生成器
    pub fn new(session_id: Option<String>) -> Self {
        Self {
            counter: AtomicU64::new(0),
            session_id: session_id.unwrap_or_else(|| {
                // 使用当前时间作为默认会话ID
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("时间倒退")
                    .as_millis() as u64;
                format!("session_{}", now)
            }),
        }
    }

    /// 获取下一个唯一ID
    pub fn next_id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    /// 获取当前时间戳（毫秒）
    pub fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("时间倒退")
            .as_millis() as u64
    }

    /// 获取当前会话ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// 生成Unit结构时的序列号（递增）
    pub fn next_sequence_number(&self) -> u32 {
        self.counter.fetch_add(1, Ordering::SeqCst) as u32
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generator() {
        let generator = IdGenerator::default();
        
        let id1 = generator.next_id();
        let id2 = generator.next_id();
        
        assert_eq!(id2, id1 + 1);
        
        let timestamp = generator.current_timestamp();
        assert!(timestamp > 0);
        
        assert!(generator.session_id().starts_with("session_"));
    }
    
    #[test]
    fn test_global_generator() {
        let id1 = GLOBAL_ID_GENERATOR.next_id();
        let id2 = GLOBAL_ID_GENERATOR.next_id();
        
        assert_eq!(id2, id1 + 1);
        
        let timestamp = GLOBAL_ID_GENERATOR.current_timestamp();
        assert!(timestamp > 0);
        
        assert_eq!(GLOBAL_ID_GENERATOR.session_id(), "global_session");
    }
}