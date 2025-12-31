use bevy::prelude::*;

use crate::{module::{camera::fps::*}, graph::axis};

pub fn rd_setup (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 添加环境光
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
        ..default()
    });

    // 添加FPS相机控制器
    commands
        .spawn((
            Camera3d::default(),
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