//! VS Code 风格的 shell 布局 — 活动栏 + 侧栏

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::frame::{FrameManager, PlaybackState, FrameStorage};
use crate::ui::file_manager::{FileSaveState, files_content};
use crate::ui::playback_control::{playback_content, ResetCameraView};
use crate::ui::axis_adjust::axis_adjust_content;
use crate::render::coord_system::CoordSystem;
use crate::ui::notifications::NotificationCenter;
use crate::assets::fonts::FontLoadStatus;
use crate::render::init::LightMode;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum SidebarView {
    #[default]
    Playback,
    Files,
    AxisAdjust,
}

#[derive(Resource, Default)]
pub struct SidebarState {
    pub active_view: SidebarView,
    pub visible: bool,
}

pub struct ShellPlugin;

impl Plugin for ShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SidebarState>()
            .add_systems(EguiPrimaryContextPass, shell_system.run_if(font_loaded))
            .add_systems(Update, toggle_sidebar_shortcut);
    }
}

fn font_loaded(font_status: Res<FontLoadStatus>) -> bool {
    *font_status == FontLoadStatus::Loaded
}

fn toggle_sidebar_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sidebar: ResMut<SidebarState>,
) {
    if keyboard.just_released(KeyCode::AltLeft) || keyboard.just_released(KeyCode::AltRight) {
        sidebar.visible = !sidebar.visible;
    }
}

/// 活动栏颜色
const ACTIVITY_BG: egui::Color32 = egui::Color32::from_rgb(51, 51, 51);

/// 侧栏背景色
const SIDEBAR_BG: egui::Color32 = egui::Color32::from_rgb(37, 37, 38);

fn shell_system(
    mut contexts: EguiContexts,
    cursor_options: bevy::prelude::Single<&bevy::window::CursorOptions>,
    mut sidebar: ResMut<SidebarState>,
    mut frame_manager: ResMut<FrameManager>,
    mut playback_state: ResMut<PlaybackState>,
    mut save_state: ResMut<FileSaveState>,
    storage: Res<FrameStorage>,
    mut notifications: ResMut<NotificationCenter>,
    mut coord: ResMut<CoordSystem>,
    mut reset_camera: ResMut<ResetCameraView>,
    mut light_mode: ResMut<LightMode>,
) {
    if cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // ── 活动栏（窄图标条） ──────────────────────────────
    egui::SidePanel::left("activity_bar")
        .resizable(false)
        .exact_width(48.0)
        .frame(egui::Frame {
            fill: ACTIVITY_BG,
            inner_margin: egui::Margin::symmetric(4, 8),
            ..default()
        })
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);

                let btn_size = egui::vec2(36.0, 36.0);

                // 回放
                let pb = sidebar.active_view == SidebarView::Playback;
                if ui
                    .add(icon_button("▶", pb, btn_size))
                    .on_hover_text("回放控制")
                    .clicked()
                {
                    sidebar.active_view = SidebarView::Playback;
                    sidebar.visible = true;
                }

                ui.add_space(4.0);

                // 文件
                let fl = sidebar.active_view == SidebarView::Files;
                if ui
                    .add(icon_button("🗄", fl, btn_size))
                    .on_hover_text("文件管理")
                    .clicked()
                {
                    sidebar.active_view = SidebarView::Files;
                    sidebar.visible = true;
                }

                ui.add_space(4.0);

                // 坐标系
                let aa = sidebar.active_view == SidebarView::AxisAdjust;
                if ui
                    .add(icon_button("↕", aa, btn_size))
                    .on_hover_text("坐标系")
                    .clicked()
                {
                    sidebar.active_view = SidebarView::AxisAdjust;
                    sidebar.visible = true;
                }

                ui.add_space(4.0);

                ui.separator();

                // 面向世界中心（底部）
                if ui
                    .add(icon_button("◎", false, btn_size))
                    .on_hover_text("面向世界中心")
                    .clicked()
                {
                    reset_camera.0 = true;
                }

                ui.add_space(4.0);

                // 环境光切换（底部）
                let (icon, hover) = match *light_mode {
                    LightMode::Light => ("☀", "明亮模式 (点击切换暗色)"),
                    LightMode::Dark => ("🌙", "暗色模式 (点击切换明亮)"),
                };
                if ui
                    .add(icon_button(icon, false, btn_size))
                    .on_hover_text(hover)
                    .clicked()
                {
                    *light_mode = match *light_mode {
                        LightMode::Light => LightMode::Dark,
                        LightMode::Dark => LightMode::Light,
                    };
                }
            });
        });

    // ── 侧栏（内容面板） ────────────────────────────────
    if sidebar.visible {
        egui::SidePanel::left("side_panel")
            .resizable(true)
            .default_width(280.0)
            .min_width(200.0)
            .max_width(400.0)
            .frame(egui::Frame {
                fill: SIDEBAR_BG,
                inner_margin: egui::Margin::symmetric(12, 8),
                ..default()
            })
            .show(ctx, |ui| {
                // 标题栏
                let header = match sidebar.active_view {
                    SidebarView::Playback => "回放控制",
                    SidebarView::Files => "文件管理",
                    SidebarView::AxisAdjust => "坐标系",
                };
                ui.horizontal(|ui| {
                    ui.heading(header);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new("✕").fill(egui::Color32::TRANSPARENT).min_size(egui::vec2(24.0, 24.0)))
                            .clicked()
                        {
                            sidebar.visible = false;
                        }
                    });
                });
                ui.separator();
                ui.add_space(4.0);

                // 滚动内容
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match sidebar.active_view {
                            SidebarView::Playback => {
                                playback_content(ui, &mut frame_manager, &mut playback_state);
                            }
                            SidebarView::Files => {
                                files_content(ui, &frame_manager, &storage, &mut save_state, &mut notifications);
                            }
                            SidebarView::AxisAdjust => {
                                axis_adjust_content(ui, &mut coord);
                            }
                        }
                    });
            });
    }
}

/// 活动栏图标按钮
fn icon_button<'a>(text: &'a str, selected: bool, size: egui::Vec2) -> egui::Button<'a> {
    let mut btn = egui::Button::new(egui::RichText::new(text).size(18.0));
    btn = btn.min_size(size);
    if selected {
        btn = btn.fill(egui::Color32::from_rgb(38, 79, 120));
        btn = btn.stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 122, 204)));
    } else {
        btn = btn.fill(egui::Color32::TRANSPARENT);
        btn = btn.stroke(egui::Stroke::NONE);
    }
    btn.corner_radius(6)
}
