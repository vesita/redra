use bevy::prelude::*;
use nalgebra::Vector3;

use crate::{geometry::pose::RDRPose};


#[derive(Debug, Default, Message)]
pub struct RDCube {
    pub pose: RDRPose,
    // 边长
    pub edges: Vector3<f32>,
}


impl RDCube {
    pub fn to_mesh(&self) -> Mesh {
        let (x_length, y_length, z_length) = (self.edges[0], self.edges[1], self.edges[2]);
        Mesh::from(Cuboid::new(x_length, y_length, z_length))
    }

    pub fn pose(&self) -> Transform {
        // 提取平移部分
        let translation = Vec3::new(
            self.pose.pose[(0, 3)],
            self.pose.pose[(1, 3)],
            self.pose.pose[(2, 3)],
        );

        // 提取旋转矩阵的3x3部分并转换为四元数
        let rotation_mat3 = Mat3::from_cols(
            Vec3::new(self.pose.pose[(0, 0)], self.pose.pose[(1, 0)], self.pose.pose[(2, 0)]),
            Vec3::new(self.pose.pose[(0, 1)], self.pose.pose[(1, 1)], self.pose.pose[(2, 1)]),
            Vec3::new(self.pose.pose[(0, 2)], self.pose.pose[(1, 2)], self.pose.pose[(2, 2)]),
        );
        
        let rotation = Quat::from_mat3(&rotation_mat3);

        Transform {
            translation,
            rotation,
            scale: Vec3::ONE,
        }
    }
}