pub mod setup;
pub mod update;
pub mod action;
pub mod axis;
pub mod init;
pub mod communicate;
pub mod ui;

// 导入材质模块
pub mod material;
pub use material::MaterialManager;

use bevy::prelude::*;

// 定义 GraphPlugin 来整合所有图形相关的插件和系统
pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(action::ActionPlugin)
            .add_plugins(ui::UiModule);
    }
}