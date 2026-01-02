use bevy::prelude::*;

pub mod spawn;
pub mod clear;

// 定义 ActionPlugin 来注册相关系统
pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, clear::clear_all_entities);
    }
}