use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use crate::graph::data_processing::actions::record::{DataRecorder, PlaybackManager};

/// 回放 UI 插件
pub struct PlaybackUiPlugin;

impl Plugin for PlaybackUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameSelector>()
            .add_systems(
                Update, 
                playback_ui_system
                    .after(crate::manager::font::core::setup_egui_fonts)
                    .run_if(|font_assets: Option<Res<crate::manager::font::core::FontAssets>>| font_assets.is_some())
            );
    }
}

/// 帧选择器资源 - 管理光标选择的帧范围
#[derive(Resource, Default)]
pub struct FrameSelector {
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub show_timeline: bool,
    pub show_frame_list: bool,
    pub search_filter: String,
}

/// 回放 UI 系统（使用 egui）
pub fn playback_ui_system(
    mut contexts: EguiContexts,
    mut playback: ResMut<PlaybackManager>,
    mut recorder: ResMut<DataRecorder>,
    mut selector: ResMut<FrameSelector>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // 第一帧跳过，确保 egui 完全初始化
    if time.elapsed_secs() < 0.1 {
        return;
    }
    
    // 处理键盘输入
    handle_keyboard_input(&keyboard_input, &mut playback, &mut recorder, &mut selector);

    // 获取 egui 上下文，如果不可用则跳过
    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };
    
    // 主控制面板
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        egui::Window::new("数据回放控制")
            .fixed_pos(egui::pos2(10.0, 10.0))
            .collapsible(false)
            .resizable(true)
            .default_size([350.0, 600.0])
            .show(egui_ctx, |ui| {
                ui.heading("播放控制");
                
                ui.separator();
                
                // 状态显示
                egui::Grid::new("status_grid")
                    .num_columns(2)
                    .spacing([10.0, 5.0])
                    .show(ui, |ui| {
                        ui.label("录制状态:");
                        ui.label(if recorder.is_recording { "开启" } else { "关闭" });
                        ui.end_row();
                        
                        ui.label("总帧数:");
                        ui.label(format!("{}", get_total_frames(&recorder)));
                        ui.end_row();
                        
                        ui.label("当前帧:");
                        ui.label(format!("{}/{}", playback.current_frame_index, get_total_frames(&recorder).max(1)));
                        ui.end_row();
                        
                        ui.label("播放速度:");
                        ui.label(format!("{:.1}x", playback.playback_speed));
                        ui.end_row();
                    });
                
                ui.separator();
                
                // 播放控制按钮
                ui.horizontal(|ui| {
                    if ui.button("播放").clicked() {
                        playback.play();
                    }
                    if ui.button("暂停").clicked() {
                        playback.pause();
                    }
                    if ui.button("停止").clicked() {
                        playback.stop();
                    }
                });
                
                ui.horizontal(|ui| {
                    if ui.button("上一帧").clicked() {
                        playback.previous_frame();
                    }
                    if ui.button("下一帧").clicked() {
                        playback.next_frame(get_total_frames(&recorder));
                    }
                });
                
                ui.separator();
                
                // 速度控制滑块
                ui.label("播放速度:");
                ui.add(egui::Slider::new(&mut playback.playback_speed, 0.1..=4.0)
                    .text("x")
                    .logarithmic(true));
                
                ui.horizontal(|ui| {
                    if ui.button("0.5x").clicked() {
                        playback.playback_speed = 0.5;
                    }
                    if ui.button("1.0x").clicked() {
                        playback.playback_speed = 1.0;
                    }
                    if ui.button("2.0x").clicked() {
                        playback.playback_speed = 2.0;
                    }
                });
                
                ui.separator();
                
                // 视图切换
                ui.collapsing("视图选项", |ui| {
                    ui.checkbox(&mut selector.show_timeline, "显示时间轴");
                    ui.checkbox(&mut selector.show_frame_list, "📋 显示帧列表");
                });
                
                ui.separator();
                
                // 录制控制
                ui.collapsing("🔧 录制设置", |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(if recorder.is_recording { "停止录制" } else { "开始录制" }).clicked() {
                            recorder.is_recording = !recorder.is_recording;
                        }
                        if ui.button("清除所有帧").clicked() {
                            recorder.clear();
                            playback.stop();
                        }
                    });
                    
                    // SQLite 存储状态
                    if recorder.storage.is_some() {
                        ui.colored_label(egui::Color32::GREEN, "SQLite 存储：已启用");
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "SQLite 存储：未启用（内存模式）");
                    }
                });
                
                ui.separator();
                
                // 快捷键说明
                ui.collapsing("快捷键", |ui| {
                    ui.label("空格 - 播放/暂停");
                    ui.label("R - 切换录制");
                    ui.label("C - 清除录制");
                    ui.label("←/→ - 上一帧/下一帧");
                    ui.label("T - 切换时间轴");
                    ui.label("L - 切换帧列表");
                });
            });
        
        // 时间轴窗口
        if selector.show_timeline {
            render_timeline_window(egui_ctx, &mut playback, &recorder, &mut selector);
        }
        
        // 帧列表窗口
        if selector.show_frame_list {
            render_frame_list_window(egui_ctx, &mut playback, &recorder, &mut selector);
        }
    }));
}

/// 获取总帧数（支持 SQLite 和内存模式）
fn get_total_frames(recorder: &DataRecorder) -> usize {
    recorder.total_frames()
}

/// 渲染时间轴窗口
fn render_timeline_window(
    egui_ctx: &egui::Context,
    playback: &mut PlaybackManager,
    recorder: &DataRecorder,
    selector: &mut FrameSelector,
) {
    egui::Window::new("时间轴")
        .fixed_pos(egui::pos2(400.0, 10.0))
        .default_size([400.0, 200.0])
        .resizable(true)
        .show(egui_ctx, |ui| {
            let total_frames = get_total_frames(recorder);
            
            if total_frames == 0 {
                ui.label("暂无数据");
                return;
            }
            
            ui.label(format!("总帧数：{}", total_frames));
            
            ui.separator();
            
            // 时间轴滑块
            ui.horizontal(|ui| {
                ui.label("起始:");
                let mut start_val = selector.selection_start.unwrap_or(0);
                if ui.add(egui::DragValue::new(&mut start_val).range(0..=total_frames.saturating_sub(1))).changed() {
                    selector.selection_start = Some(start_val);
                    // 确保结束不小于起始
                    if let Some(end) = &mut selector.selection_end {
                        if *end < start_val {
                            *end = start_val;
                        }
                    }
                }
                
                ui.label("结束:");
                let mut end_val = selector.selection_end.unwrap_or(total_frames.saturating_sub(1));
                if ui.add(egui::DragValue::new(&mut end_val).range(0..=total_frames.saturating_sub(1))).changed() {
                    selector.selection_end = Some(end_val);
                    // 确保起始不大于结束
                    if let Some(start) = &mut selector.selection_start {
                        if *start > end_val {
                            *start = end_val;
                        }
                    }
                }
            });
            
            ui.separator();
            
            // 可视化时间轴
            let (response, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), 80.0), egui::Sense::click());
            let rect = response.rect;
            
            // 绘制背景
            painter.rect_filled(rect, 0.0, egui::Color32::from_gray(40));
            
            // 绘制帧刻度
            let frame_width = rect.width() / total_frames as f32;
            for i in 0..total_frames {
                let x = rect.left() + i as f32 * frame_width;
                if i % 10 == 0 {
                    painter.line_segment(
                        [egui::pos2(x, rect.bottom() - 20.0), egui::pos2(x, rect.bottom())],
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                    );
                    painter.text(
                        egui::pos2(x + 2.0, rect.bottom() - 25.0),
                        egui::Align2::LEFT_BOTTOM,
                        format!("{}", i),
                        egui::FontId::default(),
                        egui::Color32::WHITE,
                    );
                }
            }
            
            // 绘制选择范围
            if let (Some(start), Some(end)) = (selector.selection_start, selector.selection_end) {
                let start_x = rect.left() + start as f32 * frame_width;
                let end_x = rect.left() + (end + 1) as f32 * frame_width;
                let select_rect = egui::Rect::from_min_max(
                    egui::pos2(start_x, rect.top()),
                    egui::pos2(end_x, rect.bottom()),
                );
                painter.rect_filled(select_rect, 0.0, egui::Color32::from_rgba_unmultiplied(100, 149, 237, 100));
            }
            
            // 绘制当前帧位置
            let current_x = rect.left() + playback.current_frame_index as f32 * frame_width;
            painter.line_segment(
                [egui::pos2(current_x, rect.top()), egui::pos2(current_x, rect.bottom())],
                egui::Stroke::new(2.0, egui::Color32::RED),
            );
            
            // 点击时间轴跳转
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let clicked_frame = ((pos.x - rect.left()) / frame_width) as usize;
                    if clicked_frame < total_frames {
                        playback.current_frame_index = clicked_frame;
                    }
                }
            }
            
            ui.separator();
            
            // 选择操作
            ui.horizontal(|ui| {
                if ui.button("选择全部").clicked() {
                    selector.selection_start = Some(0);
                    selector.selection_end = Some(total_frames.saturating_sub(1));
                }
                if ui.button("清除选择").clicked() {
                    selector.selection_start = None;
                    selector.selection_end = None;
                }
                if ui.button("播放选中范围").clicked() {
                    if let (Some(start), Some(end)) = (selector.selection_start, selector.selection_end) {
                        playback.current_frame_index = start;
                        // TODO: 实现范围播放
                        info!("将播放帧 {} 到 {}", start, end);
                    }
                }
            });
        });
}

/// 渲染帧列表窗口
fn render_frame_list_window(
    egui_ctx: &egui::Context,
    playback: &mut PlaybackManager,
    recorder: &DataRecorder,
    selector: &mut FrameSelector,
) {
    egui::Window::new("📋 帧列表")
        .fixed_pos(egui::pos2(400.0, 250.0))
        .default_size([400.0, 300.0])
        .resizable(true)
        .show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("🔍 搜索:");
                ui.text_edit_singleline(&mut selector.search_filter);
                if ui.button("清空").clicked() {
                    selector.search_filter.clear();
                }
            });
            
            ui.separator();
            
            // 显示帧信息（只支持内存模式，SQLite 模式需要加载）
            if recorder.frames.is_empty() {
                if recorder.storage.is_some() {
                    ui.label("SQLite 模式：帧数据存储在数据库中");
                    ui.label("提示：使用上方的统计信息查看数据库状态");
                } else {
                    ui.label("暂无数据");
                }
                return;
            }
            
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for (idx, frame) in recorder.frames.iter().enumerate() {
                        let is_selected = selector.selection_start.map_or(false, |s| idx >= s) 
                            && selector.selection_end.map_or(false, |e| idx <= e);
                        let is_current = idx == playback.current_frame_index;
                        
                        let response = ui.selectable_label(is_current, format!(
                            "#{} | 点数：{} | 类型：{}",
                            idx,
                            frame.points.len(),
                            frame.frame_type.to_string()
                        ));
                        
                        // 自定义背景色（需要手动绘制）
                        if is_current {
                            // 当前帧高亮
                        } else if is_selected {
                            // 选中帧高亮
                        }
                        
                        if response.clicked() {
                            playback.current_frame_index = idx;
                        }
                    }
                });
            
            ui.separator();
            
            ui.label(format!("显示：{}/{} 帧", recorder.frames.len(), recorder.frames.len()));
        });
}

fn handle_keyboard_input(
    keyboard_input: &ButtonInput<KeyCode>,
    playback: &mut PlaybackManager,
    recorder: &mut DataRecorder,
    selector: &mut FrameSelector,
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
        playback.next_frame(get_total_frames(recorder));
    }
    
    // T 键 - 切换时间轴
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        selector.show_timeline = !selector.show_timeline;
    }
    
    // L 键 - 切换帧列表
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        selector.show_frame_list = !selector.show_frame_list;
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
    mut selector: ResMut<FrameSelector>,
) {
    handle_keyboard_input(&keyboard_input, &mut playback, &mut recorder, &mut selector);
}