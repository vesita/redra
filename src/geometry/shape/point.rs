use bevy::prelude::*;
use nalgebra::Vector3;


#[derive(Debug, Default, Message)]
pub struct RDPoint {
    pub position: Vector3<f32>,
}


impl RDPoint {
    pub fn to_mesh(&self) -> Mesh {
        Mesh::from(Sphere::new(0.07))
    }

    pub fn pose(&self) -> Transform {
        let translation = self.to_vec3();
        Transform::from_translation(translation)
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(
            self.position[0],
            self.position[1],
            self.position[2],
        )
    }
}