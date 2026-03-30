pub mod panel;
pub mod font;
pub mod share;
pub mod wheel_menu;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use panel::PanelPlugin;
use font::FontPlugin;
use wheel_menu::WheelMenuGraphPlugin;

pub struct UiModule;

impl Plugin for UiModule {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(PanelPlugin)
            .add_plugins(FontPlugin)
            .add_plugins(WheelMenuGraphPlugin);
    }
}