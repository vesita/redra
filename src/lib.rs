use bevy::prelude::*;

// Redra 核心模块 - 按职责分类
// 
// 架构理念：
// - manager: 业务逻辑层，提供高级业务抽象（录制、字体管理等）
// - renderer: 渲染执行层，提供渲染服务 API

pub mod manager;

pub mod renderer;

// 导出子 crate
pub extern crate expto;

// 导出主要插件
pub use redra_plugin::RedraPlugin;

mod redra_plugin {
    use bevy::prelude::*;
    use crate::renderer::RendererPlugin;
    use crate::manager::Manager;
    
    /// 主 Redra 插件，整合控制流、业务逻辑和渲染功能
    pub struct RedraPlugin;
    
    impl Plugin for RedraPlugin {
        fn build(&self, app: &mut App) {
            app
                .add_plugins(RendererPlugin)
                .add_plugins(Manager::default());
        }
    }
}