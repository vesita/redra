use core::net;

use bevy::prelude::*;
use tokio::sync::{broadcast, mpsc};

use crate::{channel::core::RDPack, module::resource::RDResource, net::listener::RDListener};

pub fn rd_setup (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut resources: ResMut<RDResource>
) {
    // 添加环境光
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
        ..default()
    });

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}