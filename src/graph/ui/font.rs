//! 统一字体管理模块
//! 
//! 此模块负责加载和管理所有字体资源，包括：
//! - Bevy 原生文本渲染（Text2d, Text3d）
//! - egui UI 框架文本渲染

use std::sync::Arc;
use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// 字体资源，包含 Bevy 和 egui 使用的字体
#[derive(Resource)]
pub struct FontAssets {
    /// Bevy 原生字体句柄
    pub bevy_font: Handle<bevy::text::Font>,
}

impl Default for FontAssets {
    fn default() -> Self {
        Self {
            bevy_font: Handle::default(),
        }
    }
}

/// 字体插件
pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FontAssets>()
            .add_systems(Startup, load_all_fonts)
            .add_systems(Update, setup_egui_fonts);
    }
}

/// 系统：在 Startup 阶段加载所有字体
fn load_all_fonts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("正在加载字体资源...");
    
    // 加载 Bevy 原生字体
    let bevy_font = asset_server.load("fonts/serif/SourceHanSerifCN-VF.otf");
    
    // 插入字体资源
    commands.insert_resource(FontAssets {
        bevy_font,
    });
    
    info!("字体资源加载完成");
}

/// 系统：设置 egui 字体（每帧检查，确保上下文准备好后设置）
fn setup_egui_fonts(
    mut contexts: EguiContexts,
    font_assets: Option<Res<FontAssets>>,
) {
    // 只在字体资源可用时设置
    if font_assets.is_some() {
        let mut fonts = bevy_egui::egui::FontDefinitions::default();
        
        // 从静态文件加载字体数据（用于 egui）
        fonts.font_data.insert(
            "main_font".to_owned(),
            Arc::new(bevy_egui::egui::FontData::from_static(include_bytes!(
                "../../../assets/fonts/serif/SourceHanSerifCN-VF.otf"
            ))),
        );
        
        // 设置为主要比例字体
        fonts
            .families
            .entry(bevy_egui::egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "main_font".to_owned());
        
        // 设置为等宽字体备用
        fonts
            .families
            .entry(bevy_egui::egui::FontFamily::Monospace)
            .or_default()
            .push("main_font".to_owned());
        
        // 使用 ctx_mut() 获取 egui 上下文并设置字体
        if let Ok(ctx) = contexts.ctx_mut() {
            ctx.set_fonts(fonts);
        }
    }
}

/// 辅助函数：获取字体句柄的便捷方法
pub fn get_font_handle(font_assets: &Res<FontAssets>) -> Handle<bevy::text::Font> {
    font_assets.bevy_font.clone()
}