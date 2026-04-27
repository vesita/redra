//! 数据层 — 纯帧数据管理与协议转换
//!
//! 职责：
//! - frame: 帧数据管理（FrameManager, KeyFrame, Inpto 等）
//! - protocol: 协议数据的提取与转换（Unit → Inpto）
//! - storage: 帧数据持久化

pub mod frame;
pub mod protocol;
