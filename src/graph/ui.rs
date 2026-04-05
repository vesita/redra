pub mod wheel_menu;
pub mod playback_control;
// pub mod camera; // 移除UI相机模块，bevy_egui会自动处理UI相机

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use wheel_menu::WheelMenuGraphPlugin;
use playback_control::PlaybackUiPlugin;
// use camera::UiCameraPlugin; // 移除UI相机插件导入

pub struct UiModule;

impl Plugin for UiModule {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(WheelMenuGraphPlugin)
            .add_plugins(PlaybackUiPlugin);
    }
}