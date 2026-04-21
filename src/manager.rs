use bevy::app::prelude::*;
use bevy::asset::AssetApp;
use redra_net::NetworkPlugin;

use crate::manager::materials::MaterialManager;

// Manager 模块 - 业务逻辑层
// 提供高级业务抽象（录制、字体管理、协议桥接等）
// 不负责控制流编排，那是 control 模块的职责

// 【数据管理层】- 帧数据管理
pub mod data;

// 【数据流控制】- 网络协议 ↔ 渲染数据的桥接
pub mod data_flow;

// 【交互控制流】- UI 和用户交互
pub mod interaction;

// 【材质管理】- 材质配置与智能选择
pub mod materials;


/// Manager 主插件
/// 
/// 职责：
/// - 提供业务逻辑相关的高级抽象
/// - 注册业务功能插件
/// - 初始化全局资源（MaterialManager, FrameManager等）
/// - 不负责启动/关闭流程编排（由 control 模块负责）
#[derive(Default)]
pub struct Manager;

impl Plugin for Manager {
    fn build(&self, app: &mut App) {
        app
            // // 添加 bevy_materialize 插件（自动注册 GenericMaterial 资产类型和 TOML 加载器）
            // .add_plugins(bevy_materialize::prelude::MaterializePlugin::new(
            //     bevy_materialize::prelude::TomlMaterialDeserializer,
            // ))
            
            // 初始化材质管理器（从 TOML 自动加载）
            .init_resource::<MaterialManager>()
            .add_plugins(NetworkPlugin)
            .add_plugins(interaction::font_manager::FontPlugin)
            // 【数据管理层】- 帧数据管理插件
            .add_plugins(data::frame::FrameManagerPlugin)
            // 【播放控制】- 帧播放控制插件
            .add_plugins(data::frame::FramePlaybackPlugin);
    }
}