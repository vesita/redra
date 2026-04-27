//! UI 模块 - 提供所有用户界面功能
//!
//! 本模块包含：
//! - bevy_egui 插件初始化
//! - 帧回放控制面板
//! - 轮盘菜单系统
//! - 文件管理（保存/加载）
//! - 标签显示系统
//!
//! # 架构设计
//!
//! ```text
//! ui/
//! ├── mod.rs              // 主入口，初始化 EguiPlugin
//! ├── playback_control.rs // 帧回放控制 UI
//! ├── wheel_menu.rs       // 轮盘菜单 UI
//! ├── file_manager.rs     // 文件管理 UI（保存/加载）
//! └── label.rs            // 标签显示 UI
//! ```

pub mod playback_control;
pub mod wheel_menu;
pub mod file_manager;
pub mod label;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use playback_control::PlaybackUiPlugin;
use wheel_menu::WheelMenuGraphPlugin;
use file_manager::FileManagerUiPlugin;
use label::LabelUiPlugin;

/// UI 主插件
///
/// 职责：
/// - 初始化 bevy_egui 渲染上下文
/// - 注册所有子 UI 插件
/// - 管理全局 UI 配置（如主题、字体等）
pub struct UiModule;

impl Plugin for UiModule {
    fn build(&self, app: &mut App) {
        // 1. 初始化 bevy_egui（必须在所有 UI 插件之前）
        app.add_plugins(EguiPlugin::default());

        // 2. 初始化UI状态资源
        app.init_resource::<UIStates>();

        // 3. 注册子 UI 插件
        app
            .add_plugins(PlaybackUiPlugin)
            .add_plugins(WheelMenuGraphPlugin)
            .add_plugins(FileManagerUiPlugin)
            .add_plugins(LabelUiPlugin);
    }
}


#[derive(Component, Resource, Default)]
pub struct UIStates {
    pub show_label_panel: bool,
}