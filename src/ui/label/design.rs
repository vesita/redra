use bevy::prelude::*;
use bevy_egui::prelude::*;

use crate::ui::UIStates;

#[derive(Clone, Debug)]
pub struct LabelInfo {
    pub entity_id: u64,
    pub text: String,
    pub world_position: Vec3,
}

#[derive(Resource, Default)]
pub struct HoverLabel {
    pub current: Option<LabelInfo>,
}

impl HoverLabel {
    pub fn show(&mut self, entity_id: u64, text: String, world_position: Vec3) {
        self.current = Some(LabelInfo {
            entity_id,
            text,
            world_position,
        });
    }
    pub fn hide(&mut self) {
        self.current = None;
    }
}

/// Tag 编辑状态
#[derive(Resource, Default)]
pub struct TagEditState {
    pub editing_entity: Option<u64>,
    pub draft: String,
}

/// 编辑结果（由 UI 产生，由 system 消费）
#[derive(Resource, Default)]
pub struct TagEditResult {
    pub pending: Option<(u64, String)>,
}

/// VS Code 风格悬浮标签（支持 Tag 编辑）
pub fn show_hover_label(
    mut contexts: EguiContexts,
    hover_label: Res<HoverLabel>,
    mut edit_state: ResMut<TagEditState>,
    mut edit_result: ResMut<TagEditResult>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
) {
    let Some(label_info) = &hover_label.current else {
        edit_state.editing_entity = None;
        return;
    };
    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };
    let (camera, camera_transform) = match camera_query.iter().next() {
        Some(cam) => cam,
        None => return,
    };
    let screen_pos = match camera.world_to_viewport(camera_transform, label_info.world_position) {
        Ok(pos) => pos,
        Err(_) => return,
    };

    let entity_id = label_info.entity_id;
    let is_editing = edit_state.editing_entity == Some(entity_id);
    let label_width = 220.0;

    egui::Area::new("hover_label".into())
        .fixed_pos(egui::pos2(
            screen_pos.x - label_width / 2.0,
            screen_pos.y - 10.0,
        ))
        .order(egui::Order::Foreground)
        .show(egui_ctx, |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_rgb(30, 30, 30))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(85, 85, 85)))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 2].into(),
                    blur: 8,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(100),
                })
                .corner_radius(egui::CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 6,
                    se: 6,
                })
                .inner_margin(egui::Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.set_min_width(120.0);
                    ui.set_max_width(label_width);

                    // 顶部：实体 ID + 编辑按钮
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("ID:")
                                .color(egui::Color32::from_rgb(150, 150, 150))
                                .size(11.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("{}", entity_id))
                                .color(egui::Color32::from_rgb(86, 156, 214))
                                .size(11.0),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !is_editing {
                                if ui.small_button("\u{270F}").on_hover_text("编辑 Tag").clicked() {
                                    edit_state.editing_entity = Some(entity_id);
                                    edit_state.draft = label_info.text.clone();
                                }
                            }
                        });
                    });

                    ui.separator();

                    if is_editing {
                        // 编辑模式：输入框 + 确认/取消
                        let response = ui.text_edit_singleline(&mut edit_state.draft);
                        let submitted = response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        let cancelled = response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Escape));

                        ui.horizontal(|ui| {
                            if ui.small_button("确认").clicked() || submitted {
                                edit_result.pending = Some((entity_id, edit_state.draft.clone()));
                                edit_state.editing_entity = None;
                            }
                            if ui.small_button("取消").clicked() || cancelled {
                                edit_state.editing_entity = None;
                            }
                        });

                        // 让输入框保持焦点
                        if edit_state.editing_entity.is_some() {
                            response.request_focus();
                        }
                    } else {
                        // 正常模式：显示文本
                        let display_text = if label_info.text.is_empty() {
                            "(无 Tag)"
                        } else {
                            &label_info.text
                        };
                        ui.label(
                            egui::RichText::new(display_text)
                                .color(if label_info.text.is_empty() {
                                    egui::Color32::from_rgb(100, 100, 100)
                                } else {
                                    egui::Color32::from_rgb(212, 212, 212)
                                })
                                .size(13.0),
                        );
                    }
                });
        });
}

pub fn show_label_panel(mut contexts: EguiContexts, mut ui_states: ResMut<UIStates>) {
    if !ui_states.show_label_panel {
        return;
    }
    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::Window::new("标签面板")
        .collapsible(true)
        .resizable(true)
        .default_width(300.0)
        .default_height(400.0)
        .show(egui_ctx, |ui| {
            ui.heading("标签管理");
            ui.separator();
            let mut search_text = String::new();
            ui.horizontal(|ui| {
                ui.label("搜索:");
                ui.text_edit_singleline(&mut search_text);
            });
            ui.add_space(8.0);
            ui.group(|ui| {
                ui.set_min_height(200.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, (name, desc)) in [
                        ("标签 1", "描述信息 1"),
                        ("标签 2", "描述信息 2"),
                        ("标签 3", "描述信息 3"),
                    ]
                    .iter()
                    .enumerate()
                    {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}. {}", i + 1, name));
                            if ui.small_button("编辑").clicked() {}
                            if ui.small_button("删除").clicked() {}
                        });
                        ui.indent("", |ui| {
                            ui.label(format!("   {}", desc));
                        });
                        ui.add_space(4.0);
                    }
                });
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("新建标签").clicked() {}
                if ui.button("导入").clicked() {}
                if ui.button("导出").clicked() {}
            });
            ui.add_space(8.0);
            ui.separator();
            if ui.button("关闭面板").clicked() {
                ui_states.show_label_panel = false;
            }
        });
}
