pub mod config;
pub mod geometry;
pub mod graph;
pub mod interface;
pub mod manager;

// 导出子 crate
pub extern crate expto;
pub extern crate inpto;  // 导出inpto作为核心协议
pub extern crate parser;
pub extern crate redra_storage;

// 导出主要插件
pub use redra_plugin::RedraPlugin;

mod redra_plugin {
    use bevy::prelude::*;
    use crate::graph::GraphPlugin;
    use crate::manager::Manager;
    
    /// 主 Redra 插件，整合网络、协议和渲染功能
    pub struct RedraPlugin;
    
    impl Plugin for RedraPlugin {
        fn build(&self, app: &mut App) {
            // 添加网络和协议处理插件
            app
                .add_plugins(GraphPlugin)
                .add_plugins(Manager::default());
                // ParserPlugin 已移除，现在由 Manager 统一管理
                // StoragePlugin 已移除，storage 不再是 Bevy 插件
        }
    }
}
