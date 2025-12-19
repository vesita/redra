use bevy::{
    mesh::{Mesh, Mesh3d},
    transform::components::Transform,
};
use std::sync::Arc;

#[derive(Clone)]
pub enum RDPack {
    Message(String),
    Spawn(SpawnPack),
}

#[derive(Clone)]
pub struct SpawnPack {
    pub mesh: Arc<Mesh>,
    pub transform: Transform,
    pub material: String,
}