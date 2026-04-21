use std::collections::HashMap;

use expto::rdmp::{ExMesh, Unit};
use bevy::prelude::*;
use redra_net::{NetworkPlugin, RDChannel};

pub mod basic;
pub mod play;

pub use play::*;

#[derive(Resource, Default)]
pub struct FrameManager { 
    pub current_frame: usize,
    timestamp: u64,
    keyframes: Vec<KeyFrame>,
    frames: Vec<UnitPack>,
    temp_units: Vec<Unit>,
    temp_keyframe: Option<KeyFrame>,
}

pub struct UnitPack {
    last_keyframe: Option<usize>,
    pack: Vec<Unit>,
}

pub struct Inpto {
    pub mesh: ExMesh,
    pub material: String,
    pub transform: Transform,
}

pub struct KeyFrame {
    timestamp: u64,
    ids: HashMap<u64, usize>,
    packs: Vec<Inpto>,
}

/// FrameManager 插件
pub struct FrameManagerPlugin;

impl Plugin for FrameManagerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlaybackState>()  // 初始化播放状态
            .add_systems(Startup, setup_frame_manager)
            .add_systems(Update, update_frame_manager);
    }
}

pub fn setup_frame_manager(mut commands: Commands) { 
    commands.insert_resource(FrameManager::default());
}

pub fn update_frame_manager(
    mut frame_manager: ResMut<FrameManager>,
    mut channel: ResMut<RDChannel>
) { 
    // 使用 try_recv 进行非阻塞接收，适配 Bevy 同步系统
    if let Ok(unit) = channel.redra_recver.try_recv() {
        frame_manager.submit(&unit);
    }
    frame_manager.generate_keyframe();
}