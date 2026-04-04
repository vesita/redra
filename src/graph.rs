pub mod rendering;
pub mod data_processing;
pub mod interaction;
pub mod ui;
pub mod axis;

// 导入材质模块
pub use rendering::material::{MaterialManager, PredefinedMaterial};

// 保留原通信模块
pub mod communicate;

// 保留init模块
pub mod init;

use bevy::prelude::*;
use ui::data_play_control::DataPlayControlPlugin;

// 定义 GraphPlugin 来整合所有图形相关的插件和系统
pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(data_processing::DataProcessingPlugin)
            .add_plugins(rendering::RenderingPlugin)  // 添加RenderingPlugin
            .add_plugins(ui::UiModule)
            .add_plugins(DataPlayControlPlugin);
    }
}