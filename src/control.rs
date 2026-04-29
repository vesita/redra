//! 编排层 — 跨模块的协调和流程控制
//!
//! 职责（逐步完善）：
//! - 数据 → 渲染 的管线编排
//! - 应用状态机（录制/回放/空闲等）
//! - 模块之间的消息路由
//!
//! 当前从 Manager 插件演进而来

use bevy::app::prelude::*;
use redra_net::NetworkPlugin;
use redra_calib::prelude::CalibPlugin;

use crate::data::frame::{FrameManagerPlugin, FramePlaybackPlugin, FrameStoragePlugin};
use crate::assets::fonts::FontPlugin;
use crate::assets::materials::MaterialManager;
use crate::render::RenderPlugin;
use crate::ui::UiModule;

/// 应用编排插件 — 按依赖顺序注册所有子插件
#[derive(Default)]
pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app
            // 资源层
            .init_resource::<MaterialManager>()
            .add_plugins(FontPlugin)
            .add_plugins(CalibPlugin)
            // 数据层
            .add_plugins(NetworkPlugin)
            .add_plugins(FrameManagerPlugin)
            .add_plugins(FrameStoragePlugin)
            .add_plugins(FramePlaybackPlugin)
            // 渲染层
            .add_plugins(RenderPlugin)
            // UI 层（需在渲染之后，提供 EguiContexts 等资源）
            .add_plugins(UiModule);
    }
}
