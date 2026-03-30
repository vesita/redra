pub mod wheel_menu;
pub mod data_play_control;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use wheel_menu::WheelMenuGraphPlugin;

pub struct UiModule;

impl Plugin for UiModule {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(WheelMenuGraphPlugin);
    }
}