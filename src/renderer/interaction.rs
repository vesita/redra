use bevy::prelude::*;

pub mod camera;
pub mod picking;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(camera::CameraInteractionPlugin);
            // .add_plugins(picking::PickingInteractionPlugin);
    }
}
