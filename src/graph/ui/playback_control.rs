use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use crate::graph::action::record::{DataRecorder, PlaybackManager};

/// 回放 UI 插件
pub struct PlaybackUiPlugin;

impl Plugin for PlaybackUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update, 
            playback_ui_system
                .after(crate::graph::ui::font::setup_egui_fonts)
                .run_if(|font_assets: Option<Res<crate::graph::ui::font::FontAssets>>| font_assets.is_some())
        );
    }
}

/// 回放 UI 系统（使用 egui）
pub fn playback_ui_system(
    mut contexts: EguiContexts,
    mut playback: ResMut<PlaybackManager>,
    mut recorder: ResMut<DataRecorder>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // 第一帧跳过，确保 egui 完全初始化
    if time.elapsed_secs() < 0.1 {
        return;
    }
    
    // 处理键盘输入
    handle_keyboard_input(&keyboard_input, &mut playback, &mut recorder);

    // 获取 egui 上下文，如果不可用则跳过
    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };
    
    // 尝试创建窗口，如果字体未准备好可能会 panic，所以用 catch_unwind
    // 注意：catch_unwind 只能捕获 unwind 类型的 panic，且性能开销较小
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        egui::Window::new("Playback Control")
            .fixed_pos(egui::pos2(10.0, 10.0))
            .collapsible(false)
            .show(egui_ctx, |ui| {
                ui.heading("数据回放控制");
                
                ui.separator();
                
                // 状态显示
                ui.label(format!(
                    "录制状态：{}",
                    if recorder.is_recording { "开启" } else { "关闭" }
                ));
                ui.label(format!("总帧数：{}", recorder.total_frames()));
                ui.label(format!("当前帧：{}/{}", playback.current_frame_index, recorder.frames.len().max(1)));
                ui.label(format!("播放速度：{:.1}x", playback.playback_speed));
                
                ui.separator();
                
                // 播放控制按钮
                ui.horizontal(|ui| {
                    if ui.button("▶ 播放").clicked() {
                        playback.play();
                    }
                    if ui.button("⏸ 暂停").clicked() {
                        playback.pause();
                    }
                    if ui.button("⏹ 停止").clicked() {
                        playback.stop();
                    }
                });
                
                ui.horizontal(|ui| {
                    if ui.button("◀◀ 上一帧").clicked() {
                        playback.previous_frame();
                    }
                    if ui.button("▶▶ 下一帧").clicked() {
                        playback.next_frame(recorder.frames.len());
                    }
                });
                
                ui.separator();
                
                // 速度控制
                ui.horizontal(|ui| {
                    if ui.button("-0.5x").clicked() {
                        let new_speed = (playback.playback_speed - 0.5).max(0.1);
                        playback.set_speed(new_speed);
                    }
                    if ui.button("+0.5x").clicked() {
                        let new_speed = playback.playback_speed + 0.5;
                        playback.set_speed(new_speed);
                    }
                    if ui.button("重置速度").clicked() {
                        playback.set_speed(1.0);
                    }
                });
                
                ui.separator();
                
                // 录制控制
                ui.horizontal(|ui| {
                    if ui.button(if recorder.is_recording { "停止录制" } else { "开始录制" }).clicked() {
                        recorder.is_recording = !recorder.is_recording;
                    }
                    if ui.button("清除所有帧").clicked() {
                        recorder.clear();
                        playback.stop();
                    }
                });
                
                ui.separator();
                
                // 快捷键说明
                ui.label("快捷键:");
                ui.label("  空格 - 播放/暂停");
                ui.label("  R - 切换录制");
                ui.label("  C - 清除录制");
                ui.label("  ←/→ - 上一帧/下一帧");
            });
    }));

}

fn handle_keyboard_input(
    keyboard_input: &ButtonInput<KeyCode>,
    playback: &mut PlaybackManager,
    recorder: &mut DataRecorder,
) {
    // 空格键 - 播放/暂停
    if keyboard_input.just_pressed(KeyCode::Space) {
        if playback.is_playing {
            playback.pause();
        } else {
            playback.play();
        }
    }

    // R 键 - 切换录制
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        recorder.is_recording = !recorder.is_recording;
    }

    // C 键 - 清除录制
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        recorder.clear();
        playback.stop();
    }

    // 左箭头 - 上一帧
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        playback.previous_frame();
    }

    // 右箭头 - 下一帧
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        playback.next_frame(recorder.frames.len());
    }
}

// 保持旧的 API 兼容
pub fn spawn_playback_ui(_commands: Commands) {
    // egui 不需要手动 spawn UI
}

pub fn handle_playback_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut playback: ResMut<PlaybackManager>,
    mut recorder: ResMut<DataRecorder>,
) {
    handle_keyboard_input(&keyboard_input, &mut playback, &mut recorder);
}
