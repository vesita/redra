use bevy::ecs::message::Message;
use nalgebra::Matrix4;


#[derive(Debug, Default, Message)]
pub struct RDRPose {
    pub pose: Matrix4<f32>,
}
