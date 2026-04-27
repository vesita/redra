//! 通知系统 — 右下角水杯 + Toast 弹出消息
//!
//! 水杯隐喻：消息如水源源不断流入杯中，展开消息记录即「喝水」（已读），
//! 水位归零。历史消息由队列自动管理（FIFO，最多 20 条），无需手动清理。

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

/// 通知容量上限（也是水杯满水对应的数量）
const MAX_TOASTS: usize = 20;

/// 水杯尺寸
const CUP_SIZE: (f32, f32) = (44.0, 44.0);

// ============================================================================
// 通知中心
// ============================================================================

#[derive(Clone)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub is_error: bool,
    pub created_at: Instant,
}

#[derive(Resource)]
pub struct NotificationCenter {
    toasts: Vec<Toast>,
    next_id: u64,
    pub expanded: bool,
    /// 未读数，即「杯中的水」，展开面板（喝水）后归零
    unread: usize,
}

impl Default for NotificationCenter {
    fn default() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
            expanded: false,
            unread: 0,
        }
    }
}

impl NotificationCenter {
    /// 入队一条消息。「水源」—— 未读数 +1，水杯水量上涨。
    /// 超过容量时自动淘汰最旧的。记录永远保留在历史中。
    pub fn notify(&mut self, message: impl Into<String>, is_error: bool) {
        let toast = Toast {
            id: self.next_id,
            message: message.into(),
            is_error,
            created_at: Instant::now(),
        };
        self.next_id += 1;
        self.unread += 1;
        self.toasts.push(toast);
        if self.toasts.len() > MAX_TOASTS {
            self.toasts.remove(0);
        }
    }

    pub fn dismiss(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }

    /// 4s 内的活跃 Toast（用于弹窗显示）
    pub fn active(&self) -> Vec<&Toast> {
        let now = Instant::now();
        self.toasts
            .iter()
            .filter(|t| now.duration_since(t.created_at) < Duration::from_secs(4))
            .collect()
    }

    /// 全部历史消息（先进先出队列，最多 MAX_TOASTS 条）。记录永久保留不消失。
    pub fn all(&self) -> &[Toast] {
        &self.toasts
    }

    /// 「喝水」—— 标记所有消息为已读，水位归零
    pub fn drink(&mut self) {
        self.unread = 0;
    }
}

// ============================================================================
// 插件
// ============================================================================

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationCenter>()
            .add_systems(EguiPrimaryContextPass, combined_ui_system);
    }
}

// ============================================================================
// 水杯 + Toast 合并渲染（共用一个 egui::Area，避免首帧 available_rect 问题）
// ============================================================================

/// 解码水杯 PNG 为 egui ColorImage
fn decode_cup_png() -> egui::ColorImage {
    let img = image::load_from_memory(include_bytes!("../../assets/icon/水杯.png"))
        .expect("水杯.png")
        .into_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw())
}

fn combined_ui_system(
    mut contexts: EguiContexts,
    cursor_options: bevy::prelude::Single<&bevy::window::CursorOptions>,
    mut center: ResMut<NotificationCenter>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // 手动解码 PNG → egui 纹理（egui 没有内置 PNG 解码器）
    static CUP_TEX: std::sync::OnceLock<(egui::TextureId, egui::Vec2)> = std::sync::OnceLock::new();
    let (cup_tex, tex_size) = *CUP_TEX.get_or_init(|| {
        let color_image = decode_cup_png();
        let size = egui::vec2(color_image.width() as f32, color_image.height() as f32);
        let id = ctx.load_texture("水杯", color_image, egui::TextureOptions::default()).id();
        (id, size)
    });
    let sized_tex = egui::load::SizedTexture { id: cup_tex, size: tex_size };
    let cup_image = egui::Image::from_texture(sized_tex);
    let total = center.unread;
    let ratio = (total as f32 / MAX_TOASTS as f32).clamp(0.0, 1.0);
    let cursor_locked = cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked;
    let active_clone: Vec<Toast> = center.active().into_iter().cloned().collect();

    egui::Area::new(egui::Id::new("notification_area"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                // bottom_up 布局：先添加的 widget 在底部 → 水杯先画（最底部）
                // ── 水杯（永远在最底部） ────────────────
                let btn_size = egui::vec2(CUP_SIZE.0, CUP_SIZE.1);
                let response = ui.add_sized(
                    btn_size,
                    egui::Button::new("")
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE),
                );

                let rect = response.rect;
                if ui.is_rect_visible(rect) {
                    let painter = ui.painter();

                    painter.rect_filled(
                        rect,
                        egui::CornerRadius::same(6),
                        egui::Color32::from_rgba_premultiplied(30, 30, 35, 120),
                    );

                    // 水量填充
                    if ratio > 0.02 {
                        let inset_x = 5.0;
                        let inset_bottom = 5.0;
                        let inset_top = 10.0;
                        let fill_h = (btn_size.y - inset_top - inset_bottom) * ratio;
                        let water_rect = egui::Rect::from_min_max(
                            egui::pos2(rect.min.x + inset_x, rect.max.y - inset_bottom - fill_h),
                            egui::pos2(rect.max.x - inset_x, rect.max.y - inset_bottom),
                        );
                        if water_rect.top() < water_rect.bottom() {
                            painter.rect_filled(
                                water_rect,
                                egui::CornerRadius::same(2),
                                egui::Color32::from_rgba_premultiplied(14, 233, 228, 150),
                            );
                        }
                    }

                    // PNG 杯体叠加
                    cup_image.clone().paint_at(
                        ui,
                        egui::Rect::from_center_size(rect.center(), btn_size - egui::vec2(2.0, 2.0)),
                    );
                }

                if response.clicked() {
                    center.expanded = !center.expanded;
                    if center.expanded {
                        center.drink();
                    }
                }

                // ── 活跃 Toast（水杯上方） ──────────────
                for toast in active_clone.iter().rev() {
                    let alpha = toast_alpha(toast.created_at);
                    if alpha <= 0.01 {
                        continue;
                    }
                    let drop_y = toast_drop_offset(toast.created_at);
                    show_toast(ui, toast, alpha, drop_y, cursor_locked, &mut center);
                }

                // ── 历史面板（再往上） ──────────
                if center.expanded {
                    show_history(ui, &mut center, cursor_locked);
                    ui.add_space(4.0);
                }
            });
        });
}

/// 透明度：3s 内不透明，3~4s 淡出
fn toast_alpha(created: Instant) -> f32 {
    let elapsed = created.elapsed().as_secs_f32();
    if elapsed < 3.0 {
        1.0
    } else if elapsed < 4.0 {
        (1.0 - (elapsed - 3.0)).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// 下落偏移：从上方 15px 弹入，0.35s 内落到原位
fn toast_drop_offset(created: Instant) -> f32 {
    let elapsed = created.elapsed().as_secs_f32();
    if elapsed < 0.35 {
        (1.0 - elapsed / 0.35) * -15.0
    } else {
        0.0
    }
}

/// 单条 Toast 卡片
fn show_toast(
    ui: &mut egui::Ui,
    toast: &Toast,
    alpha: f32,
    drop_y: f32,
    cursor_locked: bool,
    center: &mut NotificationCenter,
) {
    let a8 = (alpha * 255.0) as u8;
    let a32 = a8 as u32;
    let bg = if toast.is_error {
        egui::Color32::from_rgba_premultiplied(55, 18, 18, (200u32 * a32 / 255).min(255) as u8)
    } else {
        egui::Color32::from_rgba_premultiplied(18, 45, 18, (200u32 * a32 / 255).min(255) as u8)
    };
    let border =
        egui::Color32::from_rgba_premultiplied(90, 90, 90, (150u32 * a32 / 255).min(255) as u8);
    let text_color = if toast.is_error {
        egui::Color32::from_rgba_premultiplied(255, 130, 130, a8)
    } else {
        egui::Color32::from_rgba_premultiplied(130, 255, 130, a8)
    };
    let icon = if toast.is_error { "✗" } else { "✓" };

    let margin = if drop_y < 0.0 {
        egui::Margin {
            left: 10,
            right: 10,
            top: (drop_y as i8).max(-50),
            bottom: 6,
        }
    } else {
        egui::Margin::symmetric(10, 6)
    };

    egui::Frame::NONE
        .fill(bg)
        .stroke(egui::Stroke::new(1.0, border))
        .corner_radius(6)
        .inner_margin(margin)
        .show(ui, |ui| {
            ui.set_max_width(300.0);
            ui.horizontal(|ui| {
                ui.colored_label(text_color, format!("{} {}", icon, toast.message));
                if !cursor_locked {
                    if ui
                        .add(egui::Button::new("×").min_size(egui::vec2(16.0, 16.0)))
                        .clicked()
                    {
                        center.dismiss(toast.id);
                    }
                }
            });
        });
    ui.add_space(4.0);
}

/// 历史面板
fn show_history(ui: &mut egui::Ui, center: &mut NotificationCenter, cursor_locked: bool) {
    egui::Frame::NONE
        .fill(egui::Color32::from_rgba_premultiplied(22, 22, 28, 220))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 70)))
        .corner_radius(6)
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.set_max_width(300.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("通知历史")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(180, 180, 180)),
                );
                if !cursor_locked {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new("收起").min_size(egui::vec2(36.0, 18.0)))
                            .clicked()
                        {
                            center.expanded = false;
                        }
                    });
                }
            });
            ui.separator();

            let now = Instant::now();
            for toast in center.all().iter().rev() {
                let ago = now.duration_since(toast.created_at).as_secs();
                let color = if toast.is_error {
                    egui::Color32::from_rgb(220, 120, 120)
                } else {
                    egui::Color32::from_rgb(120, 220, 120)
                };
                ui.horizontal(|ui| {
                    ui.colored_label(
                        color,
                        format!("{} {}", if toast.is_error { "✗" } else { "✓" }, toast.message),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}s前", ago))
                                .color(egui::Color32::from_rgb(120, 120, 120))
                                .size(10.0),
                        );
                    });
                });
            }
        });
}
