use bevy::{
    mesh::Mesh,
    transform::components::Transform,
};
use std::sync::Arc;

#[derive(Clone)]
pub enum RDPack {
    Message(String),
    SpawnShape(Box<RDShapePack>),
    SpawnFormat(Box<FormatPack>),
}

#[derive(Clone)]
pub struct RDShapePack {
    pub mesh: Arc<Mesh>,
    pub transform: Transform,
    pub material: String,
}

#[derive(Clone)]
pub struct FormatPack {

}