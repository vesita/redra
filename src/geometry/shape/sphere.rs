use bevy::prelude::*;

use crate::geometry::base::RDRPosVec;


#[derive(Debug, Default, Message)]
pub struct RDSphere {
    pub pose: RDRPosVec,
    pub radius: f32,
}

impl RDSphere {
    pub fn to_mesh(&self) -> Mesh {
        Mesh::from(Sphere::new(self.radius))
    }


    pub fn pose(&self) -> Transform {
        let translation = Vec3::new(
            self.pose.pos[0],
            self.pose.pos[1],
            self.pose.pos[2],
        );
        // Transform {
        //     translation,
        //     rotation: Quat::IDENTITY,
        //     scale: Vec3::ONE,
        // }
        Transform::from_translation(translation)
    }
}