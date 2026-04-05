use bevy::prelude::*;
use smooth_bevy_cameras::controllers::fps::{FpsCameraBundle, FpsCameraController};
use crate::graph::materials::{MaterialManager, PredefinedMaterial};  // 添加导入

// 初始化插件
pub struct InitPlugin;

impl Plugin for InitPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, rd_setup)
            .add_systems(Startup, initialize_materials);
            // 移除了 toggle_camera_control，因为 FPS 控制器已有内置的 Alt 键处理
    }
}

pub fn rd_setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut global_ambient: ResMut<GlobalAmbientLight>,
) {
    // 设置全局环境光
    *global_ambient = GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: true,
    };

    // 添加FPS相机控制器
    commands
        .spawn((
            Camera3d::default(),
            Camera {
                order: 0,
                ..default()
            },
            Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                enabled: true,
                mouse_rotate_sensitivity: Vec2::new(0.1, 0.1),
                ..Default::default()
            },
            Vec3::new(-2.5, 4.5, 9.0),  // 相机位置
            Vec3::new(0.0, 0.0, 0.0),    // 看向的目标点
            Vec3::Y,
        ));

    // 添加坐标轴
    crate::graph::rendering::axis::spawn_axis_setup(commands, meshes, materials);
}

pub fn initialize_materials(
    mut commands: Commands,
) {
    // 创建材质管理器
    let mut material_manager = MaterialManager::new();
    
    // 添加一些基础颜色材质
    material_manager.register_material(
        &"grid".to_string(),
        PredefinedMaterial::Color(Color::srgb(0.2, 0.2, 0.2)),
    );
    material_manager.register_material(
        &"red".to_string(),
        PredefinedMaterial::Color(Color::srgb(0.8, 0.2, 0.2)),
    );
    material_manager.register_material(
        &"green".to_string(),
        PredefinedMaterial::Color(Color::srgb(0.2, 0.8, 0.2)),
    );
    material_manager.register_material(
        &"blue".to_string(),
        PredefinedMaterial::Color(Color::srgb(0.2, 0.2, 0.8)),
    );
    material_manager.register_material(
        &"axis_x".to_string(),  // 红色
        PredefinedMaterial::Color(Color::srgb(0.8, 0.2, 0.2)),
    );
    material_manager.register_material(
        &"axis_y".to_string(),  // 绿色
        PredefinedMaterial::Color(Color::srgb(0.2, 0.8, 0.2)),
    );
    material_manager.register_material(
        &"axis_z".to_string(),  // 蓝色
        PredefinedMaterial::Color(Color::srgb(0.2, 0.2, 0.8)),
    );

    // 添加一些特殊材质
    material_manager.register_material(
        &"metal".to_string(),
        PredefinedMaterial::Standard(StandardMaterial {
            base_color: Color::srgb(0.7, 0.8, 0.9),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            ..default()
        })
    );
    
    material_manager.register_material(
        &"glow".to_string(),
        PredefinedMaterial::Standard(StandardMaterial {
            base_color: Color::srgb(0.1, 0.8, 0.5),
            emissive: LinearRgba::from(Color::srgb(0.1, 0.8, 0.5)) * 3.0,
            ..default()
        })
    );
    
    material_manager.register_material(
        &"glass".to_string(),
        PredefinedMaterial::Standard(StandardMaterial {
            base_color: Color::srgba(0.5, 0.8, 0.9, 0.7),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.1,
            metallic: 0.5,
            ..default()
        })
    );

    // 将材质管理器添加到全局资源中
    commands.insert_resource(material_manager);
}