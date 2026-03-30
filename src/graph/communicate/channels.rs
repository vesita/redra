use bevy::ecs::resource::Resource;
use tokio::sync::{broadcast, mpsc};

use crate::module::parser::core::RDPack;


#[derive(Resource)]
pub struct RDChannel {
    pub sender: broadcast::Sender<RDPack>,
    /// 主 receiver - 用于记录数据帧（优先消费）
    pub receiver: mpsc::Receiver<RDPack>,
}