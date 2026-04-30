use bevy::prelude::*;
use bevy_egui::egui;

use crate::data::frame::FrameManager;

/// 旋转调整面板状态
#[derive(Resource)]
pub struct AxisAdjustState {
    /// X 轴旋转角度（度）
    pub rot_x: f32,
    /// Y 轴旋转角度（度）
    pub rot_y: f32,
    /// Z 轴旋转角度（度）
    pub rot_z: f32,
    /// 是否绕世界轴旋转（同时旋转位置和朝向）
    pub world_axis: bool,
    /// 上次操作状态消息
    pub status: Option<String>,
    /// 状态消息剩余帧数
    status_ttl: u32,
}

impl Default for AxisAdjustState {
    fn default() -> Self {
        Self { rot_x: 0.0, rot_y: 0.0, rot_z: 0.0, world_axis: false, status: None, status_ttl: 0 }
    }
}

pub struct AxisAdjustPlugin;

impl Plugin for AxisAdjustPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AxisAdjustState>();
        app.add_systems(Update, tick_status);
    }
}

fn tick_status(mut state: ResMut<AxisAdjustState>) {
    if state.status_ttl > 0 {
        state.status_ttl -= 1;
        if state.status_ttl == 0 {
            state.status = None;
        }
    }
}

/// 侧栏中嵌入的旋转调整 UI 内容
pub fn axis_adjust_content(
    ui: &mut egui::Ui,
    frame_manager: &mut FrameManager,
    state: &mut AxisAdjustState,
) {
    ui.heading("旋转调整");
    ui.separator();
    ui.add_space(4.0);

    let entity_count: usize = frame_manager.get_all_keyframes().iter()
        .map(|kf| kf.packs.len())
        .sum();

    if entity_count == 0 {
        ui.colored_label(egui::Color32::GRAY, "当前无帧数据");
        return;
    }

    // 三轴角度滑块
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.checkbox(&mut state.world_axis, "绕世界轴旋转（含位置）");
        if state.world_axis {
            ui.colored_label(egui::Color32::GRAY, "位置将绕原点公转");
        }
    });
    ui.add_space(4.0);

    ui.label("绕 X 轴旋转:");
    ui.add(egui::Slider::new(&mut state.rot_x, -180.0..=180.0)
        .suffix("°"));

    ui.add_space(2.0);
    ui.label("绕 Y 轴旋转:");
    ui.add(egui::Slider::new(&mut state.rot_y, -180.0..=180.0)
        .suffix("°"));

    ui.add_space(2.0);
    ui.label("绕 Z 轴旋转:");
    ui.add(egui::Slider::new(&mut state.rot_z, -180.0..=180.0)
        .suffix("°"));

    ui.add_space(12.0);

    // 状态消息
    if let Some(ref msg) = state.status {
        ui.colored_label(egui::Color32::from_rgb(100, 220, 100), msg);
        ui.add_space(4.0);
    }

    // 应用按钮
    ui.horizontal(|ui| {
        ui.label(format!("共 {} 个对象", entity_count));
    });

    ui.add_space(8.0);

    if ui
        .add(
            egui::Button::new("应用旋转")
                .min_size(egui::vec2(240.0, 36.0))
                .fill(egui::Color32::from_rgb(0, 100, 180)),
        )
        .clicked()
    {
        apply_rotation(frame_manager, state);
    }
}

/// 将所有关键帧中的实体绕 X/Y/Z 轴旋转指定角度（度）
fn apply_rotation(
    frame_manager: &mut FrameManager,
    state: &mut AxisAdjustState,
) {
    let rad_x = state.rot_x.to_radians();
    let rad_y = state.rot_y.to_radians();
    let rad_z = state.rot_z.to_radians();

    if rad_x.abs() < f32::EPSILON && rad_y.abs() < f32::EPSILON && rad_z.abs() < f32::EPSILON {
        state.status = Some("角度均为 0，未作任何改变".into());
        state.status_ttl = 120;
        return;
    }

    let rot = Quat::from_euler(EulerRot::XYZ, rad_x, rad_y, rad_z);

    let count: usize = frame_manager.get_all_keyframes().iter()
        .map(|kf| kf.packs.len())
        .sum();

    // 打印操作前的第一个实体变换用于诊断
    if let Some(kf) = frame_manager.get_current_keyframe() {
        if let Some((_, inpto)) = kf.iter_entities().next() {
            let (r1, r2, r3) = inpto.transform.rotation.to_euler(EulerRot::XYZ);
            log::info!(
                "旋转前 第一个实体 rotation = ({:.4}, {:.4}, {:.4})",
                r1.to_degrees(), r2.to_degrees(), r3.to_degrees(),
            );
        }
    }

    for keyframe in frame_manager.get_all_keyframes_mut().iter_mut() {
        for inpto in keyframe.packs.iter_mut() {
            if state.world_axis {
                // 绕世界轴：位置绕原点公转 + 朝向世界空间旋转
                inpto.transform.translation = rot * inpto.transform.translation;
            }
            inpto.transform.rotation = rot * inpto.transform.rotation;
        }
    }

    // 打印操作后的第一个实体变换用于诊断
    if let Some(kf) = frame_manager.get_current_keyframe() {
        if let Some((_, inpto)) = kf.iter_entities().next() {
            let (r1, r2, r3) = inpto.transform.rotation.to_euler(EulerRot::XYZ);
            log::info!(
                "旋转后 第一个实体 rotation = ({:.4}, {:.4}, {:.4})",
                r1.to_degrees(), r2.to_degrees(), r3.to_degrees(),
            );
        }
    }

    let mode = if state.world_axis { "世界轴" } else { "局部轴" };
    state.status = Some(format!(
        "✓ [{}] 已旋转 {} 个对象 X={:.0}° Y={:.0}° Z={:.0}°",
        mode, count, state.rot_x, state.rot_y, state.rot_z,
    ));
    state.status_ttl = 120;

    log::info!(
        "旋转调整（{}）：已旋转 {} 个对象 X={:.1}° Y={:.1}° Z={:.1}°",
        mode, count, state.rot_x, state.rot_y, state.rot_z,
    );
}
