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
            .init_resource::<crate::data::tag::TagRegistry>()
            .init_resource::<crate::data::tag::TagFilter>()
            .add_systems(Startup, setup_frame_manager)
            .add_systems(Update, update_frame_manager);
    }
}

fn setup_frame_manager(mut commands: Commands) {
    commands.insert_resource(FrameManager::default());
}

fn update_frame_manager(
    mut frame_manager: ResMut<FrameManager>,
    mut tag_registry: ResMut<crate::data::tag::TagRegistry>,
    mut channel: ResMut<RDChannel>,
) {
    let mut processed_count = 0;
    while let Ok(unit) = channel.redra_recver.try_recv() {
        // 提取 TagCollectionDef 到 TagRegistry（不进入帧数据流）
        let filtered: Vec<_> = unit.objects.iter().filter(|obj| {
            use expto::rdmp::ex_object::UObject;
            if let Some(UObject::TagCollectionDef(def)) = &obj.u_object {
                tag_registry.register(def.clone(), crate::data::tag::CollectionSource::Dynamic);
                false
            } else { true }
        }).cloned().collect();

        if filtered.len() != unit.objects.len() {
            let mut filtered_unit = unit.clone();
            filtered_unit.objects = filtered;
            frame_manager.submit(&filtered_unit);
        } else {
            frame_manager.submit(&unit);
        }
        processed_count += 1;
    }

    if processed_count > 0 {
        log::debug!("帧管理器处理了 {} 个 Unit", processed_count);
    }

    frame_manager.generate_keyframe();
}
