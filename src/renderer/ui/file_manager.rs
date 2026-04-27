//! 文件管理 UI 模块
//!
//! 提供类似 Office 的文件保存交互体验：
//! - 另存为：通过系统原生对话框选择保存位置
//! - 快速保存：记住上次保存路径，一键保存
//! - 保存状态反馈：显示保存进度和结果
//!
//! # 职责
//! - 仅负责 UI 交互和路径选择
//! - 调用 FrameStorage Resource 执行实际保存
//! - 管理保存状态和反馈

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::manager::data::frame::{FrameManager, storage::FrameStorage, basic::SerializableKeyFrame};
use crate::manager::data::frame::KeyFrame;
use crate::renderer::frame_renderer::EntityMap;
use crate::manager::interaction::font_manager::FontLoadStatus;

/// 文件保存状态资源
#[derive(Resource, Default)]
pub struct FileSaveState {
    /// 上次保存的文件路径（用于快速保存）
    pub last_save_path: Option<PathBuf>,
    /// 是否正在保存
    pub is_saving: bool,
    /// 保存结果消息（Some 表示有消息需要显示）
    pub save_result: Option<SaveResult>,
    /// 是否正在加载
    pub is_loading: bool,
    /// 加载结果消息
    pub load_result: Option<LoadResult>,
    /// 待加载的文件路径（由 UI 设置，由加载系统处理）
    pub pending_load_path: Option<PathBuf>,
    /// 请求删除选中实体（由 UI 设置，由删除系统处理）
    pub delete_requested: bool,
}

/// 保存结果
#[derive(Clone, Debug)]
pub enum SaveResult {
    /// 保存成功
    Success(String),
    /// 保存失败
    Error(String),
}

impl SaveResult {
    fn is_success(&self) -> bool {
        matches!(self, SaveResult::Success(_))
    }

    fn message(&self) -> &str {
        match self {
            SaveResult::Success(msg) => msg,
            SaveResult::Error(msg) => msg,
        }
    }
}

/// 加载结果
#[derive(Clone, Debug)]
pub enum LoadResult {
    /// 加载成功
    Success(String),
    /// 加载失败
    Error(String),
}

impl LoadResult {
    fn is_success(&self) -> bool {
        matches!(self, LoadResult::Success(_))
    }

    fn message(&self) -> &str {
        match self {
            LoadResult::Success(msg) => msg,
            LoadResult::Error(msg) => msg,
        }
    }
}

/// 删除选中实体事件
#[derive(Event)]
pub struct DeleteSelectedEvent;

/// 文件管理 UI 插件
pub struct FileManagerUiPlugin;

impl Plugin for FileManagerUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileSaveState>()
            .add_systems(EguiPrimaryContextPass, (
                file_manager_ui_system.run_if(font_loaded),
            ))
            .add_systems(Update, (
                delete_selected_entities_system,
                file_load_system,
            ));
    }
}

/// 字体加载状态检查
fn font_loaded(font_status: Res<FontLoadStatus>) -> bool {
    *font_status == FontLoadStatus::Loaded
}

/// 文件管理 UI 系统
pub fn file_manager_ui_system(
    mut contexts: EguiContexts,
    cursor_options: bevy::prelude::Single<&bevy::window::CursorOptions>,
    mut save_state: ResMut<FileSaveState>,
    frame_manager: Res<FrameManager>,
    storage: Res<FrameStorage>,
) {
    // 如果光标被锁定（FPS模式），不显示UI
    if cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked {
        return;
    }

    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };

    let total_frames = frame_manager.total_frames();
    let has_data = total_frames > 0;

    // 文件管理面板
    egui::Window::new("文件管理")
        .fixed_pos(egui::pos2(350.0, 10.0))
        .collapsible(true)
        .resizable(false)
        .default_size([280.0, 240.0])
        .show(egui_ctx, |ui| {
            ui.set_max_width(260.0);

            // 数据状态提示
            if !has_data {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 200, 100),
                    "无数据可保存"
                );
                ui.label("请先接收帧数据");
            } else {
                ui.label(format!("当前帧数: {}", total_frames));
            }
            
            ui.separator();

            // 加载按钮区域（始终可用）
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 8.0;

                // 加载按钮
                let load_enabled = !save_state.is_loading && !save_state.is_saving;
                if ui.add_enabled(load_enabled, egui::Button::new("从文件加载...")).clicked() {
                    // 检查当前是否有数据
                    if frame_manager.has_frames() {
                        // 有数据，询问是否保存
                        let should_save = rfd::MessageDialog::new()
                            .set_title("确认")
                            .set_description("当前有未保存的数据，是否先保存？")
                            .set_buttons(rfd::MessageButtons::YesNo)
                            .show();
                        
                        if should_save == rfd::MessageDialogResult::Yes {
                            // 用户选择"是"，先触发保存对话框
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("保存帧数据")
                                .set_file_name("frames.rdra")
                                .add_filter("Redra Data", &["rdra"])
                                .save_file()
                            {
                                log::info!("选择保存路径: {:?}", path);
                                
                                save_state.is_saving = true;
                                let serializable_frames: Vec<SerializableKeyFrame> = frame_manager.get_all_keyframes()
                                    .iter()
                                    .map(SerializableKeyFrame::from)
                                    .collect();
                                
                                match storage.save_to_file(&path, &serializable_frames) {
                                    Ok(_) => {
                                        save_state.last_save_path = Some(path.clone());
                                        save_state.save_result = Some(SaveResult::Success(
                                            format!("已保存 {} 帧到: {}", total_frames, path.display())
                                        ));
                                    }
                                    Err(e) => {
                                        save_state.save_result = Some(SaveResult::Error(
                                            format!("保存失败: {}", e)
                                        ));
                                        save_state.is_saving = false;
                                        return; // 保存失败，不继续加载
                                    }
                                }
                                save_state.is_saving = false;
                            } else {
                                // 用户取消了保存，也取消加载
                                return;
                            }
                        }
                        // 如果用户选择"否"，继续加载流程（不保存）
                    }

                    // 弹出文件选择对话框
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("加载帧数据")
                        .add_filter("Redra Data", &["rdra"])
                        .pick_file()
                    {
                        log::info!("选择加载文件: {:?}", path);
                        
                        // 设置待加载路径，由加载系统处理
                        save_state.pending_load_path = Some(path);
                        save_state.is_loading = true;
                    }
                }

                // 显示加载结果
                if let Some(ref result) = save_state.load_result {
                    let color = if result.is_success() {
                        egui::Color32::from_rgb(100, 255, 100)
                    } else {
                        egui::Color32::from_rgb(255, 100, 100)
                    };

                    ui.colored_label(color, result.message());

                    // 添加关闭按钮
                    ui.horizontal(|ui| {
                        if ui.small_button("×").clicked() {
                            save_state.load_result = None;
                        }
                    });
                }
            });

            // 如果有数据，显示保存功能
            if has_data {
                ui.separator();

                // 保存按钮区域
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;

                    // 另存为按钮
                    let save_as_enabled = !save_state.is_saving;
                    if ui.add_enabled(save_as_enabled, egui::Button::new("另存为...")).clicked() {
                        // 触发文件对话框
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("保存帧数据")
                            .set_file_name("frames.rdra")
                            .add_filter("Redra Data", &["rdra"])
                            .save_file()
                        {
                            log::info!("选择保存路径: {:?}", path);
                            
                            // 准备数据并执行保存
                            save_state.is_saving = true;
                            let serializable_frames: Vec<SerializableKeyFrame> = frame_manager.get_all_keyframes()
                                .iter()
                                .map(SerializableKeyFrame::from)
                                .collect();
                            
                            match storage.save_to_file(&path, &serializable_frames) {
                                Ok(_) => {
                                    save_state.last_save_path = Some(path.clone());
                                    save_state.save_result = Some(SaveResult::Success(
                                        format!("已保存 {} 帧到: {}", total_frames, path.display())
                                    ));
                                }
                                Err(e) => {
                                    save_state.save_result = Some(SaveResult::Error(
                                        format!("保存失败: {}", e)
                                    ));
                                }
                            }
                            save_state.is_saving = false;
                        }
                    }

                    // 快速保存按钮
                    let quick_save_enabled = !save_state.is_saving && save_state.last_save_path.is_some();
                    let quick_save_text = if let Some(ref path) = save_state.last_save_path {
                        format!("快速保存 ({})", path.file_name().unwrap_or_default().to_string_lossy())
                    } else {
                        "快速保存".to_string()
                    };

                    if ui.add_enabled(quick_save_enabled, egui::Button::new(quick_save_text)).clicked() {
                        // 克隆路径以避免借用冲突
                        if let Some(path) = save_state.last_save_path.clone() {
                            log::info!("快速保存到: {:?}", path);
                            
                            // 准备数据并执行保存
                            save_state.is_saving = true;
                            let serializable_frames: Vec<SerializableKeyFrame> = frame_manager.get_all_keyframes()
                                .iter()
                                .map(SerializableKeyFrame::from)
                                .collect();
                            
                            match storage.save_to_file(&path, &serializable_frames) {
                                Ok(_) => {
                                    save_state.save_result = Some(SaveResult::Success(
                                        format!("已保存 {} 帧到: {}", total_frames, path.display())
                                    ));
                                }
                                Err(e) => {
                                    save_state.save_result = Some(SaveResult::Error(
                                        format!("保存失败: {}", e)
                                    ));
                                }
                            }
                            save_state.is_saving = false;
                        }
                    }

                    // 保存状态提示
                    if save_state.is_saving {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("正在保存...");
                        });
                    }

                    // 显示保存结果
                    if let Some(ref result) = save_state.save_result {
                        let color = if result.is_success() {
                            egui::Color32::from_rgb(100, 255, 100)
                        } else {
                            egui::Color32::from_rgb(255, 100, 100)
                        };

                        ui.colored_label(color, result.message());

                        // 添加关闭按钮
                        ui.horizontal(|ui| {
                            if ui.small_button("×").clicked() {
                                save_state.save_result = None;
                            }
                        });
                    }
                });
            }

            ui.separator();

            // 实体管理区域
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 8.0;

                // 删除选中实体按钮
                if ui.button("删除选中实体 (Del)").clicked() {
                    save_state.delete_requested = true;
                }

                ui.label("提示: 点击场景中的实体可选中");
            });

            ui.separator();

            // 帮助信息
            ui.collapsing("说明", |ui| {
                ui.label("• 另存为: 选择新的保存位置");
                ui.label("• 快速保存: 保存到上次位置");
                ui.label("• 加载: 从文件恢复帧数据");
                ui.label("• 删除: 移除选中的实体");
                ui.label("• 数据将保存为二进制格式");
            });
        });
}

/// 删除选中实体系统
fn delete_selected_entities_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut save_state: ResMut<FileSaveState>,
    query: Query<(Entity, &crate::renderer::interaction::picking::PickableEntity), With<crate::renderer::Selected>>,
    mut entity_map: ResMut<EntityMap>,
    mut frame_manager: ResMut<crate::manager::data::frame::FrameManager>,
) {
    // 检查 Delete 键是否按下或 UI 请求删除
    if keyboard.just_pressed(KeyCode::Delete) || save_state.delete_requested {
        let mut deleted_count = 0;
        let mut entity_ids_to_delete = Vec::new();
        
        // 收集所有要删除的实体ID
        for (entity, pickable) in query.iter() {
            entity_ids_to_delete.push(pickable.entity_id);
            
            // 从 EntityMap 中移除映射
            entity_map.map.remove(&pickable.entity_id);
            
            // 销毁 Bevy 实体
            commands.entity(entity).despawn();
            deleted_count += 1;
        }
        
        // 从帧数据中永久删除这些实体
        if !entity_ids_to_delete.is_empty() {
            frame_manager.delete_entities(&entity_ids_to_delete);
        }
        
        if deleted_count > 0 {
            log::info!("已删除 {} 个选中实体", deleted_count);
        } else {
            log::warn!("没有选中的实体可删除");
        }
        
        // 重置标志
        save_state.delete_requested = false;
    }
}

/// 文件加载系统（处理待加载的文件）
fn file_load_system(
    mut commands: Commands,
    mut save_state: ResMut<FileSaveState>,
    mut frame_manager: ResMut<FrameManager>,
    storage: Res<FrameStorage>,
    mut entity_map: ResMut<EntityMap>,
) {
    // 检查是否有待加载的文件
    let pending_path = save_state.pending_load_path.take();
    
    if let Some(path) = pending_path {
        log::info!("开始加载文件: {:?}", path);
        
        match storage.load_from_file(&path) {
            Ok(serializable_frames) => {
                let frame_count = serializable_frames.len();
                
                // 清空实体映射并 despawn 所有帧实体
                for (_, entity) in entity_map.map.drain() {
                    commands.entity(entity).despawn();
                }
                
                // 清空现有数据
                frame_manager.clear();
                
                // 将 SerializableKeyFrame 转换回 KeyFrame 并添加
                for sf in serializable_frames {
                    let keyframe = KeyFrame::from(sf);
                    frame_manager.add_keyframe(keyframe);
                }
                
                // 重置当前帧索引
                frame_manager.seek_to_frame(0);
                
                save_state.load_result = Some(LoadResult::Success(
                    format!("已加载 {} 帧从: {}", frame_count, path.display())
                ));
                log::info!("成功加载 {} 帧", frame_count);
            }
            Err(e) => {
                save_state.load_result = Some(LoadResult::Error(
                    format!("加载失败: {}", e)
                ));
            }
        }
        save_state.is_loading = false;
    }
}






















