use bevy::ecs::message::Message;

use crate::geometry::base::RDRPosVec;


#[derive(Debug, Default, Message)]
pub struct RDRBall {
    pub pose: RDRPosVec,
    pub radius: f32,
}