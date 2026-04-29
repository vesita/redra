use nalgebra::{UnitQuaternion, Vector3};
use crate::Transform3;

/// 坐标轴约定
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisConvention {
    /// Y 轴向上（OpenGL / Bevy 默认）
    YUp,
    /// Z 轴向上（GIS / 某些点云格式）
    ZUp,
}

/// 在不同坐标轴约定间转换变换
pub fn convert_axis(
    t: &Transform3,
    from: AxisConvention,
    to: AxisConvention,
) -> Transform3 {
    if from == to {
        return t.clone();
    }

    // YUp → ZUp: 绕 X 轴旋转 +90°（Y→Z, Z→-Y）
    // ZUp → YUp: 绕 X 轴旋转 -90°（Z→Y, Y→-Z）
    let angle = match (from, to) {
        (AxisConvention::YUp, AxisConvention::ZUp) => std::f32::consts::FRAC_PI_2,
        (AxisConvention::ZUp, AxisConvention::YUp) => -std::f32::consts::FRAC_PI_2,
        _ => unreachable!(),
    };
    let q = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), angle);

    Transform3 {
        translation: q * t.translation,
        rotation: q * t.rotation,
        scale: t.scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_yup_to_zup_roundtrip() {
        let t = Transform3 {
            translation: Vector3::new(1.0, 2.0, 3.0),
            rotation: UnitQuaternion::identity(),
            scale: 1.0,
        };

        let t_zup = convert_axis(&t, AxisConvention::YUp, AxisConvention::ZUp);
        let t_back = convert_axis(&t_zup, AxisConvention::ZUp, AxisConvention::YUp);

        assert!((t.translation - t_back.translation).norm() < 1e-6);
    }

    #[test]
    fn test_zup_conversion() {
        let t = Transform3 {
            translation: Vector3::new(0.0, 1.0, 0.0), // Y-up 中的向上方向
            rotation: UnitQuaternion::identity(),
            scale: 1.0,
        };

        let t_zup = convert_axis(&t, AxisConvention::YUp, AxisConvention::ZUp);
        // Y-up 的 (0,1,0) 应变为 Z-up 的 (0,0,1)
        assert!((t_zup.translation - Vector3::new(0.0, 0.0, 1.0)).norm() < 1e-6);
    }
}
