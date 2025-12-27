use bevy::prelude::*;

use crate::{graph::spawn::general_spawn, module::resource::RDResource};



pub fn rd_update (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut resources: ResMut<RDResource>
) {
    general_spawn(commands, meshes, materials, resources);
}