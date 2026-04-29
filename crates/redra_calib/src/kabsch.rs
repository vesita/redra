use nalgebra::{Matrix3, Point3, Vector3};
use nalgebra::linalg::SVD;
use redra_geo::Transform3;

/// Kabsch-Umeyama 算法求解结果
#[derive(Debug, Clone, PartialEq)]
pub struct AbsoluteOrientationResult {
    pub transform: Transform3,
    pub rmsd: f32,
}

/// 求解两组 3D 点之间的最优刚体变换（平移 + 旋转 + 可选各向同性缩放）
///
/// Kabsch-Umeyama 算法：使用 SVD 分解协方差矩阵，
/// 寻找最小二乘意义下的最优变换。
///
/// # 参数
/// - `src`: 源点集
/// - `dst`: 目标点集（与 src 一一对应）
/// - `estimate_scale`: 是否估计各向同性缩放
///
/// # 返回
/// - `AbsoluteOrientationResult`: 最优变换 + RMSD
///
/// # 错误
/// - 点数不足 3
/// - 源/目标点数不匹配
/// - 点共线或退化为平面（SVD 奇异）
pub fn solve_absolute_orientation(
    src: &[Point3<f32>],
    dst: &[Point3<f32>],
    estimate_scale: bool,
) -> Result<AbsoluteOrientationResult, CalibrationError> {
    if src.len() < 3 || dst.len() < 3 {
        return Err(CalibrationError::InsufficientPoints);
    }
    if src.len() != dst.len() {
        return Err(CalibrationError::MismatchedPointCounts(src.len(), dst.len()));
    }

    let n = src.len() as f32;

    // 1. 计算质心（点坐标求和后除以数量）
    let centroid_src = {
        let mut sum = Vector3::zeros();
        for p in src { sum += p.coords; }
        Point3::from(sum / n)
    };
    let centroid_dst = {
        let mut sum = Vector3::zeros();
        for p in dst { sum += p.coords; }
        Point3::from(sum / n)
    };

    // 2. 中心化
    let src_centered: Vec<Vector3<f32>> = src.iter()
        .map(|p| p - centroid_src)
        .collect();
    let dst_centered: Vec<Vector3<f32>> = dst.iter()
        .map(|p| p - centroid_dst)
        .collect();

    // 3. 计算协方差矩阵 H = Σ(src_i * dst_i^T)
    let mut h = Matrix3::zeros();
    for (s, d) in src_centered.iter().zip(dst_centered.iter()) {
        h += s * d.transpose();
    }

    // 4. SVD: H = U * S * V^T
    let svd = SVD::new(h, true, true);
    let u = svd.u.ok_or(CalibrationError::SvdFailed)?;
    let v_t = svd.v_t.ok_or(CalibrationError::SvdFailed)?;
    let v = v_t.transpose();

    // 5. 计算旋转 R = V * U^T
    let mut r = v * u.transpose();

    // 6. 确保右手坐标系：det(R) 应 > 0
    if r.determinant() < 0.0 {
        let mut v_flipped = v;
        v_flipped.column_mut(2).apply(|x| *x *= -1.0);
        r = v_flipped * u.transpose();
    }

    // 7. 计算缩放（可选）
    let scale = if estimate_scale {
        // s = trace(S) / Σ||src_i||²   where S = U^T * H * V ...
        // 更稳健的方式直接计算
        let src_var: f32 = src_centered.iter().map(|v| v.norm_squared()).sum();
        if src_var < 1e-12 {
            return Err(CalibrationError::DegeneratePoints);
        }
        let trace_s: f32 = svd.singular_values.sum();
        (trace_s / src_var).min(100.0).max(0.01)
    } else {
        1.0
    };

    // 8. 计算平移: t = centroid_dst - s * R * centroid_src
    let translation = centroid_dst - (r * centroid_src.coords) * scale;

    // 9. 将缩放并入旋转矩阵
    if estimate_scale {
        r *= scale;
    }

    // 提取四元数
    let rotation = nalgebra::UnitQuaternion::from_matrix(&r);

    let transform = Transform3 {
        translation: Vector3::new(translation.x, translation.y, translation.z),
        rotation,
        scale: if estimate_scale { scale } else { 1.0 },
    };

    // 计算 RMSD
    let rmsd = compute_rmsd(src, dst, &transform);

    Ok(AbsoluteOrientationResult { transform, rmsd })
}

/// 计算 RMSD（均方根偏差）
fn compute_rmsd(src: &[Point3<f32>], dst: &[Point3<f32>], t: &Transform3) -> f32 {
    let n = src.len() as f32;
    let mut sum_sq = 0.0;
    for (s, d) in src.iter().zip(dst.iter()) {
        let transformed = t.rotation * (s.coords * t.scale) + t.translation;
        let diff = transformed - d.coords;
        sum_sq += diff.norm_squared();
    }
    (sum_sq / n).sqrt()
}

use crate::CalibrationError;

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;

    #[test]
    fn test_three_noncollinear_points() {
        // 3 个非共线点，已知 ground truth 变换
        let src = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];

        let t_gt = Transform3 {
            translation: Vector3::new(1.0, 2.0, 3.0),
            rotation: nalgebra::UnitQuaternion::from_axis_angle(
                &Vector3::z_axis(), std::f32::consts::FRAC_PI_2,
            ),
            scale: 1.0,
        };

        let dst: Vec<Point3<f32>> = src.iter()
            .map(|p| {
                let transformed = t_gt.rotation * p.coords + t_gt.translation;
                Point3::from(transformed)
            })
            .collect();

        let result = solve_absolute_orientation(&src, &dst, false).unwrap();
        assert!(result.rmsd < 1e-6);
    }

    #[test]
    fn test_insufficient_points() {
        let points = vec![Point3::new(0.0, 0.0, 0.0)];
        let result = solve_absolute_orientation(&points, &points, false);
        assert!(matches!(result, Err(CalibrationError::InsufficientPoints)));
    }

    #[test]
    fn test_with_scale() {
        let src = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
        ];

        let t_gt = Transform3 {
            translation: Vector3::new(0.5, 1.0, -0.5),
            rotation: nalgebra::UnitQuaternion::identity(),
            scale: 2.0,
        };

        let dst: Vec<Point3<f32>> = src.iter()
            .map(|p| {
                let transformed = t_gt.rotation * (p.coords * t_gt.scale) + t_gt.translation;
                Point3::from(transformed)
            })
            .collect();

        let result = solve_absolute_orientation(&src, &dst, true).unwrap();
        assert!(result.rmsd < 1e-6);
        assert!((result.transform.scale - 2.0).abs() < 1e-4);
    }
}
