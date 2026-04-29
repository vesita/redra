pub mod correspondence;
pub mod kabsch;
pub mod icp;
pub mod plugin;
pub mod prelude;

/// 标定算法错误类型
#[derive(Debug)]
pub enum CalibrationError {
    InsufficientPoints,
    MismatchedPointCounts(usize, usize),
    DegeneratePoints,
    SvdFailed,
}

impl std::fmt::Display for CalibrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalibrationError::InsufficientPoints => write!(f, "至少需要 3 个对应点"),
            CalibrationError::MismatchedPointCounts(s, d) => {
                write!(f, "源点数量 ({}) 与目标点数量 ({}) 不匹配", s, d)
            }
            CalibrationError::DegeneratePoints => write!(f, "点共线或退化为平面"),
            CalibrationError::SvdFailed => write!(f, "SVD 分解失败"),
        }
    }
}

impl std::error::Error for CalibrationError {}
