//! 文件管理 — 帧数据的保存、加载、清空
//!
//! 也提供通用二次确认 UI（ConfirmRequest / ConfirmResult），全局可用。

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_egui::{EguiPrimaryContextPass, egui};

use crate::data::frame::{FrameManager, KeyFrame, SerializableKeyFrame, FrameStorage};
use crate::render::frame_renderer::EntityMap;
use crate::render::interaction::picking::SelectionBox;
use crate::ui::notifications::NotificationCenter;

// ============================================================================
// 通用二次确认 — 任何系统都可以使用
// ============================================================================

/// 发起确认请求
#[derive(Resource, Default)]
pub struct ConfirmRequest {
    pub active: bool,
    pub title: String,
    pub description: String,
    pub confirm_label: String,
}

/// 读取确认结果
#[derive(Resource, Default)]
pub struct ConfirmResult {
    pub confirmed: bool,
    pub consumed: bool,
}

/// 通用确认框系统（独立于文件管理器运行）
pub fn confirm_dialog_ui_system(
    mut contexts: bevy_egui::EguiContexts,
    cursor_options: bevy::prelude::Single<&bevy::window::CursorOptions>,
    mut request: ResMut<ConfirmRequest>,
    mut result: ResMut<ConfirmResult>,
) {
    if cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked {
        return;
    }
    if !request.active {
        if !result.consumed {
            result.confirmed = false;
            result.consumed = true;
        }
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    // 只读不取 —— confirm_dialog_file_system 只在激活时写一次，后续帧保持
    let title = request.title.as_str();
    let desc = request.description.as_str();
    let confirm_label = request.confirm_label.as_str();

    let mut confirmed = false;
    let mut cancelled = false;

    egui::Window::new(title)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .order(egui::Order::Foreground)
        .frame(egui::Frame {
            fill: egui::Color32::from_rgb(37, 37, 38),
            stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)),
            corner_radius: egui::CornerRadius::same(8),
            inner_margin: egui::Margin::symmetric(20, 16),
            ..default()
        })
        .show(ctx, |ui| {
            ui.set_width(280.0);
            ui.set_height(100.0);
            ui.label(
                egui::RichText::new(title)
                    .size(15.0)
                    .color(egui::Color32::from_rgb(220, 220, 220)),
            );
            ui.add_space(8.0);

            ui.label(
                egui::RichText::new(desc)
                    .size(13.0)
                    .color(egui::Color32::from_rgb(160, 160, 160)),
            );
            ui.add_space(16.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(
                    egui::Button::new(
                        egui::RichText::new(confirm_label)
                            .color(egui::Color32::WHITE)
                            .size(13.0),
                    )
                    .min_size(egui::vec2(90.0, 30.0))
                    .fill(egui::Color32::from_rgb(0, 90, 160))
                    .corner_radius(4)
                ).clicked() {
                    confirmed = true;
                }

                ui.add_space(8.0);

                if ui.add(
                    egui::Button::new(
                        egui::RichText::new("取消")
                            .color(egui::Color32::from_rgb(180, 180, 180))
                            .size(13.0),
                    )
                    .min_size(egui::vec2(60.0, 30.0))
                    .fill(egui::Color32::from_rgb(50, 50, 50))
                    .corner_radius(4)
                ).clicked() {
                    cancelled = true;
                }
            });
        });

    if confirmed {
        request.active = false;
        result.confirmed = true;
        result.consumed = false;
    } else if cancelled || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        request.active = false;
        result.confirmed = false;
        result.consumed = false;
    }
}

// ============================================================================
// 文件管理状态定义
// ============================================================================

#[derive(Debug)]
enum FileOp {
    Loading,
}

#[derive(Debug)]
enum PendingAction {
    Clear,
    LoadWithWarning,
}

#[derive(Resource, Default)]
pub struct FileSaveState {
    active_op: Option<FileOp>,
    pub pending_load_path: Option<PathBuf>,
    pub clear_requested: bool,
    /// 由 files_content 设置，confirm_dialog_file_system 消费。
    /// 存 Option 而非两个 bool，保证跨帧持久。
    confirm_action: Option<PendingAction>,
    /// Phase 1 移入的待确认操作，跨帧保留到 Phase 2 消费
    pending_action_type: Option<PendingAction>,
}

// ============================================================================
// 插件
// ============================================================================

/// 文件操作系统集合，用于确保在渲染系统之前执行
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileOpSet;

pub struct FileManagerUiPlugin;

impl Plugin for FileManagerUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileSaveState>()
            .init_resource::<ConfirmRequest>()
            .init_resource::<ConfirmResult>()
            .configure_sets(Update, FileOpSet)
            .add_systems(EguiPrimaryContextPass, confirm_dialog_ui_system)
            .add_systems(Update, (
                confirm_dialog_file_system,
                file_op_system,
                clear_all_data_system,
            ).in_set(FileOpSet));
    }
}

// ============================================================================
// 文件管理专用确认调度
// ============================================================================

/// 将 FileSaveState 的 pending 标志转译为 ConfirmRequest，并处理结果
fn confirm_dialog_file_system(
    mut state: ResMut<FileSaveState>,
    mut request: ResMut<ConfirmRequest>,
    mut result: ResMut<ConfirmResult>,
) {
    // 对话框正在显示中，不干涉
    if request.active {
        return;
    }

    // Phase 1: 有待确认操作 → 设置对话框（不重存 confirm_action，避免循环）
    if let Some(action) = std::mem::take(&mut state.confirm_action) {
        let (title, description, confirm_label) = match action {
            PendingAction::Clear => (
                "确认清空".into(),
                "清空所有帧数据？\n此操作不可撤销，未保存的数据将丢失。".into(),
                "清空".into(),
            ),
            PendingAction::LoadWithWarning => (
                "未保存的数据".into(),
                "当前数据尚未保存，继续加载将丢失。\n建议先保存再加载。".into(),
                "继续加载".into(),
            ),
        };
        request.active = true;
        request.title = title;
        request.description = description;
        request.confirm_label = confirm_label;
        result.consumed = false;
        state.pending_action_type = Some(action);
        return;
    }

    // Phase 2: 消费确认结果
    if !result.consumed {
        result.consumed = true;
        if result.confirmed {
            if let Some(action) = state.pending_action_type.take() {
                match action {
                    PendingAction::Clear => {
                        state.clear_requested = true;
                    }
                    PendingAction::LoadWithWarning => {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("加载帧数据")
                            .add_filter("Redra Data", &["rdra"])
                            .add_filter("PCD 点云", &["pcd"])
                            .add_filter("所有文件", &["*"])
                            .pick_file()
                        {
                            state.pending_load_path = Some(path);
                        }
                    }
                }
            }
        } else {
            state.pending_action_type = None;
        }
    }
}

// ============================================================================
// 侧栏 UI 内容
// ============================================================================

pub fn files_content(
    ui: &mut egui::Ui,
    frame_manager: &FrameManager,
    storage: &FrameStorage,
    state: &mut FileSaveState,
    notifications: &mut NotificationCenter,
) {
    let total_frames = frame_manager.total_frames();
    let has_data = total_frames > 0;
    let is_busy = state.active_op.is_some();
    // 如果 ConfirmRequest::active 为 true，说明弹窗正在显示
    let confirm_showing = state.confirm_action.is_some();

    // ── 状态横幅 ──────────────────────────────────
    if has_data {
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("📊");
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 200, 255),
                        format!("{} 帧 · {} 实体", total_frames, frame_manager.get_current_keyframe()
                            .map(|kf| kf.ids.len()).unwrap_or(0)),
                    );
                });
            });
    } else {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("📭");
            ui.colored_label(egui::Color32::from_rgb(200, 160, 80), "无帧数据");
        });
        ui.label("等待接收数据后再操作");
    }

    ui.separator();

    // ── 保存 ──────────────────────────────────────
    ui.add_space(2.0);
    ui.label(egui::RichText::new("保存").color(egui::Color32::from_rgb(150, 150, 150)).size(11.0));

    ui.add_enabled(has_data && !is_busy && !confirm_showing, egui::Button::new("💾 另存为..."))
        .clicked()
        .then(|| {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("保存帧数据")
                .set_file_name("frames.rdra")
                .add_filter("Redra Data", &["rdra"])
                .save_file()
            {
                let frames: Vec<SerializableKeyFrame> = frame_manager
                    .get_all_keyframes().iter()
                    .map(SerializableKeyFrame::from).collect();
                match storage.save_to_file(&path, &frames) {
                    Ok(()) => {
                        notifications.notify("已保存 ✓", false);
                    }
                    Err(e) => {
                        notifications.notify(format!("保存失败: {}", e), true);
                    }
                }
            }
        });

    if is_busy {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("正在处理...");
        });
    }

    ui.add_space(4.0);
    ui.separator();

    // ── 加载 ──────────────────────────────────────
    ui.add_space(2.0);
    ui.label(egui::RichText::new("文件").color(egui::Color32::from_rgb(150, 150, 150)).size(11.0));

    if ui.add_enabled(!is_busy && !confirm_showing, egui::Button::new("📂 从文件加载...")).clicked() {
        if has_data {
            state.confirm_action = Some(PendingAction::LoadWithWarning);
        } else {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("加载帧数据")
                .add_filter("Redra Data", &["rdra"])
                .add_filter("PCD 点云", &["pcd"])
                .add_filter("所有文件", &["*"])
                .pick_file()
            {
                state.pending_load_path = Some(path);
            }
        }
    }

    ui.add_space(2.0);
    ui.separator();

    // ── 清空 ──────────────────────────────────────
    ui.add_space(2.0);
    ui.label(egui::RichText::new("数据").color(egui::Color32::from_rgb(150, 150, 150)).size(11.0));

    if ui.add_enabled(has_data && !confirm_showing, egui::Button::new("🗑 清空当前数据")).clicked() {
        state.confirm_action = Some(PendingAction::Clear);
    }
    ui.label(egui::RichText::new("清空后即可接收全新数据").color(egui::Color32::from_rgb(120, 120, 120)).size(11.0));

    ui.separator();
    ui.add_space(2.0);

    // ── 说明 ──────────────────────────────────────
    ui.collapsing("说明", |ui| {
        ui.label("• 帧数据以二进制格式保存 (.rdra)");
        ui.label("• 另存为: 选择保存位置");
        ui.label("• 加载: 从 .rdra/.pcd 文件恢复帧数据");
        ui.label("• 清空: 清除所有帧数据，准备接收新数据");
    });
}

// ============================================================================
// 系统
// ============================================================================

fn file_op_system(
    mut commands: Commands,
    mut state: ResMut<FileSaveState>,
    mut frame_manager: ResMut<FrameManager>,
    storage: Res<FrameStorage>,
    mut entity_map: ResMut<EntityMap>,
    mut notifications: ResMut<NotificationCenter>,
    selection_boxes: Query<Entity, With<SelectionBox>>,
) {
    let path = match state.pending_load_path.take() {
        Some(p) => p,
        None => return,
    };

    state.active_op = Some(FileOp::Loading);

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pcd" => {
            match redra_io::pcd::load_pcd(&path) {
                Ok(pcd_frame) => {
                    clear_all_scene(&mut commands, &selection_boxes, &mut entity_map, &mut frame_manager);
                    let entities = redra_io::pcd::points_to_entities(&pcd_frame.points);
                    let mut kf = KeyFrame::new(0);
                    for (id, mesh, transform) in entities {
                        kf.insert_entity(id, mesh, transform);
                    }
                    let point_count = pcd_frame.points.len();
                    frame_manager.add_keyframe(kf);
                    frame_manager.seek_to_frame(0);
                    notifications.notify(
                        format!("已加载 PCD ({} 个点)", point_count),
                        false,
                    );
                }
                Err(e) => {
                    notifications.notify(format!("PCD 加载失败: {}", e), true);
                }
            }
        }
        _ => {
            match storage.load_from_file(&path) {
                Ok(serializable_frames) => {
                    let frame_count = serializable_frames.len();
                    clear_all_scene(&mut commands, &selection_boxes, &mut entity_map, &mut frame_manager);
                    for sf in serializable_frames {
                        frame_manager.add_keyframe(KeyFrame::from(sf));
                    }
                    frame_manager.seek_to_frame(0);
                    notifications.notify(
                        format!("已加载 {} 帧 ({})", frame_count, path.file_name().unwrap_or_default().to_string_lossy()),
                        false,
                    );
                }
                Err(e) => {
                    notifications.notify(format!("加载失败: {}", e), true);
                }
            }
        }
    }
    state.active_op = None;
}

/// 清空场景中所有渲染实体和帧数据
fn clear_all_scene(
    commands: &mut Commands,
    selection_boxes: &Query<Entity, With<SelectionBox>>,
    entity_map: &mut EntityMap,
    frame_manager: &mut FrameManager,
) {
    for box_entity in selection_boxes.iter() {
        commands.entity(box_entity).despawn();
    }
    for (_, entity) in entity_map.map.drain() {
        commands.entity(entity).despawn();
    }
    for pe in entity_map.drain_point_group_entities() {
        commands.entity(pe).despawn();
    }
    entity_map.clear();
    frame_manager.clear();
}

fn clear_all_data_system(
    mut commands: Commands,
    mut state: ResMut<FileSaveState>,
    mut entity_map: ResMut<EntityMap>,
    mut frame_manager: ResMut<FrameManager>,
    mut notifications: ResMut<NotificationCenter>,
    selection_boxes: Query<Entity, With<SelectionBox>>,
) {
    if !state.clear_requested {
        return;
    }
    state.clear_requested = false;

    let entity_count = entity_map.map.len();
    for box_entity in selection_boxes.iter() {
        commands.entity(box_entity).despawn();
    }
    for (_, entity) in entity_map.map.drain() {
        commands.entity(entity).despawn();
    }
    for pe in entity_map.drain_point_group_entities() {
        commands.entity(pe).despawn();
    }
    entity_map.clear();
    frame_manager.clear();

    notifications.notify(
        format!("已清空，准备接收新数据 (移除了 {} 个实体)", entity_count),
        false,
    );
    info!("数据已清空，共移除 {} 个实体", entity_count);
}
