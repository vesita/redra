use nalgebra::{UnitQuaternion, Vector3};
use expto::rdmp::ExTransform;
use crate::Transform3;

/// 将协议 ExTransform 转换为几何 Transform3
pub fn extransform_to_transform3(et: &ExTransform) -> Transform3 {
    let rotation = UnitQuaternion::from_euler_angles(et.rx, et.ry, et.rz);

    Transform3 {
        translation: Vector3::new(et.x, et.y, et.z),
        rotation,
        scale: (et.sx + et.sy + et.sz) / 3.0,
    }
}

/// 将几何 Transform3 转换为协议 ExTransform
pub fn transform3_to_extransform(t: &Transform3) -> ExTransform {
    let (rx, ry, rz) = t.rotation.euler_angles();

    ExTransform {
        x: t.translation.x,
        y: t.translation.y,
        z: t.translation.z,
        rx, ry, rz,
        sx: t.scale,
        sy: t.scale,
        sz: t.scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_roundtrip() {
        let t = Transform3 {
            translation: Vector3::new(1.0, 2.0, 3.0),
            rotation: UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.5),
            scale: 2.0,
        };

        let et = transform3_to_extransform(&t);
        let t_back = extransform_to_transform3(&et);

        assert!((t.translation - t_back.translation).norm() < 1e-6);
        assert!(t.rotation.angle_to(&t_back.rotation) < 1e-6);
        assert!((t.scale - t_back.scale).abs() < 1e-6);
    }

    #[test]
    fn test_identity() {
        let et = ExTransform {
            x: 0.0, y: 0.0, z: 0.0,
            rx: 0.0, ry: 0.0, rz: 0.0,
            sx: 1.0, sy: 1.0, sz: 1.0,
        };
        let t = extransform_to_transform3(&et);
        assert!((t.translation - Vector3::zeros()).norm() < 1e-6);
        assert!(t.rotation.angle_to(&UnitQuaternion::identity()) < 1e-6);
        assert!((t.scale - 1.0).abs() < 1e-6);
    }
}
