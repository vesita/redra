use bevy::ecs::message::Message;
use nalgebra::Vector3;


#[derive(Debug, Default, Message)]
pub struct RDPoint {
    pub position: Vector3<f32>,
}
