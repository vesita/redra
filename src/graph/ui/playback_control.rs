use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use crate::manager::record::{RecordingState, PlaybackState};
use crate::manager::font::FontLoadStatus;

/// 回放 UI 插件
pub struct PlaybackUiPlugin;

impl Plugin for PlaybackUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, playback_ui_system.run_if(font_loaded));
    }
}

/// 字体加载状态检查函数
fn font_loaded(font_status: Res<FontLoadStatus>) -> bool {
    *font_status == FontLoadStatus::Loaded
}

/// 回放 UI 系统（使用 egui）
pub fn playback_ui_system(
    mut contexts: EguiContexts,
    cursor_options: bevy::prelude::Single<&bevy::window::CursorOptions>,
    recording_state: ResMut<RecordingState>,
    playback_state: ResMut<PlaybackState>,
) {
    // 如果光标被锁定（FPS模式），不显示UI
    if cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked {
        return;
    }

    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };

    // 主控制面板
    egui::Window::new("录制与回放控制")
        .fixed_pos(egui::pos2(10.0, 10.0))
        .collapsible(false)
        .resizable(true)
        .default_size([350.0, 400.0])
        .show(egui_ctx, |ui| {
            ui.heading("📹 录制控制");
            
            // 录制状态
            let storage_status = if recording_state.is_storage_initialized() {
                "✅ 存储就绪"
            } else {
                "⏳ 未初始化（需要手动调用 initialize）"
            };
            ui.label(storage_status);
            
            let status_text = if recording_state.is_recording() {
                format!("🔴 录制中... ({} 帧)", recording_state.recorded_frames())
            } else {
                "⏹️ 未录制".to_string()
            };
            
            let status_color = if recording_state.is_recording() {
                egui::Color32::RED
            } else {
                egui::Color32::GRAY
            };
            
            ui.colored_label(status_color, status_text);
            
            ui.horizontal(|ui| {
                if !recording_state.is_storage_initialized() {
                    ui.label("⚠️ 存储未初始化");
                } else if recording_state.is_recording() {
                    if ui.button("⏹ 停止录制").clicked() {
                        // 这里需要异步调用，简化处理
                        log::info!("停止录制请求已发送");
                    }
                } else {
                    if ui.button("🔴 开始录制").clicked() {
                        log::info!("开始录制请求已发送");
                    }
                }
            });
            
            ui.separator();
            ui.heading("▶️ 回放控制");
            
            // 回放状态
            let playback_status = match playback_state.as_ref() {
                PlaybackState::Stopped => "⏹️ 已停止",
                PlaybackState::Playing { current_frame, total_frames, .. } => {
                    &format!("▶️ 播放中: {} / {}", current_frame + 1, total_frames)
                },
                PlaybackState::Paused { current_frame, total_frames } => {
                    &format!("⏸️ 已暂停: {} / {}", current_frame + 1, total_frames)
                },
            };
            
            ui.label(playback_status);
            
            ui.horizontal(|ui| {
                match playback_state.as_ref() {
                    PlaybackState::Playing { .. } => {
                        if ui.button("⏸ 暂停").clicked() {
                            log::info!("暂停请求");
                        }
                    },
                    _ => {
                        if ui.button("▶ 播放").clicked() {
                            log::info!("播放请求");
                        }
                    }
                }
                
                if ui.button("⏹ 停止").clicked() {
                    log::info!("停止请求");
                }
            });
            
            ui.separator();
            
            // 快捷键说明
            ui.collapsing("快捷键", |ui| {
                ui.label("空格 - 播放/暂停");
                ui.label("R - 重新开始");
            });
        });
}
