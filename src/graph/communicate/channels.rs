use bevy::ecs::resource::Resource;
use tokio::sync::{broadcast, mpsc};

use crate::module::parser::core::RDPack;


#[derive(Resource)]
pub struct RDChannel {
    pub sender: broadcast::Sender<RDPack>,
    pub receiver: mpsc::Receiver<RDPack>,
}