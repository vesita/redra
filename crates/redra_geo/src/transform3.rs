use nalgebra::{Vector3, UnitQuaternion};

/// 3D 变换：平移 + 四元数旋转 + 各向同性缩放
#[derive(Debug, Clone, PartialEq)]
pub struct Transform3 {
    pub translation: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: f32,
}

impl Transform3 {
    pub fn identity() -> Self {
        Self {
            translation: Vector3::zeros(),
            rotation: UnitQuaternion::identity(),
            scale: 1.0,
        }
    }
}

impl Default for Transform3 {
    fn default() -> Self {
        Self::identity()
    }
}
