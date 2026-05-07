use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

pub use system::label_ui_observe;
pub use design::HoverLabel;
use design::{show_hover_label, TagEditState, TagEditResult};
use system::apply_tag_edit;

pub mod system;
pub mod design;

pub struct LabelUiPlugin;

impl Plugin for LabelUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<HoverLabel>()
            .init_resource::<TagEditState>()
            .init_resource::<TagEditResult>()
            .add_systems(Update, (label_ui_observe, apply_tag_edit))
            .add_systems(EguiPrimaryContextPass, show_hover_label);
    }
}
