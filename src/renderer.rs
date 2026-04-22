use bevy::prelude::*;

// Renderer 模块入口文件
// 统一导出渲染模块的所有公共接口
// 
// 职责：提供渲染执行服务，包括网格创建、材质应用、相机控制等
// 注意：根据数据与渲染职责分离规范，Renderer 不负责帧数据管理

pub mod interaction;
pub mod ui;
pub mod init;
pub mod frame_rate;
pub mod frame_renderer;      // 基于Frame的实体渲染器
pub mod scene_initializer;   // 静态场景初始化器

// ==================== 公共 API 导出 ====================

// 材质管理（从 manager 模块导入）
pub use crate::manager::materials::{MaterialManager, GenericMaterial, GenericMaterial3d};

// 交互相关
pub use interaction::InteractionPlugin;

// UI 相关
pub use ui::UiModule;

// 初始化相关
pub use init::InitPlugin;

// 帧率控制
pub use frame_rate::FrameRatePlugin;

// Frame 渲染器
pub use frame_renderer::FrameRendererPlugin;

// 场景初始化器
pub use scene_initializer::SceneInitializerPlugin;

// ==================== 基础转换工具 API ====================

/// 提供基础的 proto → Bevy 转换工具
/// 这些是纯函数，不涉及实体管理，可被 FrameRenderer 或其他模块复用
pub mod conversion {
    use super::*;
    use expto::rdmp::{ExMesh, ExTransform};
    
    /// 从 proto Mesh 转换为 Bevy Mesh
    pub fn proto_mesh_to_bevy(
        meshes: &mut Assets<Mesh>,
        proto_mesh: &ExMesh,
    ) -> Option<Mesh3d> {
        use expto::rdmp::mesh::ex_mesh::UMesh;
        
        let bevy_mesh = match &proto_mesh.u_mesh {
            Some(UMesh::Sphere(sphere)) => {
                Mesh3d(meshes.add(Sphere::new(sphere.radius)))
            }
            Some(UMesh::Point(_point)) => {
                // Point 用小球体表示
                Mesh3d(meshes.add(Sphere::new(0.05)))
            }
            Some(UMesh::Line(line)) => {
                let start = line.start.as_ref()?;
                let end = line.end.as_ref()?;
                
                let direction = Vec3::new(end.x - start.x, end.y - start.y, end.z - start.z);
                let length = direction.length();
                
                if length < 0.001 {
                    return None;
                }
                
                Mesh3d(meshes.add(Cylinder::new(0.02, length)))
            }
            Some(UMesh::Cylinder(cylinder)) => {
                Mesh3d(meshes.add(Cylinder::new(cylinder.radius, cylinder.height)))
            }
            Some(UMesh::Cone(cone)) => {
                Mesh3d(meshes.add(Cone::new(cone.radius, cone.height)))
            }
            None => return None,
        };
        
        Some(bevy_mesh)
    }
    
    /// 从 proto Transform 转换为 Bevy Transform
    pub fn proto_transform_to_bevy(transform: &ExTransform) -> Transform {
        Transform::from_translation(Vec3::new(transform.x, transform.y, transform.z))
            .with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                transform.rx,
                transform.ry,
                transform.rz,
            ))
            .with_scale(Vec3::new(transform.sx, transform.sy, transform.sz))
    }
}

// ==================== 便捷渲染 API ====================

/// 提供便捷的实体生成 API（适用于非 Frame 场景，如启动场景、调试工具等）
pub mod helpers {
    use super::*;
    use crate::manager::materials::MaterialManager;
    use expto::rdmp::{ExMesh, ExTransform};
    
    /// 快速生成单个实体（用于启动场景、测试等）
    pub fn spawn_entity(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        asset_server: &AssetServer,
        material_manager: &MaterialManager,
        mesh: &ExMesh,
        transform: &ExTransform,
        material_name: &str,
        name: &str,
    ) -> Entity {
        let mesh_handle = super::conversion::proto_mesh_to_bevy(meshes, mesh)
            .unwrap_or_else(|| {
                log::warn!("网格转换失败，使用备用球体");
                Mesh3d(meshes.add(Sphere::new(0.1)))
            });
        
        let material = material_manager.load_generic_material(material_name, asset_server);
        let transform_comp = super::conversion::proto_transform_to_bevy(transform);
        
        commands.spawn((
            mesh_handle,
            GenericMaterial3d(material),
            transform_comp,
            Name::new(name.to_string()),
        )).id()
    }
    
    /// 生成带箭头的坐标轴（圆柱体 + 圆锥）
    pub fn spawn_axis_with_arrow(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        asset_server: &AssetServer,
        material_manager: &MaterialManager,
        position: Vec3,
        rotation: Quat,
        length: f32,
        radius: f32,
        material_name: &str,
        name: &str,
    ) {
        let arrow_radius = radius * 2.0;
        let arrow_height = radius * 4.0;
        
        let material = material_manager.load_generic_material(material_name, asset_server);
        
        // 生成圆柱体主体
        let cylinder_mesh = meshes.add(Cylinder::new(radius, length));
        let cylinder_transform = Transform::from_translation(position + rotation * Vec3::new(0.0, length / 2.0, 0.0))
            .with_rotation(rotation);
        
        commands.spawn((
            Mesh3d(cylinder_mesh),
            GenericMaterial3d(material.clone()),
            cylinder_transform,
            Name::new(format!("{}_Body", name)),
        ));
        
        // 生成圆锥箭头
        let cone_mesh = meshes.add(Cone::new(arrow_radius, arrow_height));
        let cone_transform = Transform::from_translation(position + rotation * Vec3::new(0.0, length, 0.0))
            .with_rotation(rotation);
        
        commands.spawn((
            Mesh3d(cone_mesh),
            GenericMaterial3d(material),
            cone_transform,
            Name::new(format!("{}_Arrow", name)),
        ));
    }
}

// ==================== RendererPlugin ====================

/// 定义 RendererPlugin 来整合所有渲染相关的插件和系统
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(scene_initializer::SceneInitializerPlugin)  // 静态场景初始化（最先执行）
            .add_plugins(init::InitPlugin)                 // 其他初始化插件
            .add_plugins(interaction::InteractionPlugin)   // 交互插件
            .add_plugins(ui::UiModule)                     // UI插件
            .add_plugins(frame_rate::FrameRatePlugin)      // 帧率控制插件
            .add_plugins(frame_renderer::FrameRendererPlugin);  // Frame实体渲染器
    }
}
