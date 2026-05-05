use bevy::prelude::*;

/// 坐标系手性
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handedness {
    /// Bevy 默认：左手系，Y-up
    LeftHanded,
    /// 标准数学：右手系，Y-up
    RightHanded,
}

impl Default for Handedness {
    fn default() -> Self {
        Self::LeftHanded
    }
}

/// 静态场景实体标记组件，用于在坐标系变更时重新渲染
#[derive(Component)]
pub struct StaticSceneEntity;

/// 将 Bevy 左手系 Y-up Transform 转换为右手系 Y-up
///
/// Z 反射矩阵 R·q·R⁻¹ 对旋转的作用：
/// - 绕 X 轴的旋转取反（qy 取反，qw 取反）
/// - 绕 Y 轴的旋转取反（qx 取反，qw 取反）
/// - 绕 Z 轴的旋转不变
///
/// 位移: (x, y, z) → (x, y, -z)
/// 旋转: Quat(x, y, z, w) → Quat(x, -y, z, -w)
/// 缩放: 不变（对称网格无需处理）
pub fn lh_to_rh_transform(t: Transform) -> Transform {
    Transform {
        translation: Vec3::new(t.translation.x, t.translation.y, -t.translation.z),
        rotation: Quat::from_xyzw(
            t.rotation.x,
            -t.rotation.y,
            t.rotation.z,
            -t.rotation.w,
        ),
        scale: t.scale,
    }
}

/// 根据当前坐标系手性条件性地转换 Transform
///
/// LeftHanded（Bevy 原生）: 原样返回
/// RightHanded: 通过 lh_to_rh_transform 转换
pub fn apply_handedness(t: Transform, handedness: Handedness) -> Transform {
    match handedness {
        Handedness::LeftHanded => t,
        Handedness::RightHanded => lh_to_rh_transform(t),
    }
}

pub struct CoordSystemPlugin;

impl Plugin for CoordSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Handedness>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_passthrough_lh() {
        let t = Transform::from_xyz(1.0, 2.0, 3.0);
        let result = apply_handedness(t, Handedness::LeftHanded);
        assert_eq!(result, t);
    }

    #[test]
    fn test_translation_z_negated() {
        let t = Transform::from_xyz(1.0, 2.0, 3.0);
        let result = apply_handedness(t, Handedness::RightHanded);
        assert!((result.translation.x - 1.0).abs() < 1e-6);
        assert!((result.translation.y - 2.0).abs() < 1e-6);
        assert!((result.translation.z - (-3.0)).abs() < 1e-6);
    }

    #[test]
    fn test_rotation_y_negated() {
        let q = Quat::from_euler(EulerRot::XYZ, 0.5, 1.0, 0.3);
        let t = Transform::default().with_rotation(q);
        let result = apply_handedness(t, Handedness::RightHanded);
        // Z 反射: x 不变, y 取反, z 不变, w 取反
        assert!((result.rotation.x - q.x).abs() < 1e-6);
        assert!((result.rotation.y - (-q.y)).abs() < 1e-6);
        assert!((result.rotation.z - q.z).abs() < 1e-6);
        assert!((result.rotation.w - (-q.w)).abs() < 1e-6);
    }

    #[test]
    fn test_roundtrip() {
        let t = Transform::from_xyz(1.0, 2.0, 3.0)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.5, 1.0, 0.3));
        let rh = apply_handedness(t, Handedness::RightHanded);
        let back = lh_to_rh_transform(rh);
        assert!((back.translation - t.translation).length() < 1e-6);
    }
}
