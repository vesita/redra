use nalgebra::Point3;

/// 3D 对应点对
#[derive(Debug, Clone, PartialEq)]
pub struct Correspondence3D {
    /// 源点（如：待配准点云中的点）
    pub src: Point3<f32>,
    /// 目标点（如：参考点云中的对应点）
    pub dst: Point3<f32>,
}

impl Correspondence3D {
    pub fn new(src: Point3<f32>, dst: Point3<f32>) -> Self {
        Self { src, dst }
    }
}
