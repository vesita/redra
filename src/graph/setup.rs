use bevy::prelude::*;

use crate::{module::{camera::fps::*}, graph::axis};

pub fn rd_setup (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut global_ambient: ResMut<GlobalAmbientLight>,  // 添加全局环境光资源
) {

    // 注意：Bevy 0.18 中环境光已默认集成到 PBR 渲染管线中
    // 无需手动添加 AmbientLight，可通过修改 World Environment 或使用 HDR 贴图来控制全局光照
    // 当前默认环境光强度和颜色由渲染器自动管理

    // let cascade_shadow_config = CascadeShadowConfigBuilder {
    //     first_cascade_far_bound: 0.3,
    //     maximum_distance: 3.0,
    //     ..default()
    // }
    // .build();


    // commands.spawn((
    //     DirectionalLight {
    //         color: Color::srgb(0.98, 0.95, 0.82),
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
    //     cascade_shadow_config,
    // ));


    // // 创建相机并附加天空盒 (设置较低的渲染顺序，作为背景)
    // commands.spawn((
    //     Transform::from_xyz(-1.7, 1.5, 4.5)
    //         .looking_at(Vec3::new(-1.5, 1.7, 3.5), Vec3::Y),
    //     Skybox {
    //         image: asset_server.load("半空中_textures/半空中.mat.meta"), // 加载PlasmaSky纹理
    //         brightness: 1000.0,  // HDR亮度缩放
    //         rotation: Quat::IDENTITY,  // 初始旋转
    //         ..default()
    //     },
    // ));

    // 设置全局环境光
    *global_ambient = GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,  // 调整亮度值
        affects_lightmapped_meshes: true,
    };


    // 添加FPS相机控制器 (设置较高的渲染顺序，作为主相机)
    commands
        .spawn((
            Camera3d::default(),
            Camera {
                order: 1,  // 设置唯一的渲染顺序，作为主相机
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
    axis::spawn_axis(&mut commands, &mut meshes, &mut materials, 3.0);
}