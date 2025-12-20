use nalgebra::Matrix3;

use crate::geometry::base::RDRRotation;

impl RDRRotation {
    pub fn to_matrix(&self) -> Matrix3<f32> {
        // 绕Z轴旋转矩阵
        let rzm = Matrix3::new(
            self.rz.cos(),
            -self.rz.sin(),
            0.0,
            self.rz.sin(),
            self.rz.cos(),
            0.0,
            0.0,
            0.0,
            1.0,
        );
        
        // 绕Y轴旋转矩阵
        let rym = Matrix3::new(
            self.ry.cos(),
            0.0,
            self.ry.sin(),
            0.0,
            1.0,
            0.0,
            -self.ry.sin(),
            0.0,
            self.ry.cos(),
        );
        
        // 绕X轴旋转矩阵
        let rxm = Matrix3::new(
            1.0,
            0.0,
            0.0,
            0.0,
            self.rx.cos(),
            -self.rx.sin(),
            0.0,
            self.rx.sin(),
            self.rx.cos(),
        );

        rzm * rym * rxm
    }
}
