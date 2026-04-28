use bevy::prelude::*;
use redra::RedraPlugin;
use smooth_bevy_cameras::LookTransformPlugin;

use redra::render::framerate::FrameRateState;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(RedraPlugin)
        .add_plugins(LookTransformPlugin)
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.9)))
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 })
        .run();
}
