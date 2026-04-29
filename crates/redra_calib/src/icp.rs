use nalgebra::Point3;
use redra_geo::Transform3;

use crate::CalibrationError;

/// ICP 参数
#[derive(Debug, Clone)]
pub struct IcpParams {
    /// 最大迭代次数
    pub max_iterations: usize,
    /// 收敛阈值（平移变化量）
    pub convergence_translation: f32,
    /// 收敛阈值（旋转角度变化量，弧度）
    pub convergence_rotation: f32,
}

impl Default for IcpParams {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            convergence_translation: 1e-6,
            convergence_rotation: 1e-6,
        }
    }
}

/// ICP 结果
#[derive(Debug, Clone)]
pub struct IcpResult {
    pub transform: Transform3,
    pub rmsd: f32,
    pub iterations: usize,
    pub converged: bool,
}

/// ICP 精配准（点到点，暴力最近邻）
///
/// 给定初始变换，通过迭代最近点算法优化变换矩阵。
/// 使用暴力 O(N*M) 最近邻搜索，适用于小规模点云（< 1000 点）。
pub fn icp_refine(
    src: &[Point3<f32>],
    dst: &[Point3<f32>],
    initial: &Transform3,
    params: &IcpParams,
) -> Result<IcpResult, CalibrationError> {
    if src.is_empty() || dst.is_empty() {
        return Err(CalibrationError::InsufficientPoints);
    }

    let mut current = initial.clone();

    for iter in 0..params.max_iterations {
        // 1. 变换源点
        let transformed_src: Vec<Point3<f32>> = src.iter()
            .map(|p| {
                let v = current.rotation * (p.coords * current.scale) + current.translation;
                Point3::from(v)
            })
            .collect();

        // 2. 为每个变换后的源点找到最近的目标点（暴力）
        let mut correspondences = Vec::with_capacity(src.len());
        for s in &transformed_src {
            let nearest = dst.iter()
                .min_by(|a, b| {
                    let da = (s.coords - a.coords).norm_squared();
                    let db = (s.coords - b.coords).norm_squared();
                    da.partial_cmp(&db).unwrap()
                })
                .ok_or(CalibrationError::InsufficientPoints)?;
            correspondences.push(nearest.clone());
        }

        // 3. 用 Kabsch 计算增量变换（从 transformed_src → 最近点）
        let delta = super::kabsch::solve_absolute_orientation(&transformed_src, &correspondences, false)?;

        // 4. 合成变换: T_new = T_delta * T_current
        //    即 R_new = R_d * R_c, t_new = R_d * t_c + t_d, s_new = s_c * s_d
        let d = &delta.transform;
        let translation = d.rotation * current.translation + d.translation;
        let rotation = d.rotation * current.rotation;
        let scale = current.scale * d.scale;
        current = Transform3 { translation, rotation, scale };

        // 5. 检查收敛
        if d.translation.norm() < params.convergence_translation
            && d.rotation.angle() < params.convergence_rotation
        {
            let rmsd = compute_rmsd_icp(src, dst, &current);
            return Ok(IcpResult {
                transform: current,
                rmsd,
                iterations: iter + 1,
                converged: true,
            });
        }
    }

    let rmsd = compute_rmsd_icp(src, dst, &current);
    Ok(IcpResult {
        transform: current,
        rmsd,
        iterations: params.max_iterations,
        converged: false,
    })
}

fn compute_rmsd_icp(src: &[Point3<f32>], dst: &[Point3<f32>], t: &Transform3) -> f32 {
    let n = src.len() as f32;
    let mut sum_sq = 0.0;
    for s in src {
        let v = t.rotation * (s.coords * t.scale) + t.translation;
        // 对每个变换后的点找最近目标点计算误差
        let min_dist = dst.iter()
            .map(|d| (v - d.coords).norm_squared())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        sum_sq += min_dist;
    }
    (sum_sq / n).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Point3, Vector3};

    #[test]
    fn test_icp_convergence() {
        let gt = Transform3 {
            translation: Vector3::new(0.5, -0.3, 0.2),
            rotation: nalgebra::UnitQuaternion::from_axis_angle(
                &Vector3::y_axis(),
                0.3,
            ),
            scale: 1.0,
        };

        let initial = Transform3::identity();
        let src = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.0, 1.0, 1.0),
        ];
        let dst: Vec<Point3<f32>> = src.iter()
            .map(|p| {
                let v = gt.rotation * (p.coords * gt.scale) + gt.translation;
                Point3::from(v)
            })
            .collect();

        let result = icp_refine(&src, &dst, &initial, &IcpParams::default()).unwrap();
        assert!(result.converged);
        assert!(result.rmsd < 1e-4);
    }
}
