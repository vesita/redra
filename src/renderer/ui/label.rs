
use bevy::prelude::*;

use design::{HoverLabel, show_hover_label};

pub use system::label_ui_observe;

pub mod system;
pub mod design;

#[derive(Component)]
pub struct LabelPanel;

/// 标签UI插件
pub struct LabelUiPlugin;

impl Plugin for LabelUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<HoverLabel>()
            .add_systems(Update, (show_hover_label, label_ui_observe));
    }
}
