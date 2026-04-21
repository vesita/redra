use std::sync::Arc;
use tokio::sync::Mutex;

// 定义类型别名 ThLc，表示 Arc<Mutex<T>> 类型，用于在多任务环境中安全共享数据
// ThLc 代表 Thread Local Concurrent，提供了跨线程安全访问数据的能力
pub type ThLc<T> = Arc<Mutex<T>>;