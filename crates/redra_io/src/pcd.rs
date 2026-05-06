use std::path::Path;

use expto::rdmp::{ExMesh, ExTransform, Point};

/// PCD 文件解析结果
pub struct PcdFrame {
    pub points: Vec<(f32, f32, f32)>,
}

/// PCD 加载错误
#[derive(Debug)]
pub enum PcdLoadError {
    Io(std::io::Error),
    Parse(String),
    MissingField(String),
}

impl std::fmt::Display for PcdLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PcdLoadError::Io(e) => write!(f, "IO 错误: {}", e),
            PcdLoadError::Parse(msg) => write!(f, "解析错误: {}", msg),
            PcdLoadError::MissingField(name) => write!(f, "缺少字段: {}", name),
        }
    }
}

impl From<std::io::Error> for PcdLoadError {
    fn from(e: std::io::Error) -> Self {
        PcdLoadError::Io(e)
    }
}

/// 读取 PCD 文件，提取所有点的 (x, y, z) 坐标
pub fn load_pcd(path: &Path) -> Result<PcdFrame, PcdLoadError> {
    use pcd_rs::DynReader;

    let mut reader = DynReader::open(path).map_err(|e| PcdLoadError::Parse(format!("打开 PCD 文件失败: {}", e)))?;

    // 从 schema 中查找 x, y, z 字段的索引
    let schema = &reader.meta().field_defs.fields;
    let x_idx = find_field_index(schema, "x")?;
    let y_idx = find_field_index(schema, "y")?;
    let z_idx = find_field_index(schema, "z")?;

    let mut points = Vec::new();

    for result in reader.by_ref() {
        let record = result.map_err(|e| PcdLoadError::Parse(format!("读取点数据失败: {}", e)))?;
        let x = extract_field_f32(&record.0, x_idx, "x")?;
        let y = extract_field_f32(&record.0, y_idx, "y")?;
        let z = extract_field_f32(&record.0, z_idx, "z")?;
        points.push((x, y, z));
    }

    log::info!("从 PCD 文件加载了 {} 个点", points.len());
    Ok(PcdFrame { points })
}

/// 将点云转为 (entity_id, ExMesh, ExTransform) 列表，供构建 KeyFrame 使用
pub fn points_to_entities(points: &[(f32, f32, f32)]) -> Vec<(u64, ExMesh, ExTransform)> {
    points
        .iter()
        .enumerate()
        .map(|(i, &(x, y, z))| {
            let id = i as u64;
            let mesh = ExMesh::from(Point::from((x, y, z)));
            let transform = ExTransform {
                x,
                y,
                z,
                rx: 0.0,
                ry: 0.0,
                rz: 0.0,
                sx: 1.0,
                sy: 1.0,
                sz: 1.0,
            };
            (id, mesh, transform)
        })
        .collect()
}

fn find_field_index(
    schema: &[pcd_rs::metas::FieldDef],
    name: &str,
) -> Result<usize, PcdLoadError> {
    schema
        .iter()
        .position(|f| f.name == name)
        .ok_or_else(|| PcdLoadError::MissingField(name.into()))
}

fn extract_field_f32(
    fields: &[pcd_rs::Field],
    idx: usize,
    name: &str,
) -> Result<f32, PcdLoadError> {
    use pcd_rs::Field;

    let field = fields
        .get(idx)
        .ok_or_else(|| PcdLoadError::MissingField(name.into()))?;

    match field {
        Field::F32(vec) => vec.first().copied().ok_or_else(|| PcdLoadError::Parse(format!("字段 '{}' 为空", name))),
        Field::F64(vec) => vec.first().map(|&v| v as f32).ok_or_else(|| PcdLoadError::Parse(format!("字段 '{}' 为空", name))),
        Field::I8(vec) => vec.first().map(|&v| v as f32).ok_or_else(|| PcdLoadError::Parse(format!("字段 '{}' 为空", name))),
        Field::I16(vec) => vec.first().map(|&v| v as f32).ok_or_else(|| PcdLoadError::Parse(format!("字段 '{}' 为空", name))),
        Field::I32(vec) => vec.first().map(|&v| v as f32).ok_or_else(|| PcdLoadError::Parse(format!("字段 '{}' 为空", name))),
        Field::I64(vec) => vec.first().map(|&v| v as f32).ok_or_else(|| PcdLoadError::Parse(format!("字段 '{}' 为空", name))),
        Field::U8(vec) => vec.first().map(|&v| v as f32).ok_or_else(|| PcdLoadError::Parse(format!("字段 '{}' 为空", name))),
        Field::U16(vec) => vec.first().map(|&v| v as f32).ok_or_else(|| PcdLoadError::Parse(format!("字段 '{}' 为空", name))),
        Field::U32(vec) => vec.first().map(|&v| v as f32).ok_or_else(|| PcdLoadError::Parse(format!("字段 '{}' 为空", name))),
        Field::U64(vec) => vec.first().map(|&v| v as f32).ok_or_else(|| PcdLoadError::Parse(format!("字段 '{}' 为空", name))),
    }
}
