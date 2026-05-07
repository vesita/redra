use bevy::prelude::*;
use bevy_egui::egui;
use crate::data::frame::{FrameManager, PlaybackState};
use crate::ui::file_manager::FileSaveState;
use crate::ui::shell::{SidebarState, SidebarView};
use smooth_bevy_cameras::LookTransform;

/// 视角回正请求资源
#[derive(Resource, Default)]
pub struct ResetCameraView(pub bool);

pub struct PlaybackUiPlugin;

impl Plugin for PlaybackUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResetCameraView>()
            .add_systems(Update, (keyboard_shortcuts, reset_camera_system));
    }
}

fn keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    sidebar: Res<SidebarState>,
    mut frame_manager: ResMut<FrameManager>,
    mut playback_state: ResMut<PlaybackState>,
) {
    let total_frames = frame_manager.total_frames();

    // 空格播放/暂停仅在回放面板激活时生效
    let playback_active = sidebar.visible && sidebar.active_view == SidebarView::Playback;
    if playback_active && keyboard.just_pressed(KeyCode::Space) {
        if total_frames == 0 {
            playback_state.toggle();
            if playback_state.is_playing {
                log::warn!("没有帧数据，无法播放");
            }
        } else {
            playback_state.toggle();
        }
    }

    if total_frames == 0 {
        return;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        frame_manager.prev_frame();
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        frame_manager.next_frame();
    }
    if keyboard.just_pressed(KeyCode::Home) {
        frame_manager.seek_to_frame(0);
    }
    if keyboard.just_pressed(KeyCode::End) {
        frame_manager.seek_to_frame(total_frames - 1);
    }
}

/// 侧栏中嵌入的回放控制 UI 内容
pub fn playback_content(
    ui: &mut egui::Ui,
    frame_manager: &mut FrameManager,
    playback_state: &mut PlaybackState,
    file_state: &FileSaveState,
) {
    let total_frames = frame_manager.total_frames();
    let current_frame = frame_manager.current_frame_index();

    if total_frames == 0 {
        ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "等待数据...");
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

    // 文件名
    if let Some(name) = &file_state.current_file_name {
        ui.horizontal(|ui| {
            ui.label("文件:");
            ui.colored_label(egui::Color32::from_rgb(220, 220, 170), name);
        });
        ui.separator();
    }

    // 帧信息
    ui.horizontal(|ui| {
        ui.label("当前帧:");
        ui.colored_label(
            egui::Color32::from_rgb(100, 200, 255),
            format!("{}/{}", current_frame + 1, total_frames),
        );
    });
    ui.separator();

    // 播放控制按钮
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        let first = ui.add_enabled(current_frame > 0, egui::Button::new("⏮").min_size(egui::vec2(32.0, 28.0)));
        if first.clicked() {
            frame_manager.seek_to_frame(0);
        }

        let prev = ui.add_enabled(current_frame > 0, egui::Button::new("◀").min_size(egui::vec2(32.0, 28.0)));
        if prev.clicked() {
            frame_manager.prev_frame();
        }

        let play_text = if playback_state.is_playing { "⏸" } else { "▶" };
        let play = ui.add(
            egui::Button::new(egui::RichText::new(play_text).size(16.0))
                .min_size(egui::vec2(40.0, 28.0))
                .fill(egui::Color32::from_rgb(0, 100, 180)),
        );
        if play.clicked() {
            playback_state.toggle();
        }

        let next = ui.add_enabled(current_frame < total_frames - 1, egui::Button::new("▶").min_size(egui::vec2(32.0, 28.0)));
        if next.clicked() {
            frame_manager.next_frame();
        }

        let last = ui.add_enabled(current_frame < total_frames - 1, egui::Button::new("⏭").min_size(egui::vec2(32.0, 28.0)));
        if last.clicked() {
            frame_manager.seek_to_frame(total_frames - 1);
        }
    });

    ui.separator();

    // 播放速度选择
    ui.horizontal(|ui| {
        ui.label("速度:");
        for &speed in &[10.0, 30.0, 60.0, 120.0] {
            let selected = (playback_state.playback_speed - speed).abs() < 0.1;
            let label = format!("{:.1}x", speed / 30.0);
            if ui
                .add(egui::Button::selectable(selected, label))
                .clicked()
            {
                playback_state.set_speed(speed);
            }
        }
    });

    // 自定义速度滑块
    ui.horizontal(|ui| {
        ui.label("自定义:");
        let mut speed = playback_state.playback_speed;
        if ui
            .add(egui::Slider::new(&mut speed, 1.0..=240.0).text("FPS"))
            .changed()
        {
            playback_state.set_speed(speed);
        }
    });

    ui.separator();

    // 跳转 — 进度条 + 输入框
    ui.label("跳转:");
    ui.horizontal(|ui| {
        let mut frame_idx = current_frame as i32;
        let slider = egui::Slider::new(&mut frame_idx, 0..=(total_frames as i32 - 1))
            .show_value(false);
        if ui.add(slider).changed() {
            frame_manager.seek_to_frame(frame_idx.max(0) as usize);
        }
        let mut edit_idx = current_frame as i32;
        if ui
            .add(
                egui::DragValue::new(&mut edit_idx)
                    .range(0..=(total_frames as i32 - 1))
                    .speed(1)
                    .suffix(format!(" / {}", total_frames - 1)),
            )
            .changed()
        {
            frame_manager.seek_to_frame(edit_idx.max(0) as usize);
        }
    });

    ui.separator();
    ui.collapsing("快捷键", |ui| {
        ui.label("空格 - 播放/暂停");
        ui.label("左/右箭头 - 上一帧/下一帧");
        ui.label("Home/End - 首帧/尾帧");
        ui.label("Alt - 显示/隐藏 UI");
    });
}

const DEFAULT_EYE: Vec3 = Vec3::new(-2.5, 4.5, 9.0);
const DEFAULT_TARGET: Vec3 = Vec3::ZERO;
const DEFAULT_UP: Vec3 = Vec3::Y;

fn reset_camera_system(
    mut reset: ResMut<ResetCameraView>,
    mut cameras: Query<&mut LookTransform>,
) {
    if !reset.0 {
        return;
    }
    reset.0 = false;

    for mut transform in cameras.iter_mut() {
        *transform = LookTransform::new(DEFAULT_EYE, DEFAULT_TARGET, DEFAULT_UP);
        log::info!("视角已回正：eye={:?}, target={:?}", DEFAULT_EYE, DEFAULT_TARGET);
    }
}
