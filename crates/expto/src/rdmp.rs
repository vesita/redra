pub mod decoding;
pub mod encoding;
pub mod auto;
pub mod proto;

pub use proto::*;

// 初始化日志系统（如果需要的话）
#[cfg(test)]
pub fn init_log() {
    let _ = env_logger::builder().is_test(true).try_init();
}