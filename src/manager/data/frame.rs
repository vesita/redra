use std::collections::HashMap;

use expto::rdmp::{ExMesh, Unit, Tag};
use bevy::prelude::*;
use redra_net::RDChannel;

pub mod basic;
pub mod play;
pub mod storage;

pub use play::*;
// 注意：storage::FrameStorage 现在是 Bevy Resource，用于文件保存
// 如果需要数据库功能，使用 storage::DatabaseFrameStorage
pub use storage::{FrameStorage, FrameStoragePlugin};

#[derive(Resource, Default)]
pub struct FrameManager { 
    pub current_frame: usize,
    timestamp: u64,
    keyframes: Vec<KeyFrame>,
    frames: Vec<UnitPack>,
    temp_units: Vec<Unit>,
    temp_keyframe: Option<KeyFrame>,
    first_temp_unit_timestamp: Option<u64>,
}

pub struct UnitPack {
    last_keyframe: Option<usize>,
    pack: Vec<Unit>,
}

pub struct Inpto {
    pub mesh: ExMesh,
    pub material: String,
    pub transform: Transform,
    pub tag: Option<Tag>,
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
    // 使用循环处理所有可用的 Unit，避免数据堆积
    let mut processed_count = 0;
    while let Ok(unit) = channel.redra_recver.try_recv() {
        frame_manager.submit(&unit);
        processed_count += 1;
    }
    
    if processed_count > 0 {
        log::debug!("帧管理器处理了 {} 个 Unit", processed_count);
    }
    
    frame_manager.generate_keyframe();
}