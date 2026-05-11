#[cfg(feature = "graph")]
use bevy::prelude::*;
#[cfg(feature = "graph")]
use redra::RedraPlugin;
#[cfg(feature = "graph")]
use smooth_bevy_cameras::LookTransformPlugin;

#[cfg(feature = "graph")]
use redra::render::framerate::FrameRateState;

#[cfg(feature = "graph")]
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(RedraPlugin)
        .add_plugins(LookTransformPlugin)
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.9)))
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 })
        .run();
}

#[cfg(not(feature = "graph"))]
fn main() {
    println!("Redra headless mode (--no-default-features)");
    println!("Run with 'cargo run' for the full UI experience.");
}
