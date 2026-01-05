use bevy::prelude::*;

use crate::graph::{MaterialManager, action::spawn::general_spawn, communicate::channels};



pub fn rd_update (
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    material_manager: ResMut<MaterialManager>,
    channel: ResMut<channels::RDChannel>,
) {
    general_spawn(commands, meshes, materials, material_manager, channel);
}