use bevy::prelude::*;
use expto::rdmp::{Unit, object::ex_object::UObject, mesh::ex_mesh::UMesh};
use std::collections::HashMap;

// 导出 bevy_materialize 的类型，方便其他模块使用
pub use bevy_materialize::prelude::{GenericMaterial, GenericMaterial3d};

// ==================== 材质管理器 ====================

/// MaterialManager - 材质管理系统（基于 bevy_materialize 优化）
/// 
/// 职责：
/// 1. 维护 material_id 到材质文件路径的映射
/// 2. 为其他模块提供材质查询服务
/// 3. 利用 bevy_materialize 自动加载 TOML 材质文件
/// 
/// 优势：
/// - 无需手动解析 TOML，bevy_materialize 自动处理
/// - 支持材质继承，减少配置重复
/// - 支持热重载，修改 TOML 文件自动生效
/// - 自动处理纹理路径（相对路径）
#[derive(Resource)]
pub struct MaterialManager {
    /// material_id 到材质文件路径的映射（相对于 assets/materials/）
    pub material_id_map: HashMap<String, String>,
}

impl Default for MaterialManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialManager {
    /// 创建新的材质管理器
    pub fn new() -> Self {
        let mut manager = MaterialManager {
            material_id_map: HashMap::new(),
        };
        
        // 注册默认的 material_id 映射
        manager.register_default_material_id_mappings();
        
        info!("材质管理器初始化完成（使用 bevy_materialize）");
        manager
    }

    // ==================== 核心功能：material_id 映射 ====================

    /// 注册默认的 material_id 到材质文件路径的映射
    /// 
    /// 材质文件按用途分类存储在 assets/materials/ 的子目录中：
    /// - base/          : 基础颜色材质（RGB 预设）
    /// - mesh_types/    : 网格类型专用材质
    /// - effects/       : 特殊效果材质（金属、玻璃等）
    /// - ui/            : UI 和调试状态材质
    /// - axes/          : 坐标轴材质
    fn register_default_material_id_mappings(&mut self) {
        // ===== 基础颜色材质 (base/) =====
        self.material_id_map.insert("red".to_string(), "materials/base/red.toml".to_string());
        self.material_id_map.insert("green".to_string(), "materials/base/green.toml".to_string());
        self.material_id_map.insert("blue".to_string(), "materials/base/blue.toml".to_string());
        self.material_id_map.insert("white".to_string(), "materials/base/white.toml".to_string());
        self.material_id_map.insert("yellow".to_string(), "materials/base/yellow.toml".to_string());
        self.material_id_map.insert("cyan".to_string(), "materials/base/cyan.toml".to_string());
        self.material_id_map.insert("magenta".to_string(), "materials/base/magenta.toml".to_string());
        
        // ===== 网格类型专用材质 (mesh_types/) =====
        self.material_id_map.insert("point".to_string(), "materials/mesh_types/point.toml".to_string());
        self.material_id_map.insert("line".to_string(), "materials/mesh_types/line.toml".to_string());
        self.material_id_map.insert("sphere".to_string(), "materials/mesh_types/sphere.toml".to_string());
        self.material_id_map.insert("cylinder".to_string(), "materials/mesh_types/cylinder.toml".to_string());
        self.material_id_map.insert("cone".to_string(), "materials/mesh_types/cone.toml".to_string());
        
        // 兼容旧的 cube 映射
        self.material_id_map.insert("cube".to_string(), "materials/default.toml".to_string());
        
        // ===== 特殊效果材质 (effects/) =====
        self.material_id_map.insert("metal".to_string(), "materials/effects/metal.toml".to_string());
        self.material_id_map.insert("glass".to_string(), "materials/effects/glass.toml".to_string());
        self.material_id_map.insert("glow".to_string(), "materials/effects/glow.toml".to_string());
        self.material_id_map.insert("matte".to_string(), "materials/effects/matte.toml".to_string());
        self.material_id_map.insert("plastic".to_string(), "materials/effects/plastic.toml".to_string());
        self.material_id_map.insert("wood".to_string(), "materials/effects/wood.toml".to_string());
        
        // ===== 坐标轴材质 (axes/) =====
        self.material_id_map.insert("axis_x".to_string(), "materials/axes/x_axis.toml".to_string());
        self.material_id_map.insert("axis_y".to_string(), "materials/axes/y_axis.toml".to_string());
        self.material_id_map.insert("axis_z".to_string(), "materials/axes/z_axis.toml".to_string());
        
        // ===== UI 和调试状态材质 (ui/) =====
        self.material_id_map.insert("wireframe".to_string(), "materials/ui/wireframe.toml".to_string());
        self.material_id_map.insert("highlight".to_string(), "materials/ui/highlight.toml".to_string());
        self.material_id_map.insert("disabled".to_string(), "materials/ui/disabled.toml".to_string());
        
        // ===== 默认材质 =====
        self.material_id_map.insert("default".to_string(), "materials/default.toml".to_string());
    }

    // ==================== 对外 API：材质查询 ====================

    /// 根据 material_id 获取材质文件路径
    /// 
    /// # 示例
    /// ```
    /// if let Some(path) = material_manager.resolve_material_id("metal") {
    ///     // path = "materials/metal.toml"
    ///     let handle = asset_server.load::<GenericMaterial>(path);
    /// }
    /// ```
    pub fn resolve_material_id(&self, material_id: &str) -> Option<&str> {
        self.material_id_map.get(material_id).map(|s| s.as_str())
    }

    /// 从 Unit 中提取 material_id（如果存在）
    pub fn extract_material_id_from_unit(&self, unit: &Unit) -> Option<String> {
        for obj in &unit.objects {
            if let Some(UObject::MaterialId(id)) = &obj.u_object {
                return Some(id.clone());
            }
        }
        None
    }

    /// 智能选择材质：优先使用 Unit 中的 material_id，否则根据网格类型自动匹配
    /// 
    /// # 选择优先级
    /// 1. Unit 中显式指定的 material_id
    /// 2. 根据网格类型自动匹配（Point→point_material, Line→line_material等）
    /// 3. Fallback 到 "default"
    /// 
    /// # 返回
    /// 材质文件路径（相对于 assets 目录）
    pub fn select_material_smart(&self, unit: &Unit) -> &str {
        // 1. 尝试从 Unit 中提取 material_id
        if let Some(material_id) = self.extract_material_id_from_unit(unit) {
            if let Some(material_path) = self.resolve_material_id(&material_id) {
                return material_path;
            }
        }
        
        // 2. 根据网格类型自动匹配
        self.select_by_mesh_type(unit)
            .unwrap_or("materials/default.toml")
    }

    /// 根据网格类型选择材质文件路径
    fn select_by_mesh_type(&self, unit: &Unit) -> Option<&str> {
        for obj in &unit.objects {
            if let Some(UObject::Mesh(mesh)) = &obj.u_object {
                return match &mesh.u_mesh {
                    Some(UMesh::Point(_)) => Some("materials/mesh_types/point.toml"),
                    Some(UMesh::Line(_)) => Some("materials/mesh_types/line.toml"),
                    Some(UMesh::Sphere(_)) => Some("materials/mesh_types/sphere.toml"),
                    Some(UMesh::Cylinder(_)) => Some("materials/mesh_types/cylinder.toml"),
                    Some(UMesh::Cone(_)) => Some("materials/mesh_types/cone.toml"),
                    None => None,
                };
            }
        }
        Some("materials/default.toml")
    }

    /// 根据 RGB 颜色值选择最接近的材质文件路径
    pub fn select_material_by_color(&self, color: &[f32; 4]) -> &str {
        if color[0] > 0.7 && color[1] < 0.3 && color[2] < 0.3 {
            "materials/base/red.toml"
        } else if color[0] < 0.3 && color[1] > 0.7 && color[2] < 0.3 {
            "materials/base/green.toml"
        } else if color[0] < 0.3 && color[1] < 0.3 && color[2] > 0.7 {
            "materials/base/blue.toml"
        } else {
            "materials/default.toml"
        }
    }

    // ==================== 便捷 API：坐标轴材质 ====================

    /// 获取坐标轴 X 材质文件路径
    pub fn get_axis_x_material_path(&self) -> &str {
        "materials/axes/x_axis.toml"
    }

    /// 获取坐标轴 Y 材质文件路径
    pub fn get_axis_y_material_path(&self) -> &str {
        "materials/axes/y_axis.toml"
    }

    /// 获取坐标轴 Z 材质文件路径
    pub fn get_axis_z_material_path(&self) -> &str {
        "materials/axes/z_axis.toml"
    }

    // ==================== 兼容 API：与旧代码集成 ====================

    /// 根据材质名称加载 GenericMaterial（兼容旧 API）
    /// 
    /// # 参数
    /// * `material_name` - 材质名称或 material_id
    /// * `asset_server` - Bevy 资源服务器
    /// 
    /// # 返回
    /// Handle<GenericMaterial>，可用于 GenericMaterial3d 组件
    /// 
    /// # 示例
    /// ```
    /// let handle = material_manager.load_generic_material("metal", &asset_server);
    /// commands.spawn((
    ///     Mesh3d(mesh),
    ///     GenericMaterial3d(handle),
    /// ));
    /// ```
    pub fn load_generic_material(
        &self,
        material_name: &str,
        asset_server: &AssetServer,
    ) -> Handle<GenericMaterial> {
        // 尝试解析为 material_id
        let path = if let Some(material_path) = self.resolve_material_id(material_name) {
            material_path.to_string()
        } else {
            // 如果已经是路径格式（包含 .toml），直接使用
            if material_name.ends_with(".toml") {
                material_name.to_string()
            } else {
                // 否则当作默认材质处理
                warn!("未知材质 '{}', 使用默认材质", material_name);
                "materials/default.toml".to_string()
            }
        };
        
        asset_server.load::<GenericMaterial>(path)
    }

    /// 批量加载多个材质
    pub fn load_generic_materials(
        &self,
        material_names: &[&str],
        asset_server: &AssetServer,
    ) -> Vec<Handle<GenericMaterial>> {
        material_names
            .iter()
            .map(|&name| self.load_generic_material(name, asset_server))
            .collect()
    }
}
