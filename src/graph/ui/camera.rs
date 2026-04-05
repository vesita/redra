use bevy::prelude::*;

/// UI相机插件，用于处理UI渲染
pub struct UiCameraPlugin;

impl Plugin for UiCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui_camera)
            .add_systems(Update, update_camera_priority);
    }
}

/// 设置UI相机
fn setup_ui_camera(mut commands: Commands) {
    // 创建一个专门用于UI渲染的相机
    commands.spawn((
        Camera2d,
        Camera {
            order: 10, // 设置较高层级，确保UI渲染在最上层
            ..default()
        },
        UiCamera,
    ));
}

/// UI相机标记组件
#[derive(Component)]
struct UiCamera;

/// 更新相机优先级，确保UI相机始终在3D相机之上
fn update_camera_priority(
    mut cameras: Query<(&mut Camera, &mut Transform), With<UiCamera>>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    for (mut camera, mut transform) in cameras.iter_mut() {
        // 确保UI相机位于场景的合适位置
        transform.translation.z = 999.0; // 放在所有3D对象之上
    }
}