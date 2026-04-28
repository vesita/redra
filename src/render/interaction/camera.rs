use bevy::prelude::*;
use smooth_bevy_cameras::controllers::fps::FpsCameraPlugin;

pub struct CameraInteractionPlugin;

impl Plugin for CameraInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FpsCameraPlugin::default());
    }
}
