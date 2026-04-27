use bevy::prelude::*;

pub mod camera;
pub mod picking;

pub struct InteractionPlugin;


impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(InteractionMessage::default())
            .add_plugins(camera::CameraInteractionPlugin)
            // 空点击检测：点击空白处时清空选中状态
            .add_systems(Update, picking::detect_empty_click);
    }
}


#[derive(Resource, Default)]
pub struct InteractionMessage { 
    pub selected: Option<u64>,
}