use bevy::prelude::*;
use smooth_bevy_cameras::controllers::fps::{FpsCameraBundle, FpsCameraController};

// 相机控制相关的系统和资源
pub fn setup_camera(
    mut commands: Commands,
) {
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
}