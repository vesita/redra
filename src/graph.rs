use bevy::prelude::*;

// graph模块入口文件
// 统一导出graph模块的所有公共接口

pub mod rendering;
pub mod interaction;
pub mod ui;
pub mod communicate;
pub mod data_processing;  
pub mod init;
pub mod frame_rate;  // 添加缺失的模块声明
pub mod materials;  // 添加materials模块

// 定义 GraphPlugin 来整合所有图形相关的插件和系统
pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(init::InitPlugin)                 // 初始化插件
            .add_plugins(rendering::RenderingPlugin)       // 渲染插件
            .add_plugins(interaction::InteractionPlugin)   // 交互插件
            .add_plugins(ui::UiModule)                     // UI插件
            .add_plugins(data_processing::DataProcessingPlugin) // 数据处理插件
            .add_plugins(frame_rate::FrameRatePlugin);      // 帧率控制插件
    }
}