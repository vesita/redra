use bevy::prelude::*;
use redra_net::RDChannel;
use redra_parser::{InternalPointCloudPack, RDPack};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use log::{debug, error};
use redra_storage::{FrameMetadata, FrameType, FrameStorage};

pub mod models;
pub mod serialization;
pub mod recording;
pub mod replay;

// 重新导出主要类型，保持原有的API兼容性
pub use models::*;
pub use serialization::*;
pub use recording::*;
pub use replay::*;

// 为了保持与原有代码的兼容性，保留一些常用的类型别名
pub type DataRecorder = models::DataRecorder;
pub type PlaybackManager = models::PlaybackManager;
pub type RecordingMode = models::RecordingMode;
pub type ReplayedEntity = models::ReplayedEntity;
pub type DataFrame = models::DataFrame;
pub type FrameBuilder = models::FrameBuilder;

// 导出相关的函数
pub use recording::record_data_frames;
pub use replay::{replay_data_frames, debug_replayed_entities, load_frame_from_storage};
pub use serialization::{serialize_point_cloud, deserialize_point_cloud};
