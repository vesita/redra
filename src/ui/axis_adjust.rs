use bevy::prelude::*;
use bevy_egui::egui;

use crate::render::coord_system::Handedness;

pub struct AxisAdjustPlugin;

impl Plugin for AxisAdjustPlugin {
    fn build(&self, _app: &mut App) {
        // 无需额外资源，直接读写 Handedness
    }
}

/// 侧栏中嵌入的坐标系设置 UI 内容
pub fn axis_adjust_content(
    ui: &mut egui::Ui,
    handedness: &mut Handedness,
) {
    ui.heading("坐标系设置");
    ui.separator();
    ui.add_space(8.0);

    ui.label("坐标系手性:");
    ui.add_space(4.0);

    let is_lh = *handedness == Handedness::LeftHanded;
    let is_rh = *handedness == Handedness::RightHanded;

    ui.horizontal(|ui| {
        if ui.selectable_label(is_lh, "左手系 (Bevy 原生)").clicked() {
            *handedness = Handedness::LeftHanded;
        }
        if ui.selectable_label(is_rh, "右手系 (标准数学)").clicked() {
            *handedness = Handedness::RightHanded;
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    match *handedness {
        Handedness::LeftHanded => {
            ui.colored_label(
                egui::Color32::from_rgb(100, 180, 220),
                "Bevy 默认左手 Y-up 坐标系。\n数据原样渲染，不做任何转换。",
            );
        }
        Handedness::RightHanded => {
            ui.colored_label(
                egui::Color32::from_rgb(100, 220, 100),
                "标准右手 Y-up 坐标系。\nZ 轴取反，四元数 x/z 取反。",
            );
        }
    }
}
