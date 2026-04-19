/// Record 模块 - 录制与回放系统
/// 
/// 基于 parser 和 redra_storage 实现的 Bevy 集成层

pub mod recorder;
pub mod player;

// 导出公共 API
pub use recorder::{RecorderPlugin, RecordingState};
pub use player::{PlayerPlugin, PlaybackState};
