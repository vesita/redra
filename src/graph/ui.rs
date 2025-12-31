pub mod clear;
pub mod panel;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use panel::PanelPlugin;

pub struct UiModule;

impl Plugin for UiModule {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(PanelPlugin);
    }
}