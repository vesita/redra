use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use crate::manager::data::frame::{FrameManager, PlaybackState};
use crate::manager::interaction::font_manager::FontLoadStatus;

/// 帧回放控制 UI 插件
///
/// 注册以下系统：
/// - `playback_ui_system`: 在 EguiPrimaryContextPass 阶段渲染 UI
/// - `keyboard_shortcuts`: 在 Update 阶段处理键盘输入
pub struct PlaybackUiPlugin;

impl Plugin for PlaybackUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, (
            playback_ui_system.run_if(font_loaded),
        ))
        .add_systems(Update, keyboard_shortcuts);
    }
}

/// 字体加载状态检查
fn font_loaded(font_status: Res<FontLoadStatus>) -> bool {
    *font_status == FontLoadStatus::Loaded
}

/// 键盘快捷键处理系统
///
/// 支持的快捷键：
/// - `Space`: 播放/暂停
/// - `ArrowLeft`: 上一帧
/// - `ArrowRight`: 下一帧
/// - `Home`: 跳转到首帧
/// - `End`: 跳转到尾帧
fn keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut frame_manager: ResMut<FrameManager>,
    mut playback_state: ResMut<PlaybackState>,
) {
    let total_frames = frame_manager.total_frames();
    
    // 没有数据时，只允许播放/暂停切换（虽然不会有实际效果）
    if total_frames == 0 {
        // 空格键 - 播放/暂停（即使没有数据也允许切换状态）
        if keyboard.just_pressed(KeyCode::Space) {
            playback_state.toggle();
            if playback_state.is_playing {
                log::warn!("没有帧数据，无法播放");
            }
        }
        return; // 没有其他可操作的快捷键
    }
    
    // 有空键 - 播放/暂停
    if keyboard.just_pressed(KeyCode::Space) {
        playback_state.toggle();
    }
    
    // 左箭头 - 上一帧
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        if frame_manager.prev_frame() {
            log::info!("上一帧: {}", frame_manager.current_frame_index() + 1);
        }
    }
    
    // 右箭头 - 下一帧
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        if frame_manager.next_frame() {
            log::info!("下一帧: {}", frame_manager.current_frame_index() + 1);
        }
    }
    
    // Home - 跳转到首帧
    if keyboard.just_pressed(KeyCode::Home) {
        frame_manager.seek_to_frame(0);
        log::info!("跳转到第 1 帧");
    }
    
    // End - 跳转到尾帧
    if keyboard.just_pressed(KeyCode::End) {
        frame_manager.seek_to_frame(total_frames - 1);
        log::info!("跳转到第 {} 帧", total_frames);
    }
}

/// 回放 UI 系统（使用 egui）
pub fn playback_ui_system(
    mut contexts: EguiContexts,
    cursor_options: bevy::prelude::Single<&bevy::window::CursorOptions>,
    mut frame_manager: ResMut<FrameManager>,
    mut playback_state: ResMut<PlaybackState>,
) {
    // 如果光标被锁定（FPS模式），不显示UI
    if cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked {
        return;
    }

    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };

    let total_frames = frame_manager.total_frames();
    let current_frame = frame_manager.current_frame_index();
    let has_data = total_frames > 0;

    // 主控制面板 - 使用标准窗口配置
    egui::Window::new("帧回放控制")
        .fixed_pos(egui::pos2(10.0, 10.0))
        .collapsible(true)
        .resizable(false)
        .default_size([320.0, 280.0])
        .show(egui_ctx, |ui| {
            ui.set_max_width(300.0);
            
            // 数据状态提示
            if !has_data {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 200, 100),
                    "等待数据..."
                );
                ui.label("当前没有接收到帧数据");
                ui.label("请检查网络连接或数据源");
                
                ui.separator();
                
                ui.collapsing("快捷键", |ui| {
                    ui.label("空格 - 播放/暂停");
                    ui.label("左/右箭头 - 上一帧/下一帧");
                    ui.label("Home/End - 首帧/尾帧");
                    ui.label("Alt - 显示/隐藏 UI");
                });
                
                return;
            }
            
            // 有数据时的正常显示
            ui.horizontal(|ui| {
                ui.label("当前帧:");
                ui.colored_label(
                    egui::Color32::from_rgb(100, 200, 255),
                    format!("{}/{}", current_frame + 1, total_frames)
                );
            });
            
            ui.separator();
            
            // 播放控制按钮 - 直接使用 enabled 属性
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                
                // 跳转到首帧
                let can_jump_to_start = current_frame > 0;
                if ui.add_enabled(can_jump_to_start, egui::Button::new("|<")).clicked() {
                    frame_manager.seek_to_frame(0);
                    log::info!("跳转到第 1 帧");
                }
                
                // 上一帧
                let can_prev = current_frame > 0;
                if ui.add_enabled(can_prev, egui::Button::new("<")).clicked() {
                    if frame_manager.prev_frame() {
                        log::info!("上一帧: {}", frame_manager.current_frame_index() + 1);
                    }
                }
                
                // 播放/暂停
                let play_button_text = if playback_state.is_playing { "||" } else { ">" };
                if ui.button(play_button_text).clicked() {
                    playback_state.toggle();
                }
                
                // 下一帧
                let can_next = current_frame < total_frames - 1;
                if ui.add_enabled(can_next, egui::Button::new(">")).clicked() {
                    if frame_manager.next_frame() {
                        log::info!("下一帧: {}", frame_manager.current_frame_index() + 1);
                    }
                }
                
                // 跳转到尾帧
                let can_jump_to_end = current_frame < total_frames - 1;
                if ui.add_enabled(can_jump_to_end, egui::Button::new(">|")).clicked() {
                    frame_manager.seek_to_frame(total_frames - 1);
                    log::info!("跳转到第 {} 帧", total_frames);
                }
            });
            
            ui.separator();
            
            // 播放速度控制
            ui.horizontal(|ui| {
                ui.label("播放速度:");
                
                let speed_options = [10.0, 30.0, 60.0, 120.0];
                let current_speed = playback_state.playback_speed;
                
                for &speed in &speed_options {
                    let selected = (current_speed - speed).abs() < 0.1;
                    if ui.selectable_label(selected, format!("{:.1}x", speed / 30.0)).clicked() {
                        playback_state.set_speed(speed);
                    }
                }
            });
            
            // 自定义速度滑块
            ui.horizontal(|ui| {
                ui.label("自定义:");
                let mut speed = playback_state.playback_speed;
                let slider = egui::Slider::new(&mut speed, 1.0..=240.0).text("FPS");
                if ui.add(slider).changed() {
                    playback_state.set_speed(speed);
                }
            });
            
            ui.separator();
            
            // 帧跳转滑块
            ui.horizontal(|ui| {
                ui.label("跳转:");
                let mut frame_idx = current_frame as f32;
                let slider = egui::Slider::new(&mut frame_idx, 0.0..=(total_frames - 1) as f32)
                    .text("帧索引");
                if ui.add(slider).changed() {
                    frame_manager.seek_to_frame(frame_idx as usize);
                    log::info!("跳转到第 {} 帧", frame_idx as usize + 1);
                }
            });
            
            ui.separator();
            
            // 快捷键说明
            ui.collapsing("快捷键", |ui| {
                ui.label("空格 - 播放/暂停");
                ui.label("左/右箭头 - 上一帧/下一帧");
                ui.label("Home/End - 首帧/尾帧");
                ui.label("Alt - 显示/隐藏 UI");
            });
        });
}
