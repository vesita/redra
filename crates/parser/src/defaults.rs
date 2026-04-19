/// 默认材质配置
#[derive(Debug, Clone)]
pub struct DefaultMaterialConfig {
    /// 材质 TOML 文件路径（相对于 assets 目录）
    pub material_path: String,
}

impl Default for DefaultMaterialConfig {
    fn default() -> Self {
        Self {
            material_path: "materials/default.toml".to_string(),
        }
    }
}

impl DefaultMaterialConfig {
    pub fn new(material_path: &str) -> Self {
        Self {
            material_path: material_path.to_string(),
        }
    }
}
