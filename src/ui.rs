//! UI 模块 — 用户界面（基于 egui + VS Code 风格布局）
//!
//! 包含：shell 布局、帧回放控制、轮盘菜单、文件管理、标签显示

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub mod playback_control;
pub mod wheel_menu;
pub mod file_manager;
pub mod label;
pub mod theme;
pub mod shell;
pub mod notifications;
pub mod axis_adjust;

#[derive(Component, Resource, Default)]
pub struct UIStates {
    pub show_label_panel: bool,
}

/// UI 主插件
pub struct UiModule;

impl Plugin for UiModule {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default());
        app.init_resource::<UIStates>();
        app
            // VS Code 暗色主题
            .add_plugins(theme::ThemePlugin)
            // Shell 布局（活动栏 + 侧栏 + 状态栏）
            .add_plugins(shell::ShellPlugin)
            // 通知系统（全局 Toast，右上角）
            .add_plugins(notifications::NotificationPlugin)
            // 功能插件（仅注册系统，窗口由 Shell 管理）
            .add_plugins(playback_control::PlaybackUiPlugin)
            .add_plugins(wheel_menu::WheelMenuGraphPlugin)
            .add_plugins(file_manager::FileManagerUiPlugin)
            .add_plugins(label::LabelUiPlugin)
            .add_plugins(axis_adjust::AxisAdjustPlugin);
    }
}
