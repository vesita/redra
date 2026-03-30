//! Redra Storage - 基于 SQLite 的帧存储 crate
//! 
//! 本 crate 提供基于 SQLite 的帧元数据存储和检索功能，
//! 二进制数据存储在文件系统中。
//! 
//! # 特性
//! - 帧元数据存储 (SQLite)
//! - 点云二进制数据存储 (文件系统)
//! - 高效的时间范围查询和条件过滤
//! - 批量写入优化 (事务打包)

pub mod storage;

pub use storage::{
    FrameDatabase,
    FrameMetadata,
    FrameStorage,
    FrameStats,
    FrameType,
};
