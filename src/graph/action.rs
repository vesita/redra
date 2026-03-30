use bevy::prelude::*;

pub mod spawn;
pub mod clear;
pub mod record;

// 定义 ActionPlugin 来注册相关系统
pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<record::DataRecorder>()
            .init_resource::<record::PlaybackManager>()
            .add_systems(Update, clear::clear_all_entities)
            .add_systems(Update, record::record_data_frames);
    }
}