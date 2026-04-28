//! VS Code 暗色主题 — 全局 egui 风格配置

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_theme);
    }
}

fn setup_theme(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    apply_vscode_theme(&ctx);
}

pub fn apply_vscode_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(egui::Color32::from_rgb(204, 204, 204));

    // VS Code 暗色调色板
    let bg_dark = egui::Color32::from_rgb(30, 30, 30);         // #1e1e1e 编辑器背景
    let bg_medium = egui::Color32::from_rgb(37, 37, 38);        // #252526 侧栏背景
    let bg_light = egui::Color32::from_rgb(51, 51, 51);         // #333333 活动栏背景
    let border = egui::Color32::from_rgb(60, 60, 60);           // #3c3c3c 边框
    let accent = egui::Color32::from_rgb(0, 122, 204);          // #007acc 蓝色强调
    let selection = egui::Color32::from_rgb(38, 79, 120);       // #264f78 选中
    let hover_bg = egui::Color32::from_rgb(42, 45, 46);         // #2a2d2e 悬停
    let text_primary = egui::Color32::from_rgb(204, 204, 204);  // #cccccc

    // 窗口背景
    style.visuals.window_fill = bg_dark;
    style.visuals.panel_fill = bg_medium;
    style.visuals.code_bg_color = egui::Color32::from_rgb(20, 20, 20);
    style.visuals.extreme_bg_color = bg_dark;

    // 窗口边框 + 阴影
    style.visuals.window_stroke = egui::Stroke::new(1.0, border);
    style.visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4].into(),
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(80),
    };
    style.visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 8].into(),
        blur: 24,
        spread: 0,
        color: egui::Color32::from_black_alpha(100),
    };

    // 控件边框
    for w in &mut [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        w.bg_stroke = egui::Stroke::new(1.0, border);
    }

    // 控件填充
    style.visuals.widgets.noninteractive.bg_fill = bg_medium;
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text_primary);
    style.visuals.widgets.inactive.bg_fill = bg_medium;
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_primary);
    style.visuals.widgets.inactive.corner_radius = 4.0.into();
    style.visuals.widgets.hovered.bg_fill = hover_bg;
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    style.visuals.widgets.hovered.corner_radius = 4.0.into();
    style.visuals.widgets.active.bg_fill = selection;
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
    style.visuals.widgets.active.corner_radius = 4.0.into();
    style.visuals.widgets.open.bg_fill = bg_light;
    style.visuals.widgets.open.corner_radius = 4.0.into();

    // 选中
    style.visuals.selection.bg_fill = selection;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, accent);

    // 超链接
    style.visuals.hyperlink_color = accent;

    // 间距
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(8.0, 3.0);
    style.spacing.indent = 16.0;

    // 文字样式
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(11.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(12.0, egui::FontFamily::Monospace),
    );

    ctx.set_style(style);
}
