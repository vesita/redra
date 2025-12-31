use bevy::prelude::*;

use crate::geometry::base::RDRPosVec;

/// RDSphere 结构体表示一个球体的几何信息
/// 包含球体的位置和半径信息
#[derive(Debug, Default, Message)]
pub struct RDSphere {
    pub pose: RDRPosVec,  // 球体的位置信息
    pub radius: f32,      // 球体的半径
}

impl RDSphere {
    /// 将球体转换为Bevy引擎可用的Mesh对象
    /// 
    /// # 返回值
    /// * `Mesh` - Bevy引擎中的网格对象
    pub fn to_mesh(&self) -> Mesh {
        Mesh::from(Sphere::new(self.radius))
    }

    /// 将球体的位置信息转换为Bevy引擎可用的Transform对象
    /// 
    /// # 返回值
    /// * `Transform` - Bevy引擎中的变换对象，包含位置、旋转和缩放信息
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