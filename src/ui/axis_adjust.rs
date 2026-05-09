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
    ui.heading("坐标系");
    ui.separator();
    ui.add_space(6.0);

    // ── 坐标轴显隐 ──
    let axis_label = if coord.show_axes { "隐藏坐标轴" } else { "显示坐标轴" };
    if ui.button(axis_label).clicked() {
        coord.show_axes = !coord.show_axes;
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    // ── 向上方向 ──
    ui.label("向上方向:");
    ui.add_space(2.0);

    // 第一行：选轴（始终切到正方向）
    ui.horizontal(|ui| {
        ui.label("选轴");
        for (axis, name) in [(UpAxis::PlusX, "X"), (UpAxis::PlusY, "Y"), (UpAxis::PlusZ, "Z")] {
            if ui.selectable_label(coord.up_axis == axis, name).clicked() {
                coord.up_axis = axis;
            }
        }
    });

    // 第二行：翻转（同轴翻转方向，异轴切到负方向）
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.label("翻转");
        for (i, name) in ["X", "Y", "Z"].iter().enumerate() {
            let is_on_axis = (i == 0 && matches!(coord.up_axis, UpAxis::PlusX | UpAxis::MinusX))
                || (i == 1 && matches!(coord.up_axis, UpAxis::PlusY | UpAxis::MinusY))
                || (i == 2 && matches!(coord.up_axis, UpAxis::PlusZ | UpAxis::MinusZ));
            if ui.button(format!("↕ {name}")).clicked() {
                coord.up_axis = match (i, is_on_axis, coord.up_axis) {
                    (0, true, UpAxis::PlusX) => UpAxis::MinusX,
                    (0, true, UpAxis::MinusX) => UpAxis::PlusX,
                    (0, false, _) => UpAxis::MinusX,
                    (1, true, UpAxis::PlusY) => UpAxis::MinusY,
                    (1, true, UpAxis::MinusY) => UpAxis::PlusY,
                    (1, false, _) => UpAxis::MinusY,
                    (2, true, UpAxis::PlusZ) => UpAxis::MinusZ,
                    (2, true, UpAxis::MinusZ) => UpAxis::PlusZ,
                    (2, false, _) => UpAxis::MinusZ,
                    _ => unreachable!(),
                };
                // 轴方向翻转 → 手性联动
                coord.handedness = match coord.handedness {
                    Handedness::LeftHanded => Handedness::RightHanded,
                    Handedness::RightHanded => Handedness::LeftHanded,
                };
            }
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    // ── 手性：单按钮，点击在左手系/右手系间翻转 ──
    ui.horizontal(|ui| {
        ui.label("手性:");
        let hand_label = match coord.handedness {
            Handedness::LeftHanded => "左手系",
            Handedness::RightHanded => "右手系",
        };
        if ui.button(hand_label).clicked() {
            coord.handedness = match coord.handedness {
                Handedness::LeftHanded => Handedness::RightHanded,
                Handedness::RightHanded => Handedness::LeftHanded,
            };
        }
    });

    ui.add_space(6.0);
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
