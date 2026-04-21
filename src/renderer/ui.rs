pub mod wheel_menu;
pub mod playback_control; // 帧回放控制面板

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use wheel_menu::WheelMenuGraphPlugin;
use playback_control::PlaybackUiPlugin;

pub struct UiModule;

impl Plugin for UiModule {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(WheelMenuGraphPlugin)
            .add_plugins(PlaybackUiPlugin); // 启用帧回放控制 UI
    }
}
