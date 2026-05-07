use bevy::prelude::*;
use expto::rdmp::{Unit, object::ex_object::UObject, mesh::ex_mesh::UMesh};
use std::collections::HashMap;

pub use bevy_materialize::prelude::{GenericMaterial, GenericMaterial3d};

/// 材质管理器 — 管理 material_id 到材质文件路径的映射
#[derive(Resource)]
pub struct MaterialManager {
    pub material_id_map: HashMap<String, String>,
}

impl Default for MaterialManager {
    fn default() -> Self { Self::new() }
}

impl MaterialManager {
    pub fn new() -> Self {
        let mut manager = MaterialManager { material_id_map: HashMap::new() };
        manager.register_default_material_id_mappings();
        info!("材质管理器初始化完成（使用 bevy_materialize）");
        manager
    }

    fn register_default_material_id_mappings(&mut self) {
        // 基础颜色材质
        self.material_id_map.insert("red".to_string(), "materials/base/red.toml".to_string());
        self.material_id_map.insert("green".to_string(), "materials/base/green.toml".to_string());
        self.material_id_map.insert("blue".to_string(), "materials/base/blue.toml".to_string());
        self.material_id_map.insert("white".to_string(), "materials/base/white.toml".to_string());
        self.material_id_map.insert("yellow".to_string(), "materials/base/yellow.toml".to_string());
        self.material_id_map.insert("cyan".to_string(), "materials/base/cyan.toml".to_string());
        self.material_id_map.insert("magenta".to_string(), "materials/base/magenta.toml".to_string());

        // 网格类型专用材质
        self.material_id_map.insert("point".to_string(), "materials/mesh_types/point.toml".to_string());
        self.material_id_map.insert("line".to_string(), "materials/mesh_types/line.toml".to_string());
        self.material_id_map.insert("sphere".to_string(), "materials/mesh_types/sphere.toml".to_string());
        self.material_id_map.insert("cylinder".to_string(), "materials/mesh_types/cylinder.toml".to_string());
        self.material_id_map.insert("cone".to_string(), "materials/mesh_types/cone.toml".to_string());
        self.material_id_map.insert("cube".to_string(), "materials/default.toml".to_string());

        // 特殊效果材质
        self.material_id_map.insert("metal".to_string(), "materials/effects/metal.toml".to_string());
        self.material_id_map.insert("glass".to_string(), "materials/effects/glass.toml".to_string());
        self.material_id_map.insert("glow".to_string(), "materials/effects/glow.toml".to_string());
        self.material_id_map.insert("matte".to_string(), "materials/effects/matte.toml".to_string());
        self.material_id_map.insert("plastic".to_string(), "materials/effects/plastic.toml".to_string());
        self.material_id_map.insert("wood".to_string(), "materials/effects/wood.toml".to_string());

        // 坐标轴材质
        self.material_id_map.insert("axis_x".to_string(), "materials/axes/x_axis.toml".to_string());
        self.material_id_map.insert("axis_y".to_string(), "materials/axes/y_axis.toml".to_string());
        self.material_id_map.insert("axis_z".to_string(), "materials/axes/z_axis.toml".to_string());

        // UI 材质
        self.material_id_map.insert("wireframe".to_string(), "materials/ui/wireframe.toml".to_string());
        self.material_id_map.insert("highlight".to_string(), "materials/ui/highlight.toml".to_string());
        self.material_id_map.insert("disabled".to_string(), "materials/ui/disabled.toml".to_string());

        // 默认材质
        self.material_id_map.insert("default".to_string(), "materials/default.toml".to_string());

        // 聚类色板（12 种感知均匀色，30° 色相间隔）
        self.material_id_map.insert("cluster_01".to_string(), "materials/data/cluster_01.toml".to_string());
        self.material_id_map.insert("cluster_02".to_string(), "materials/data/cluster_02.toml".to_string());
        self.material_id_map.insert("cluster_03".to_string(), "materials/data/cluster_03.toml".to_string());
        self.material_id_map.insert("cluster_04".to_string(), "materials/data/cluster_04.toml".to_string());
        self.material_id_map.insert("cluster_05".to_string(), "materials/data/cluster_05.toml".to_string());
        self.material_id_map.insert("cluster_06".to_string(), "materials/data/cluster_06.toml".to_string());
        self.material_id_map.insert("cluster_07".to_string(), "materials/data/cluster_07.toml".to_string());
        self.material_id_map.insert("cluster_08".to_string(), "materials/data/cluster_08.toml".to_string());
        self.material_id_map.insert("cluster_09".to_string(), "materials/data/cluster_09.toml".to_string());
        self.material_id_map.insert("cluster_10".to_string(), "materials/data/cluster_10.toml".to_string());
        self.material_id_map.insert("cluster_11".to_string(), "materials/data/cluster_11.toml".to_string());
        self.material_id_map.insert("cluster_12".to_string(), "materials/data/cluster_12.toml".to_string());

        // 语义色
        self.material_id_map.insert("point_cloud".to_string(), "materials/semantic/point_cloud.toml".to_string());
        self.material_id_map.insert("ground".to_string(), "materials/semantic/ground.toml".to_string());
        self.material_id_map.insert("noise".to_string(), "materials/semantic/noise.toml".to_string());
        self.material_id_map.insert("bounding_box".to_string(), "materials/semantic/bounding_box.toml".to_string());
        self.material_id_map.insert("trajectory".to_string(), "materials/semantic/trajectory.toml".to_string());
        self.material_id_map.insert("selected".to_string(), "materials/semantic/selected.toml".to_string());
        self.material_id_map.insert("alert".to_string(), "materials/semantic/alert.toml".to_string());
        self.material_id_map.insert("sky".to_string(), "materials/semantic/sky.toml".to_string());
    }

    // ==================== 对外 API ====================

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
