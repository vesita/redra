#[cfg(feature = "graph")]
use bevy::prelude::*;
#[cfg(feature = "graph")]
use redra_net::RDChannel;

pub mod manager;
pub mod keyframe;
pub mod inpto;
pub mod unit_pack;
#[cfg(feature = "graph")]
pub mod playback;
#[cfg(feature = "graph")]
pub mod storage;

pub use manager::FrameManager;
pub use keyframe::KeyFrame;
pub use inpto::{Inpto, InptoTransform};
pub use unit_pack::UnitPack;
#[cfg(feature = "graph")]
pub use playback::{PlaybackState, FramePlaybackPlugin};
#[cfg(feature = "graph")]
pub use storage::{FrameStorage, FrameStoragePlugin};

// ==================== FrameManager 插件 ====================

#[cfg(feature = "graph")]
pub struct FrameManagerPlugin;

#[cfg(feature = "graph")]
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

#[cfg(feature = "graph")]
fn setup_frame_manager(mut commands: Commands) {
    commands.insert_resource(FrameManager::default());
}

#[cfg(feature = "graph")]
fn update_frame_manager(
    mut frame_manager: ResMut<FrameManager>,
    mut tag_registry: ResMut<crate::data::tag::TagRegistry>,
    mut channel: ResMut<RDChannel>,
) {
    let mut processed_count = 0;
    while let Ok(unit) = channel.redra_recver.try_recv() {
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
