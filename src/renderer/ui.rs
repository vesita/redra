pub mod wheel_menu;
// pub mod playback_control; // 已移除：录制回放功能应由 FrameManager 管理，不在 renderer 层

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use wheel_menu::WheelMenuGraphPlugin;
// use playback_control::PlaybackUiPlugin; // 已移除

pub struct UiModule;

impl Plugin for UiModule {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(WheelMenuGraphPlugin);
            // .add_plugins(PlaybackUiPlugin); // 已移除
    }
}
