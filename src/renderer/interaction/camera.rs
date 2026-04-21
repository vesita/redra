use bevy::prelude::*;
use smooth_bevy_cameras::controllers::fps::FpsCameraPlugin;

pub struct CameraInteractionPlugin;

impl Plugin for CameraInteractionPlugin {
    fn build(&self, app: &mut App) {
        // 添加FPS相机插件以提供相机控制功能
        app.add_plugins(FpsCameraPlugin::default());
    }
}