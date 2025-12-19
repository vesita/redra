use tokio::sync::{broadcast, mpsc};

use crate::channel::core::RDPack;


pub struct RDChannel {
    pub sender: broadcast::Sender<RDPack>,
    pub receiver: mpsc::Receiver<RDPack>,
}