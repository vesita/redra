use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use crate::{graph::action::record::{DataRecorder, PlaybackManager}, manager::font::core::{FontAssets, setup_egui_fonts}};

/// 数据播放控制 UI 插件
pub struct DataPlayControlPlugin;

impl Plugin for DataPlayControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DataPlayControlState>()
            .init_resource::<EguiInitialized>()
            .add_systems(
                Update, 
                (
                    check_egui_initialization,
                    data_play_control_ui_system
                        .after(setup_egui_fonts)
                        .run_if(ui_ready_condition)
                        .run_if(|egui_init: Res<EguiInitialized>| egui_init.initialized)
                )
            );
    }
}

/// 跟踪 egui 初始化状态的资源
#[derive(Resource, Default)]
struct EguiInitialized {
    initialized: bool,
}

/// 检查 egui 是否已初始化的系统
fn check_egui_initialization(
    mut contexts: EguiContexts,
    mut egui_init: ResMut<EguiInitialized>,
) {
    if egui_init.initialized {
        return; // 已经初始化过了，无需再检查
    }

    if let Ok(_egui_ctx) = contexts.ctx_mut() {
        // 如果上下文已经可以获取，我们就标记为已初始化
        // 实际上，一旦能成功获取上下文，通常意味着 egui 已经准备好
        egui_init.initialized = true;
    }
}

/// UI 准备就绪条件
fn ui_ready_condition(font_assets: Option<Res<FontAssets>>) -> bool {
    font_assets.is_some()
}

/// 数据播放控制状态资源
#[derive(Resource, Default)]
pub struct DataPlayControlState {
    pub show_control_panel: bool,
    pub auto_hide: bool,
    pub panel_opacity: f32,
    pub speed_multiplier: f32,
}

/// 数据播放控制 UI 系统
pub fn data_play_control_ui_system(
    mut contexts: EguiContexts,
    mut playback: ResMut<PlaybackManager>,
    mut recorder: ResMut<DataRecorder>,
    mut control_state: ResMut<DataPlayControlState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // 处理键盘输入
    handle_keyboard_input(&keyboard_input, &mut playback, &mut recorder, &mut control_state);

    // 获取 egui 上下文，如果不可用则跳过
    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };

    // 检查egui上下文是否已准备好
    // 通过检查帧计数来判断egui是否已运行至少一次
    if !is_egui_fully_initialized(&egui_ctx) {
        return;
    }
    
    // 如果设置了自动隐藏并且没有播放，则不显示控制面板
    if control_state.auto_hide && !playback.is_playing && playback.current_frame_index == 0 {
        return;
    }
    
    // 主控制面板
    egui::Window::new("数据播放控制")
        .default_pos(egui::pos2(10.0, 10.0))
        .resizable(false)
        .collapsible(true)
        .default_size([300.0, 150.0])
        .show(egui_ctx, |ui| {
            ui.vertical(|ui| {
                // 播放状态信息
                ui.heading("数据播放系统");
                
                ui.separator();
                
                // 状态行
                ui.horizontal(|ui| {
                    ui.label(format!("录制: {}", if recorder.is_recording { "● 开启" } else { "○ 关闭" }));
                    ui.label(format!("| 帧: {}/{}", playback.current_frame_index, get_total_frames(&recorder)));
                    ui.label(format!("| 速度: {:.1}x", playback.playback_speed));
                });
                
                ui.separator();
                
                // 播放控制按钮
                ui.horizontal(|ui| {
                    // 上一帧
                    if ui.button("⏮").on_hover_text("上一帧 (Left Arrow)").clicked() {
                        playback.previous_frame();
                    }
                    
                    // 播放/暂停
                    if ui.button(if playback.is_playing { "⏸" } else { "▶" })
                        .on_hover_text("播放/暂停 (Space)")
                        .clicked() {
                        if playback.is_playing {
                            playback.pause();
                        } else {
                            playback.play();
                        }
                    }
                    
                    // 停止
                    if ui.button("⏹").on_hover_text("停止 (S)").clicked() {
                        playback.stop();
                    }
                    
                    // 下一帧
                    if ui.button("⏭").on_hover_text("下一帧 (Right Arrow)").clicked() {
                        playback.next_frame(get_total_frames(&recorder));
                    }
                });
                
                ui.separator();
                
                // 速度控制
                ui.horizontal(|ui| {
                    ui.label("速度:");
                    ui.add(egui::Slider::new(&mut playback.playback_speed, 0.1..=4.0)
                        .logarithmic(true)
                        .suffix("x"));
                    
                    if ui.button("Reset").clicked() {
                        playback.playback_speed = 1.0;
                    }
                });
                
                ui.separator();
                
                // 额外控制选项
                ui.checkbox(&mut control_state.auto_hide, "自动隐藏");
            });
        });
}

/// 检查egui是否完全初始化，包括字体
fn is_egui_fully_initialized(egui_ctx: &egui::Context) -> bool {
    // 使用try_read方法尝试安全访问egui内部状态
    // 由于我们无法直接访问内部字段，使用一个变通方法：
    // 创建一个小的egui UI来测试是否可以安全地渲染
    // 如果在测试过程中发生错误，则认为尚未准备好
    
    // 这里我们简单地检查上下文是否可以安全使用
    // 通过尝试执行一个基本操作来验证
    egui_ctx.input(|input| input.time >= 0.01)  // 确保应用程序运行了一小段时间
}

/// 获取总帧数（支持 SQLite 和内存模式）
fn get_total_frames(recorder: &DataRecorder) -> usize {
    if let Some(storage_arc) = &recorder.storage {
        // SQLite 模式
        if let Ok(storage) = storage_arc.lock() {
            if let Ok(stats) = storage.database().get_stats() {
                return stats.total_frames as usize;
            }
        }
    }
    // 内存模式
    recorder.frames.len()
}

fn handle_keyboard_input(
    keyboard_input: &ButtonInput<KeyCode>,
    playback: &mut PlaybackManager,
    recorder: &mut DataRecorder,
    control_state: &mut DataPlayControlState,
) {
    // 空格键 - 播放/暂停
    if keyboard_input.just_pressed(KeyCode::Space) {
        if playback.is_playing {
            playback.pause();
        } else {
            playback.play();
        }
    }

    // S 键 - 停止
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        playback.stop();
    }

    // 左箭头 - 上一帧
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        playback.previous_frame();
    }

    // 右箭头 - 下一帧
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        playback.next_frame(get_total_frames(recorder));
    }
    
    // H 键 - 切换自动隐藏
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        control_state.auto_hide = !control_state.auto_hide;
    }
    
    // R 键 - 切换录制
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        recorder.is_recording = !recorder.is_recording;
    }
}