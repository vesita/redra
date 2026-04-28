use bevy::prelude::*;

pub use system::label_ui_observe;
pub use design::HoverLabel;
use design::show_hover_label;

pub mod system;
pub mod design;

pub struct LabelUiPlugin;

impl Plugin for LabelUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<HoverLabel>()
            .add_systems(Update, (show_hover_label, label_ui_observe));
    }
}
