use bevy::prelude::*;
use nalgebra::Point3;
use redra_geo::Transform3;

use crate::kabsch::{solve_absolute_orientation, AbsoluteOrientationResult};
use crate::icp::{icp_refine, IcpParams, IcpResult};
use crate::CalibrationError;

/// 标定算法资源 — 在 Bevy 系统中通过 `Res<Calibrator>` 访问
///
/// 封装 Kabsch-Umeyama、ICP 等算法，提供无状态的 computation API。
///
/// # 示例
/// ```no_run
/// # use bevy::prelude::*;
/// # use redra_calib::prelude::Calibrator;
/// fn my_system(calibrator: Res<Calibrator>) {
///     let src = vec![];
///     let dst = vec![];
///     let result = calibrator.solve_absolute_orientation(&src, &dst, false);
/// }
/// ```
#[derive(Resource, Default)]
pub struct Calibrator;

impl Calibrator {
    /// 求解两组 3D 点之间的最优刚体变换（Kabsch-Umeyama 算法）
    pub fn solve_absolute_orientation(
        &self,
        src: &[Point3<f32>],
        dst: &[Point3<f32>],
        estimate_scale: bool,
    ) -> Result<AbsoluteOrientationResult, CalibrationError> {
        solve_absolute_orientation(src, dst, estimate_scale)
    }

    /// ICP 精配准 — 从初始变换开始迭代优化
    pub fn icp_refine(
        &self,
        src: &[Point3<f32>],
        dst: &[Point3<f32>],
        initial: &Transform3,
        params: &IcpParams,
    ) -> Result<IcpResult, CalibrationError> {
        icp_refine(src, dst, initial, params)
    }
}

/// 标定插件 — 注册 `Calibrator` 资源
///
/// 在应用入口添加：
/// ```no_run
/// # use bevy::prelude::*;
/// # use redra_calib::prelude::CalibPlugin;
/// App::new().add_plugins(CalibPlugin);
/// ```
pub struct CalibPlugin;

impl Plugin for CalibPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Calibrator>();
    }
}
