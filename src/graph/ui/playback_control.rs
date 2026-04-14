use bevy::{
    camera::{CameraOutputMode, visibility::RenderLayers},
    prelude::*,
    render::render_resource::BlendState,
    window::PrimaryWindow,
};
use bevy_egui::{EguiContexts, EguiGlobalSettings, EguiPrimaryContextPass, PrimaryEguiContext, egui};

use crate::manager::{data_processing::actions::record::{DataRecorder, PlaybackManager, RecordingMode}, font::core::FontLoadStatus};


/// 回放 UI 插件
pub struct PlaybackUiPlugin;

impl Plugin for PlaybackUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameSelector>()
            .init_resource::<PlaybackPanelState>()
            // 添加启动时设置 UI 相机的系统
            .add_systems(Startup, setup_playback_ui_camera)
            // 使用 EguiPrimaryContextPass 阶段处理 UI 渲染和输入
            .add_systems(
                EguiPrimaryContextPass, 
                (
                    update_viewport_for_panels,
                    playback_ui_system.run_if(font_loaded)
                )
            );
    }
}

/// 面板状态资源 - 存储面板尺寸信息
#[derive(Resource, Default)]
pub struct PlaybackPanelState {
    pub panel_width: f32,
    pub panel_height: f32,
}

/// 字体加载状态检查函数
fn font_loaded(font_status: Res<FontLoadStatus>) -> bool {
    *font_status == FontLoadStatus::Loaded
}

/// 设置回放 UI 相机
fn setup_playback_ui_camera(
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
) {
    // 禁用自动创建主上下文，以便手动设置我们需要的相机
    egui_global_settings.auto_create_primary_context = false;

    // 主世界相机（如果还没有的话）
    // 注意：这里假设项目中已有主相机，如果没有需要取消注释
    // commands.spawn((
    //     Camera3d::default(),
    //     Name::new("Main World Camera")
    // ));

    // EGUI 相机，用于渲染 UI
    commands.spawn((
        // PrimaryEguiContext 组件需要渲染主上下文的所有内容
        PrimaryEguiContext,
        Camera2d::default(),
        // 设置渲染层为无，确保我们只渲染 UI
        RenderLayers::none(),
        Camera {
            order: 100,  // 设置更高的渲染顺序，确保 UI 在所有其他相机之上渲染
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::ALPHA_BLENDING),
                clear_color: ClearColorConfig::None,
            },
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        Name::new("Playback UI Camera")
    ));
}

/// 更新视口以适应面板布局
fn update_viewport_for_panels(
    mut contexts: EguiContexts,
    mut ui_camera: Query<&mut Camera, With<PrimaryEguiContext>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut panel_state: ResMut<PlaybackPanelState>,
) {
    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };

    let Ok(mut camera) = ui_camera.single_mut() else {
        return;
    };

    // 获取窗口物理尺寸
    let _window_width = window.physical_width() as f32;
    let _window_height = window.physical_height() as f32;
    let scale_factor = window.scale_factor();

    // 计算面板占用的空间（基于固定位置和大小）
    // 主控制面板固定在左上角 350x600
    let panel_left = 10.0 * scale_factor;
    let panel_top = 10.0 * scale_factor;
    let panel_right = panel_left + 350.0 * scale_factor;
    let panel_bottom = panel_top + 600.0 * scale_factor;

    // 如果有时间轴窗口显示，也需要考虑它的空间
    // 时间轴在 (400, 10)，大小 400x200
    let timeline_right = (400.0 + 400.0) * scale_factor;
    let timeline_bottom = (10.0 + 200.0) * scale_factor;

    // 计算最大占用区域
    let max_right = panel_right.max(timeline_right);
    let max_bottom = panel_bottom.max(timeline_bottom);

    // 保存面板状态供其他系统使用
    panel_state.panel_width = max_right;
    panel_state.panel_height = max_bottom;

    // 设置 UI 相机的视口为整个窗口（因为 UI 需要全屏接收输入）
    // 注意：这里不裁剪视口，而是让 UI 相机渲染全屏透明层
    // 实际的遮挡由 RenderLayers 和 Camera order 控制
    camera.viewport = None; // UI 相机使用完整视口
    
    // 使用 egui_ctx 避免未使用警告
    let _ = egui_ctx;
}

/// 帧选择器资源 - 管理光标选择的帧范围
#[derive(Resource, Default)]
pub struct FrameSelector {
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub show_timeline: bool,
    pub show_frame_list: bool,
    pub search_filter: String,
    pub current_page: usize,  // 当前页码（从 0 开始）
    pub page_size: usize,     // 每页显示数量
}

/// 回放 UI 系统（使用 egui）
pub fn playback_ui_system(
    mut contexts: EguiContexts,
    cursor_options: bevy::prelude::Single<&bevy::window::CursorOptions>,
    mut playback: ResMut<PlaybackManager>,
    mut recorder: ResMut<DataRecorder>,
    mut selector: ResMut<FrameSelector>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // 如果光标被锁定（FPS模式），不显示UI
    if cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked {
        return;
    }

    // 处理键盘输入
    handle_keyboard_input(&keyboard_input, &mut playback, &mut recorder, &mut *selector);

    // 获取 egui 上下文，如果不可用则跳过
    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };

    // 调试：检查输入焦点状态
    let wants_pointer = egui_ctx.wants_pointer_input();
    let wants_keyboard = egui_ctx.wants_keyboard_input();
    debug!("Egui 输入焦点 - 指针: {}, 键盘: {}", wants_pointer, wants_keyboard);

    // 处理键盘输入
    handle_keyboard_input(&keyboard_input, &mut playback, &mut recorder, &mut *selector);

    // 主控制面板
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
                        ui.label("录制模式:");
                        let mode_text = match recorder.get_recording_mode() {
                            RecordingMode::Off => "❌ 关闭",
                            RecordingMode::MemoryOnly => "💾 仅内存",
                            RecordingMode::AutoSave => "💿 自动保存",
                        };
                        ui.colored_label(
                            if recorder.get_recording_mode() == &RecordingMode::Off {
                                egui::Color32::RED
                            } else {
                                egui::Color32::GREEN
                            },
                            mode_text
                        );
                        ui.end_row();
                        
                        ui.label("内存帧数:");
                        ui.label(format!("{}", recorder.memory_frame_count()));
                        ui.end_row();
                        
                        ui.label("数据库帧数:");
                        ui.label(format!("{}", recorder.database_frame_count()));
                        ui.end_row();
                        
                        ui.label("总点数:");
                        ui.label(format!("{}", recorder.total_points_received));
                        ui.end_row();
                        
                        ui.label("当前构建帧:");
                        let building_status = if recorder.current_builder.is_some() {
                            "🔄 正在接收..."
                        } else {
                            "⏸️ 空闲"
                        };
                        ui.colored_label(
                            if recorder.current_builder.is_some() {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::GRAY
                            },
                            building_status
                        );
                        ui.end_row();
                    });
                
                ui.separator();
                
                // 录制控制
                ui.heading("📹 录制控制");
                
                // 显示当前录制状态提示
                let mode_text = match recorder.get_recording_mode() {
                    RecordingMode::Off => "❌ 录制已关闭 - 数据将被忽略",
                    RecordingMode::MemoryOnly => "💾 仅内存模式 - 数据不保存到磁盘",
                    RecordingMode::AutoSave => "💿 自动保存模式 - 数据实时保存",
                };
                
                let mode_color = match recorder.get_recording_mode() {
                    RecordingMode::Off => egui::Color32::RED,
                    RecordingMode::MemoryOnly => egui::Color32::YELLOW,
                    RecordingMode::AutoSave => egui::Color32::GREEN,
                };
                
                ui.colored_label(mode_color, mode_text);
                
                ui.horizontal(|ui| {
                    if ui.button("⏹ 关闭录制").clicked() {
                        recorder.set_recording_mode(RecordingMode::Off);
                    }
                    if ui.button("💾 仅内存").clicked() {
                        recorder.set_recording_mode(RecordingMode::MemoryOnly);
                    }
                    if ui.button("💿 自动保存").clicked() {
                        recorder.set_recording_mode(RecordingMode::AutoSave);
                    }
                });
                
                ui.horizontal(|ui| {
                    if ui.button("💾 手动保存到磁盘").clicked() {
                        match recorder.save_to_disk() {
                            Ok(count) => {
                                log::info!("✅ 成功保存 {} 帧到磁盘", count);
                            }
                            Err(e) => {
                                log::error!("❌ 保存失败: {}", e);
                            }
                        }
                    }
                    
                    if ui.button("🗑️ 清空所有数据").clicked() {
                        recorder.clear();
                    }
                });
                
                ui.separator();
                
                // 播放控制按钮
                ui.heading("▶️ 回放控制");
                
                // 显示回放状态提示
                let total_frames = recorder.total_frames();
                if total_frames == 0 {
                    ui.colored_label(egui::Color32::GRAY, "⚠️ 没有可回放的帧数据");
                    ui.label("请先开启录制模式并接收数据");
                } else if !playback.is_playing {
                    ui.colored_label(egui::Color32::GREEN, format!("✅ 就绪: {} 帧可回放", total_frames));
                    ui.label("点击 '播放' 按钮或按空格键开始回放");
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "🔄 正在回放...");
                    ui.label(format!("当前帧: {} / {}", playback.current_frame_index + 1, total_frames));
                }
                
                ui.horizontal(|ui| {
                    let play_button = ui.add_enabled(
                        total_frames > 0,
                        egui::Button::new(if playback.is_playing { "⏸ 暂停" } else { "▶ 播放" })
                    );
                    
                    if play_button.clicked() {
                        if playback.is_playing {
                            info!("✅ 暂停按钮被点击");
                            playback.pause();
                        } else {
                            info!("✅ 播放按钮被点击");
                            playback.play();
                        }
                    }
                    
                    if ui.button("⏹ 停止").clicked() {
                        info!("✅ 停止按钮被点击");
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
            render_timeline_window(egui_ctx, &mut playback, &recorder, &mut *selector);
        }
        
        // 帧列表窗口
        if selector.show_frame_list {
            render_frame_list_window(egui_ctx, &mut playback, &recorder, &mut *selector);
        }
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
            // 搜索栏
            ui.horizontal(|ui| {
                ui.label("🔍 搜索:");
                if ui.text_edit_singleline(&mut selector.search_filter).changed() {
                    // 搜索条件改变时重置页码
                    selector.current_page = 0;
                }
                if ui.button("清空").clicked() {
                    selector.search_filter.clear();
                    selector.current_page = 0;
                }
            });
            
            ui.separator();
            
            // 获取总帧数用于显示统计信息
            let total_frames = recorder.total_frames();
            
            if total_frames == 0 {
                ui.label("暂无数据");
                return;
            }
            
            // 使用分页获取帧数据（每页 100 帧）
            selector.page_size = 100;
            let (page_frames, total_pages) = recorder.get_frames_paginated(
                selector.current_page,
                selector.page_size,
            );
            
            // 如果有搜索条件，则进行搜索过滤
            let display_frames = if !selector.search_filter.is_empty() {
                recorder.search_frames(&selector.search_filter)
            } else {
                page_frames
            };
            
            // 显示帧列表
            if display_frames.is_empty() {
                ui.label("未找到匹配的帧");
            } else {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for (idx_in_list, (frame_id, sequence, timestamp, point_count, frame_type)) in display_frames.iter().enumerate() {
                            // 计算全局索引（用于选中状态判断）
                            let global_idx = if !selector.search_filter.is_empty() {
                                // 搜索模式下，需要找到该帧在完整列表中的位置
                                // 注意：这里假设 get_frame_metadata_list 返回的元数据顺序与帧ID或插入顺序一致
                                // 如果性能敏感，建议优化此查找逻辑或在 search_frames 中直接返回全局索引
                                let all_frames = recorder.get_frame_metadata_list();
                                all_frames.iter().position(|f| f.0 == *frame_id).unwrap_or(idx_in_list)
                            } else {
                                // 分页模式下，计算全局索引
                                selector.current_page * selector.page_size + idx_in_list
                            };
                            
                            let is_selected = selector.selection_start.map_or(false, |s| global_idx >= s) 
                                && selector.selection_end.map_or(false, |e| global_idx <= e);
                            let is_current = global_idx == playback.current_frame_index;
                            
                            // 格式化时间戳为可读格式
                            let time_str = format_timestamp(*timestamp);
                            
                            let response = ui.selectable_label(is_current, format!(
                                "#{} [{}] | {} 点 | seq:{} | {}",
                                frame_id,
                                frame_type,
                                point_count,
                                sequence,
                                time_str
                            ));
                            
                            // 自定义背景色（当前帧高亮）
                            if is_current {
                                // Bevy 会自动处理 selectable_label 的高亮
                            } else if is_selected {
                                // 选中帧的样式由 egui 自动处理
                            }
                            
                            if response.clicked() {
                                playback.current_frame_index = global_idx;
                            }
                        }
                    });
            }
            
            ui.separator();
            
            // 分页控制
            if !selector.search_filter.is_empty() {
                // 搜索模式下不显示分页
                ui.label(format!("搜索结果：{} 帧", display_frames.len()));
            } else {
                // 分页模式
                ui.horizontal(|ui| {
                    if ui.button("⏮ 首页").clicked() && selector.current_page > 0 {
                        selector.current_page = 0;
                    }
                    
                    if ui.button("◀ 上一页").clicked() && selector.current_page > 0 {
                        selector.current_page -= 1;
                    }
                    
                    let total_pages: usize = total_pages as usize;
                    ui.label(format!(
                        "第 {}/{} 页",
                        selector.current_page + 1,
                        total_pages.max(1)
                    ));
                    
                    if ui.button("下一页 ▶").clicked() && (total_pages == 0 || selector.current_page < total_pages - 1) {
                        selector.current_page += 1;
                    }
                    
                    if ui.button("末页 ⏭").clicked() && (total_pages == 0 || selector.current_page < total_pages - 1) {
                        selector.current_page = total_pages.saturating_sub(1);
                    }
                });
                
                ui.label(format!("总计：{} 帧", total_frames));
            }
        });
}

/// 格式化时间戳为可读字符串
fn format_timestamp(timestamp_ms: u64) -> String {
    let seconds = timestamp_ms / 1000;
    let millis = timestamp_ms % 1000;
    
    // 简单格式化为 MM:SS.mmm
    let minutes = seconds / 60;
    let secs = seconds % 60;
    
    format!("{:02}:{:02}.{:03}", minutes, secs, millis)
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

    // R 键 - 切换录制模式（循环切换：Off -> MemoryOnly -> AutoSave -> Off）
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        let new_mode = match recorder.get_recording_mode() {
            RecordingMode::Off => RecordingMode::MemoryOnly,
            RecordingMode::MemoryOnly => RecordingMode::AutoSave,
            RecordingMode::AutoSave => RecordingMode::Off,
        };
        recorder.set_recording_mode(new_mode);
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
    handle_keyboard_input(&keyboard_input, &mut playback, &mut recorder, &mut *selector);
}