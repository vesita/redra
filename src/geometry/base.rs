pub mod converse;

use bevy::ecs::message::Message;
use nalgebra::{Matrix3, Vector3};


#[derive(Debug, Default, Message)]
pub struct RDRPosVec {
    pub pos: Vector3<f32>,
}

#[derive(Debug, Default, Message)]
pub struct RDRRotMat {
    pub rot: Matrix3<f32>,
}


#[derive(Debug, Default, Message)]
pub struct RDTranslation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Default, Message)]
pub struct RDRRotation {
    pub rx: f32,
    pub ry: f32,
    pub rz: f32,
}

#[derive(Debug, Default, Message)]
pub struct RDRScale {
    pub sx: f32,
    pub sy: f32,
    pub sz: f32,
}

