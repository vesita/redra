/// TOML 配置到 Unit 协议数据的转换模块
/// 
/// 提供从 TOML 配置文件加载静态场景并转换为标准 Unit 协议数据的功能

use serde::Deserialize;
use std::fs;

/// 静态场景配置结构体
#[derive(Debug, Clone, Deserialize)]
pub struct StaticSceneConfig {
    pub global: GlobalConfig,
    #[serde(default)]
    pub entities: Vec<EntityConfig>,
}

/// 全局配置
#[derive(Debug, Clone, Deserialize)]
pub struct GlobalConfig {
    pub enabled: bool,
    pub description: String,
}

/// 实体配置
#[derive(Debug, Clone, Deserialize)]
pub struct EntityConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub position: [f32; 3],
    pub rotation: [f32; 3],  // 可以是弧度或角度，需要检测
    pub scale: [f32; 3],
    pub material: String,
    // 几何体参数（根据类型可选）
    pub radius: Option<f32>,
    pub height: Option<f32>,
    // 用于指定是否使用角度制
    #[serde(default)]
    pub degrees: bool,
}

/// 从 TOML 文件加载静态场景配置
/// 
/// # 参数
/// * `config_path` - TOML 配置文件路径
/// 
/// # 返回
/// * `Ok(StaticSceneConfig)` - 解析成功的配置
/// * `Err(String)` - 错误信息
/// 
/// # 示例
/// ```no_run
/// use expto::config::load_static_scene_config;
/// 
/// let config = load_static_scene_config("assets/init/default_scene.toml")
///     .expect("Failed to load config");
/// ```
pub fn load_static_scene_config(config_path: &str) -> Result<StaticSceneConfig, String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    let config: StaticSceneConfig = toml::from_str(&content)
        .map_err(|e| format!("TOML 解析失败: {}", e))?;
    
    Ok(config)
}

/// 将实体配置转换为 Unit 协议数据
/// 
/// # 参数
/// * `config` - 实体配置
/// * `entity_id` - 实体唯一标识（从1开始，避免与动态实体冲突）
/// 
/// # 返回
/// 标准的 Unit 协议数据，包含 Id、Mesh、Transform 和 MaterialId 对象
/// 
/// # 示例
/// ```no_run
/// use expto::config::{EntityConfig, config_to_unit};
/// 
/// let config = EntityConfig {
///     name: "X_Axis".to_string(),
///     entity_type: "cylinder".to_string(),
///     position: [0.0, 1.5, 0.0],
///     rotation: [0.0, 0.0, -1.5708],
///     scale: [1.0, 1.0, 1.0],
///     material: "materials/base/red.toml".to_string(),
///     radius: Some(0.075),
///     height: Some(3.0),
///     degrees: false,
/// };
/// 
/// let unit = config_to_unit(&config, 1);
/// ```
pub fn config_to_unit(config: &EntityConfig, entity_id: u64) -> crate::rdmp::Unit {
    use crate::rdmp::{ExTransform, ex_object::UObject};
    
    let mut unit = crate::rdmp::auto::unit::generate_unit();
    
    // 设置命令类型为 Spawn
    unit.set_spawn().expect("Failed to set spawn command");
    
    // 构建网格数据
    let mesh = build_mesh_from_config(config);
    
    // 构建变换数据（处理角度到弧度的转换）
    let rotation_rad = if config.degrees {
        log::debug!(
            "实体 '{}' 使用角度制旋转: [{:.1}, {:.1}, {:.1}]°",
            config.name,
            config.rotation[0],
            config.rotation[1],
            config.rotation[2]
        );
        // 如果标记为角度制，则转换为弧度
        [
            config.rotation[0].to_radians(),
            config.rotation[1].to_radians(),
            config.rotation[2].to_radians(),
        ]
    } else {
        log::debug!(
            "实体 '{}' 使用弧度制旋转: [{:.3}, {:.3}, {:.3}] rad",
            config.name,
            config.rotation[0],
            config.rotation[1],
            config.rotation[2]
        );
        // 默认已经是弧度制
        config.rotation
    };
    
    let transform = ExTransform {
        x: config.position[0],
        y: config.position[1],
        z: config.position[2],
        rx: rotation_rad[0],
        ry: rotation_rad[1],
        rz: rotation_rad[2],
        sx: config.scale[0],
        sy: config.scale[1],
        sz: config.scale[2],
    };
    
    // 添加对象到 Unit（按照 react_spawn 的期望顺序：Id + Mesh + Transform + MaterialId）
    unit.objects = vec![
        crate::rdmp::ExObject {
            u_object: Some(UObject::Id(entity_id)),
        },
        crate::rdmp::ExObject {
            u_object: Some(UObject::Mesh(mesh)),
        },
        crate::rdmp::ExObject {
            u_object: Some(UObject::Transform(transform)),
        },
        crate::rdmp::ExObject {
            u_object: Some(UObject::MaterialId(config.material.clone())),
        },
    ];
    
    unit
}

/// 根据配置构建网格数据
fn build_mesh_from_config(config: &EntityConfig) -> crate::rdmp::ExMesh {
    use crate::rdmp::Point;
    use crate::rdmp::mesh::ex_mesh::UMesh;
    
    let mesh = match config.entity_type.as_str() {
        "cylinder" => {
            let radius = config.radius.unwrap_or(0.1);
            let height = config.height.unwrap_or(1.0);
            UMesh::Cylinder(crate::rdmp::Cylinder { radius, height })
        }
        "cone" => {
            let radius = config.radius.unwrap_or(0.1);
            let height = config.height.unwrap_or(1.0);
            UMesh::Cone(crate::rdmp::Cone { radius, height })
        }
        "sphere" => {
            let radius = config.radius.unwrap_or(0.5);
            UMesh::Sphere(crate::rdmp::Sphere { 
                location: Some(Point { x: 0.0, y: 0.0, z: 0.0 }),
                radius 
            })
        }
        _ => {
            log::warn!("未知的实体类型: {}，使用默认球体", config.entity_type);
            UMesh::Sphere(crate::rdmp::Sphere { 
                location: Some(Point { x: 0.0, y: 0.0, z: 0.0 }),
                radius: 0.5 
            })
        }
    };
    
    crate::rdmp::ExMesh { u_mesh: Some(mesh) }
}
