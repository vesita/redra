use bevy::prelude::*;

pub mod camera;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(camera::CameraInteractionPlugin); // 重新添加相机交互插件
    }
}