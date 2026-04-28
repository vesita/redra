//! 渲染执行层 — Bevy 场景渲染、交互、UI
//!
//! 职责：
//! - 提供渲染服务（网格创建、材质应用、相机控制）
//! - 帧数据到渲染实体的转换
//! - 用户交互与场景初始化

use bevy::prelude::*;

pub mod interaction;
pub mod init;
pub mod frame_renderer;
pub mod scene;
pub mod framerate;
pub mod conversion;
pub mod helpers;

pub use crate::assets::materials::{MaterialManager, GenericMaterial, GenericMaterial3d};

// ==================== Render 插件 ====================

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(scene::SceneInitializerPlugin)
            .add_plugins(init::InitPlugin)
            .add_plugins(interaction::InteractionPlugin)
            .add_plugins(framerate::FrameRatePlugin)
            .add_plugins(frame_renderer::FrameRendererPlugin);
    }
}
