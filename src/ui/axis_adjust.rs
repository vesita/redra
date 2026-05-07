use bevy::prelude::*;
use bevy_egui::egui;

use crate::render::coord_system::{CoordSystem, Handedness, UpAxis};

pub struct AxisAdjustPlugin;

impl Plugin for AxisAdjustPlugin {
    fn build(&self, _app: &mut App) {}
}

/// 侧栏中嵌入的坐标系设置 UI 内容
pub fn axis_adjust_content(
    ui: &mut egui::Ui,
    coord: &mut CoordSystem,
) {
    ui.heading("坐标系设置");
    ui.separator();
    ui.add_space(8.0);

    // ── 手性 ──
    ui.label("手性:");
    ui.add_space(4.0);

    let is_lh = coord.handedness == Handedness::LeftHanded;
    let is_rh = coord.handedness == Handedness::RightHanded;

    ui.horizontal(|ui| {
        if ui.selectable_label(is_lh, "左手系").clicked() {
            coord.handedness = Handedness::LeftHanded;
        }
        if ui.selectable_label(is_rh, "右手系").clicked() {
            coord.handedness = Handedness::RightHanded;
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    // ── 向上轴 ──
    ui.label("向上轴:");
    ui.add_space(4.0);

    let axes = [
        UpAxis::PlusX, UpAxis::MinusX,
        UpAxis::PlusY, UpAxis::MinusY,
        UpAxis::PlusZ, UpAxis::MinusZ,
    ];

    ui.horizontal(|ui| {
        for axis in &axes {
            if ui.selectable_label(coord.up_axis == *axis, axis.label()).clicked() {
                coord.up_axis = *axis;
            }
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    // ── 当前配置说明 ──
    let hand_desc = match coord.handedness {
        Handedness::LeftHanded => "左手系",
        Handedness::RightHanded => "右手系",
    };
    let axis_desc = match coord.up_axis {
        UpAxis::PlusY => "Y 轴向上 (Bevy/OpenGL 默认)",
        UpAxis::MinusY => "Y 轴向下",
        UpAxis::PlusZ => "Z 轴向上 (GIS/点云常见)",
        UpAxis::MinusZ => "Z 轴向下",
        UpAxis::PlusX => "X 轴向上",
        UpAxis::MinusX => "X 轴向下",
    };
    ui.colored_label(
        egui::Color32::from_rgb(160, 160, 160),
        format!("{} · {}", hand_desc, axis_desc),
    );
}
