use bevy::prelude::*;
use bevy_egui::EguiPostUpdateSet;

pub mod camera;
pub mod picking;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(InteractionMessage::default())
            .add_plugins(camera::CameraInteractionPlugin)
            .add_systems(
                PostUpdate,
                picking::detect_empty_click.after(EguiPostUpdateSet::ProcessOutput),
            );
    }
}

#[derive(Resource, Default)]
pub struct InteractionMessage {
    pub selected: Option<u64>,
    /// Observer 刚刚设置了选中，detect_empty_click 跳过本帧清除
    pub just_selected: bool,
}
