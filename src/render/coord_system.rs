use bevy::prelude::*;

/// 向上轴方向
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpAxis {
    PlusY,
    MinusY,
    PlusZ,
    MinusZ,
    PlusX,
    MinusX,
}

impl Default for UpAxis {
    fn default() -> Self {
        Self::PlusY
    }
}

impl UpAxis {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PlusY => "+Y",
            Self::MinusY => "-Y",
            Self::PlusZ => "+Z",
            Self::MinusZ => "-Z",
            Self::PlusX => "+X",
            Self::MinusX => "-X",
        }
    }

    /// 返回将此轴映射到 +Y 的旋转四元数
    fn to_plus_y_rotation(self) -> Quat {
        let half = std::f32::consts::FRAC_PI_2;
        match self {
            Self::PlusY => Quat::IDENTITY,
            Self::MinusY => Quat::from_rotation_z(std::f32::consts::PI),
            Self::PlusZ => Quat::from_rotation_x(-half),
            Self::MinusZ => Quat::from_rotation_x(half),
            Self::PlusX => Quat::from_rotation_z(half),
            Self::MinusX => Quat::from_rotation_z(-half),
        }
    }
}

/// 坐标系手性
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handedness {
    LeftHanded,
    RightHanded,
}

impl Default for Handedness {
    fn default() -> Self {
        Self::LeftHanded
    }
}

/// 完整坐标系配置（手性 + 向上轴 + 显示选项）
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoordSystem {
    pub handedness: Handedness,
    pub up_axis: UpAxis,
    pub show_axes: bool,
}

impl Default for CoordSystem {
    fn default() -> Self {
        Self {
            handedness: Handedness::LeftHanded,
            up_axis: UpAxis::PlusY,
            show_axes: true,
        }
    }
}

/// 静态场景实体标记组件，用于在坐标系变更时重新渲染
#[derive(Component)]
pub struct StaticSceneEntity;

// ============================================================================
// 转换函数
// ============================================================================

/// 向上轴旋转：将数据坐标系的"上"方向映射到 Bevy 的 +Y
pub fn apply_up_axis_rotation(t: Transform, up: UpAxis) -> Transform {
    let q = up.to_plus_y_rotation();
    Transform {
        translation: q * t.translation,
        rotation: q * t.rotation,
        scale: t.scale,
    }
}

/// 手性反射：Z 反射 (x, y, z) → (x, y, -z)
///
/// 位移: (x, y, z) → (x, y, -z)
/// 旋转: Quat(x, y, z, w) → Quat(-x, -y, z, w)
pub fn apply_handedness(t: Transform, handedness: Handedness) -> Transform {
    match handedness {
        Handedness::LeftHanded => t,
        Handedness::RightHanded => Transform {
            translation: Vec3::new(t.translation.x, t.translation.y, -t.translation.z),
            rotation: Quat::from_xyzw(
                -t.rotation.x,
                -t.rotation.y,
                t.rotation.z,
                t.rotation.w,
            ),
            scale: t.scale,
        },
    }
}

/// 组合转换：先轴向旋转，再手性反射
pub fn apply_coord_system(t: Transform, coord: CoordSystem) -> Transform {
    let rotated = apply_up_axis_rotation(t, coord.up_axis);
    apply_handedness(rotated, coord.handedness)
}

// ============================================================================
// 插件
// ============================================================================

pub struct CoordSystemPlugin;

impl Plugin for CoordSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CoordSystem>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_vec3_eq(a: Vec3, b: Vec3, eps: f32) {
        assert!((a.x - b.x).abs() < eps, "x: {} vs {}", a.x, b.x);
        assert!((a.y - b.y).abs() < eps, "y: {} vs {}", a.y, b.y);
        assert!((a.z - b.z).abs() < eps, "z: {} vs {}", a.z, b.z);
    }

    #[test]
    fn test_identity_plus_y() {
        let t = Transform::from_xyz(1.0, 2.0, 3.0);
        let result = apply_up_axis_rotation(t, UpAxis::PlusY);
        assert_vec3_eq(result.translation, t.translation, 1e-6);
    }

    #[test]
    fn test_zup_to_yup() {
        let t = Transform::from_xyz(0.0, 0.0, 1.0);
        let result = apply_up_axis_rotation(t, UpAxis::PlusZ);
        assert_vec3_eq(result.translation, Vec3::new(0.0, 1.0, 0.0), 1e-5);
    }

    #[test]
    fn test_zdown_to_yup() {
        let t = Transform::from_xyz(0.0, 0.0, -1.0);
        let result = apply_up_axis_rotation(t, UpAxis::MinusZ);
        assert_vec3_eq(result.translation, Vec3::new(0.0, 1.0, 0.0), 1e-5);
    }

    #[test]
    fn test_xup_to_yup() {
        let t = Transform::from_xyz(1.0, 0.0, 0.0);
        let result = apply_up_axis_rotation(t, UpAxis::PlusX);
        assert_vec3_eq(result.translation, Vec3::new(0.0, 1.0, 0.0), 1e-5);
    }

    #[test]
    fn test_xdown_to_yup() {
        let t = Transform::from_xyz(-1.0, 0.0, 0.0);
        let result = apply_up_axis_rotation(t, UpAxis::MinusX);
        assert_vec3_eq(result.translation, Vec3::new(0.0, 1.0, 0.0), 1e-5);
    }

    #[test]
    fn test_ydown_to_yup() {
        let t = Transform::from_xyz(0.0, -1.0, 0.0);
        let result = apply_up_axis_rotation(t, UpAxis::MinusY);
        assert_vec3_eq(result.translation, Vec3::new(0.0, 1.0, 0.0), 1e-5);
    }

    #[test]
    fn test_handedness_roundtrip() {
        let t = Transform::from_xyz(1.0, 2.0, 3.0);
        let rh = apply_handedness(t, Handedness::RightHanded);
        let back = apply_handedness(rh, Handedness::RightHanded);
        assert_vec3_eq(back.translation, t.translation, 1e-6);
    }

    #[test]
    fn test_combined_rh_zup() {
        let t = Transform::from_xyz(0.0, 0.0, 1.0);
        let coord = CoordSystem { handedness: Handedness::RightHanded, up_axis: UpAxis::PlusZ, show_axes: true };
        let result = apply_coord_system(t, coord);
        assert_vec3_eq(result.translation, Vec3::new(0.0, 1.0, 0.0), 1e-5);
    }
}
