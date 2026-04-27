use bevy::prelude::*;
use redra_net::RDChannel;

pub mod manager;
pub mod keyframe;
pub mod inpto;
pub mod unit_pack;
pub mod playback;
pub mod storage;

pub use manager::FrameManager;
pub use keyframe::{KeyFrame, SerializableKeyFrame};
pub use inpto::Inpto;
pub use unit_pack::UnitPack;
pub use playback::{PlaybackState, FramePlaybackPlugin};
pub use storage::{FrameStorage, FrameStoragePlugin};

// ==================== FrameManager 插件 ====================

pub struct FrameManagerPlugin;

impl Plugin for FrameManagerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlaybackState>()
            .add_systems(Startup, setup_frame_manager)
            .add_systems(Update, update_frame_manager);
    }
}

fn setup_frame_manager(mut commands: Commands) {
    commands.insert_resource(FrameManager::default());
}

fn update_frame_manager(
    mut frame_manager: ResMut<FrameManager>,
    mut channel: ResMut<RDChannel>,
) {
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
