pub mod panel;
pub mod font;
pub mod share;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use panel::PanelPlugin;
use crate::graph::ui::font::replace_fonts;

pub struct UiModule;

impl Plugin for UiModule {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(PanelPlugin)
            // .add_systems(Update, toggle_camera_mode_system)
            .add_systems(EguiPrimaryContextPass, replace_fonts);
    }
}