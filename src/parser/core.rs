use bevy::{
    mesh::{Mesh, Mesh3d},
    transform::components::Transform,
};
use std::sync::Arc;

#[derive(Clone)]
pub enum RDPack {
    Message(String),
    SpawnShape(Box<ShapePack>),
    SpawnFormat(Box<FormatPack>),
}

#[derive(Clone)]
pub struct ShapePack {
    pub mesh: Arc<Mesh>,
    pub transform: Transform,
    pub material: String,
}

#[derive(Clone)]
pub struct FormatPack {

}