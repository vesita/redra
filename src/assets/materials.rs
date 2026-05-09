use bevy::prelude::*;
use expto::rdmp::{Unit, object::ex_object::UObject, mesh::ex_mesh::UMesh};
use std::collections::HashMap;

pub use bevy_materialize::prelude::{GenericMaterial, GenericMaterial3d};

/// 材质管理器 — 管理 material_id 到材质文件路径的映射
///
/// 支持分类访问：`manager.base("red")`、`manager.cluster(1)`、`manager.semantic("alert")`
/// 支持自省：`manager.list_materials()` 返回所有已注册材质的分类列表
#[derive(Resource)]
pub struct MaterialManager {
    pub material_id_map: HashMap<String, String>,
}

impl Default for MaterialManager {
    fn default() -> Self { Self::new() }
}

// ─── 注册辅助宏 ──────────────────────────────────────────────

/// 批量注册特殊映射（路径不遵循 `materials/{category}/{name}.toml` 规则）
macro_rules! register_special {
    ($self:ident, { $( ($name:expr, $path:expr) ),+ $(,)? }) => {
        $( $self.material_id_map.insert($name.to_string(), $path.to_string()); )+
    };
}

impl MaterialManager {
    pub fn new() -> Self {
        let mut manager = MaterialManager { material_id_map: HashMap::new() };
        manager.register_default_material_id_mappings();
        info!("材质管理器初始化完成（{} 个已注册材质）", manager.material_id_map.len());
        manager
    }

    /// 按分类批量注册（路径规则：`materials/{category}/{name}.toml`）
    fn register_category(&mut self, category: &str, names: &[&str]) {
        for &name in names {
            self.material_id_map.insert(
                name.to_string(),
                format!("materials/{}/{}.toml", category, name),
            );
        }
    }

    fn register_default_material_id_mappings(&mut self) {
        // ── 按分类批量注册 ──────────────────────────────────────
        self.register_category("base", &[
            "red", "green", "blue", "white", "yellow", "cyan", "magenta",
            "bright_blue", "bright_yellow", "bright_cyan",
        ]);
        self.register_category("data", &[
            "cluster_01", "cluster_02", "cluster_03", "cluster_04",
            "cluster_05", "cluster_06", "cluster_07", "cluster_08",
            "cluster_09", "cluster_10", "cluster_11", "cluster_12",
        ]);
        self.register_category("semantic", &[
            "point_cloud", "ground", "wall", "obstacle", "cluster_bg",
            "noise", "bounding_box",
            "trajectory", "selected", "alert", "sky",
        ]);
        self.register_category("effects", &[
            "metal", "glass", "glow", "matte", "plastic", "wood",
        ]);
        self.register_category("mesh_types", &[
            "point", "line", "sphere", "cylinder", "cone",
        ]);
        self.register_category("ui", &[
            "wireframe", "highlight", "disabled",
        ]);

        // ── 特殊映射（路径不遵循命名规则）─────────────────────────
        register_special!(self, {
            ("default",  "materials/default.toml"),
            ("cube",     "materials/default.toml"),
            ("axis_x",   "materials/axes/x_axis.toml"),
            ("axis_y",   "materials/axes/y_axis.toml"),
            ("axis_z",   "materials/axes/z_axis.toml"),
            ("x_axis",   "materials/axes/x_axis.toml"),
            ("y_axis",   "materials/axes/y_axis.toml"),
            ("z_axis",   "materials/axes/z_axis.toml"),
        });
    }

    // ==================== 分类访问 API ====================

    /// 按分类和名称加载材质
    pub fn get(&self, category: &str, name: &str, asset_server: &AssetServer) -> Handle<GenericMaterial> {
        let alias = match category {
            "base" | "data" | "semantic" | "effects" | "mesh_types" | "ui" => name.to_string(),
            _ => {
                warn!("未知材质分类 '{}'", category);
                return asset_server.load::<GenericMaterial>("materials/default.toml");
            }
        };
        self.load_generic_material(&alias, asset_server)
    }

    /// 基础色加载（`"red"`, `"green"`, `"blue"` 等）
    pub fn base(&self, name: &str, asset_server: &AssetServer) -> Handle<GenericMaterial> {
        self.load_generic_material(name, asset_server)
    }

    /// 聚类色板加载（1-based 索引，自动 clamp 到 1..=12）
    pub fn cluster(&self, index: u8, asset_server: &AssetServer) -> Handle<GenericMaterial> {
        let clamped = index.clamp(1, 12);
        self.load_generic_material(&format!("cluster_{:02}", clamped), asset_server)
    }

    /// 语义色加载（`"point_cloud"`, `"ground"`, `"alert"` 等）
    pub fn semantic(&self, name: &str, asset_server: &AssetServer) -> Handle<GenericMaterial> {
        self.load_generic_material(name, asset_server)
    }

    /// 效果材质加载（`"metal"`, `"glass"`, `"glow"` 等）
    pub fn effects(&self, name: &str, asset_server: &AssetServer) -> Handle<GenericMaterial> {
        self.load_generic_material(name, asset_server)
    }

    /// 列出所有已注册材质（按路径前缀分组）
    ///
    /// 返回 `Vec<(分类名, 材质名列表)>`
    pub fn list_materials(&self) -> Vec<(String, Vec<String>)> {
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();
        for (name, path) in &self.material_id_map {
            let category = path
                .strip_prefix("materials/")
                .and_then(|s| s.split('/').next())
                .unwrap_or("other")
                .to_string();
            groups.entry(category).or_default().push(name.clone());
        }
        let mut result: Vec<_> = groups.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, names) in &mut result {
            names.sort();
        }
        result
    }

    // ==================== 原有 API（向后兼容）====================

    pub fn resolve_material_id(&self, material_id: &str) -> Option<&str> {
        self.material_id_map.get(material_id).map(|s| s.as_str())
    }

    pub fn extract_material_id_from_unit(&self, unit: &Unit) -> Option<String> {
        for obj in &unit.objects {
            if let Some(UObject::MaterialId(id)) = &obj.u_object {
                return Some(id.clone());
            }
        }
        None
    }

    pub fn select_material_smart(&self, unit: &Unit) -> &str {
        if let Some(material_id) = self.extract_material_id_from_unit(unit) {
            if let Some(material_path) = self.resolve_material_id(&material_id) {
                return material_path;
            }
        }
        self.select_by_mesh_type(unit).unwrap_or("materials/default.toml")
    }

    fn select_by_mesh_type(&self, unit: &Unit) -> Option<&str> {
        for obj in &unit.objects {
            if let Some(UObject::Mesh(mesh)) = &obj.u_object {
                return match &mesh.u_mesh {
                    Some(UMesh::Point(_)) => Some("materials/mesh_types/point.toml"),
                    Some(UMesh::Line(_)) => Some("materials/mesh_types/line.toml"),
                    Some(UMesh::Sphere(_)) => Some("materials/mesh_types/sphere.toml"),
                    Some(UMesh::Cylinder(_)) => Some("materials/mesh_types/cylinder.toml"),
                    Some(UMesh::Cone(_)) => Some("materials/mesh_types/cone.toml"),
                    Some(UMesh::Cube(_)) => Some("materials/mesh_types/cube.toml"),
                    None => None,
                };
            }
        }
        Some("materials/default.toml")
    }

    pub fn select_material_by_color(&self, color: &[f32; 4]) -> &str {
        if color[0] > 0.7 && color[1] < 0.3 && color[2] < 0.3 { "materials/base/red.toml" }
        else if color[0] < 0.3 && color[1] > 0.7 && color[2] < 0.3 { "materials/base/green.toml" }
        else if color[0] < 0.3 && color[1] < 0.3 && color[2] > 0.7 { "materials/base/blue.toml" }
        else { "materials/default.toml" }
    }

    pub fn get_axis_x_material_path(&self) -> &str { "materials/axes/x_axis.toml" }
    pub fn get_axis_y_material_path(&self) -> &str { "materials/axes/y_axis.toml" }
    pub fn get_axis_z_material_path(&self) -> &str { "materials/axes/z_axis.toml" }

    pub fn load_generic_material(&self, material_name: &str, asset_server: &AssetServer) -> Handle<GenericMaterial> {
        let path = if let Some(material_path) = self.resolve_material_id(material_name) {
            material_path.to_string()
        } else if material_name.ends_with(".toml") {
            material_name.to_string()
        } else {
            warn!("未知材质 '{}', 使用默认材质", material_name);
            "materials/default.toml".to_string()
        };
        asset_server.load::<GenericMaterial>(path)
    }

    pub fn load_generic_materials(&self, material_names: &[&str], asset_server: &AssetServer) -> Vec<Handle<GenericMaterial>> {
        material_names.iter().map(|&name| self.load_generic_material(name, asset_server)).collect()
    }
}
