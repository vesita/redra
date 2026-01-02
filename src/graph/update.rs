use bevy::prelude::*;

use crate::graph::{MaterialManager, action::spawn::general_spawn, communicate::channels};



pub fn rd_update (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material_manager: ResMut<MaterialManager>,
    mut channel: ResMut<channels::RDChannel>,
) {
    general_spawn(commands, meshes, materials, material_manager, channel);
}