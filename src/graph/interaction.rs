use bevy::prelude::*;

pub mod camera;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        // 在这里注册交互相关的系统
    }
}