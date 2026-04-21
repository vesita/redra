use bevy::prelude::*;
use bevy_materialize::prelude::*;
use smooth_bevy_cameras::controllers::fps::{FpsCameraBundle, FpsCameraController};

// pub mod axis;

// 初始化插件
pub struct InitPlugin;

impl Plugin for InitPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MaterializePlugin::new(TomlMaterialDeserializer))
            .add_systems(Startup, rd_setup);
            // .add_systems(Startup, axis::axis_setup);
            // 移除了 toggle_camera_control，因为 FPS 控制器已有内置的 Alt 键处理
    }
}

pub fn rd_setup(
    mut commands: Commands,
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

    // 注意：坐标轴由 graph/rendering/axis.rs 中的 AxisRenderingPlugin 自动生成
    // 此处不再手动调用 spawn_axis_setup，避免重复生成

}