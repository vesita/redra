use bevy::prelude::*;
use bevy_egui::prelude::*;

use crate::data::tag::{TagFilter, FilterMode, TagRegistry};
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
    let label_width = 260.0;

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

                        if edit_state.editing_entity.is_some() {
                            response.request_focus();
                        }
                    } else {
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

/// 标签面板 + Tag 筛选控件
pub fn show_label_panel(
    mut contexts: EguiContexts,
    mut ui_states: ResMut<UIStates>,
    tag_registry: Res<TagRegistry>,
    mut tag_filter: ResMut<TagFilter>,
) {
    if !ui_states.show_label_panel {
        return;
    }
    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::Window::new("标签面板")
        .collapsible(true)
        .resizable(true)
        .default_width(320.0)
        .default_height(450.0)
        .show(egui_ctx, |ui| {
            ui.heading("标签管理");
            ui.separator();

            // ── 筛选开关 ──────────────────────────────
            ui.horizontal(|ui| {
                let mut enabled = tag_filter.enabled;
                ui.checkbox(&mut enabled, "启用筛选");
                if enabled != tag_filter.enabled {
                    tag_filter.enabled = enabled;
                }
            });

            ui.add_space(4.0);

            if !tag_filter.enabled {
                // 筛选未启用，显示注册的 tag 集合信息
                ui.group(|ui| {
                    ui.set_min_height(150.0);
                    if tag_registry.collections.is_empty() {
                        ui.label(egui::RichText::new("暂无 Tag 集合定义")
                            .color(egui::Color32::from_rgb(120, 120, 120))
                            .size(12.0));
                        ui.label(egui::RichText::new("可通过 TOML 配置或网络动态注册")
                            .color(egui::Color32::from_rgb(100, 100, 100))
                            .size(11.0));
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for collection in tag_registry.collections.values() {
                                ui.collapsing(format!("{} ({})", collection.display_name, collection.name), |ui| {
                                    for opt in &collection.options {
                                        ui.horizontal(|ui| {
                                            let fill = egui::Color32::from_rgba_premultiplied(
                                                (opt.color_r * 255.0) as u8,
                                                (opt.color_g * 255.0) as u8,
                                                (opt.color_b * 255.0) as u8,
                                                (opt.color_a * 255.0) as u8,
                                            );
                                            egui::Frame::default()
                                                .fill(fill)
                                                .corner_radius(2)
                                                .inner_margin(egui::Margin::same(4))
                                                .show(ui, |ui| {
                                                    ui.label(egui::RichText::new(&opt.label).size(11.0));
                                                });
                                        });
                                    }
                                });
                            }
                        });
                    }
                });
                ui.add_space(4.0);
                ui.separator();
                // 面板底部：显示集合信息
                ui.label(egui::RichText::new(format!("已注册 {} 个集合", tag_registry.collections.len()))
                    .color(egui::Color32::from_rgb(120, 120, 120))
                    .size(11.0));
                return;
            }

            // ── 筛选模式切换 ──────────────────────────
            let mode_text = match tag_filter.mode {
                FilterMode::PassThrough => "全部显示",
                FilterMode::Simple => "简单模式",
                FilterMode::Advanced => "复杂模式",
            };
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("模式:").size(12.0));
                if ui.button(egui::RichText::new(mode_text).size(12.0)).clicked() {
                    // 循环切换模式: PassThrough → Simple → Advanced → PassThrough
                    tag_filter.mode = match tag_filter.mode {
                        FilterMode::PassThrough => FilterMode::Simple,
                        FilterMode::Simple => FilterMode::Advanced,
                        FilterMode::Advanced => FilterMode::PassThrough,
                    };
                    if tag_filter.mode == FilterMode::Advanced {
                        // 同步 advanced_text
                        tag_filter.advanced_text = format_tag_filter_expr(&tag_filter);
                    }
                }
            });

            ui.add_space(4.0);

            // ── 筛选内容 ──────────────────────────────
            match &mut tag_filter.mode {
                FilterMode::PassThrough => {
                    ui.label(egui::RichText::new("显示全部实体（无筛选）")
                        .color(egui::Color32::from_rgb(140, 140, 140))
                        .size(12.0));
                }
                FilterMode::Simple => {
                    // 简单模式：按集合展示 checkbox
                    if tag_registry.collections.is_empty() {
                        ui.label(egui::RichText::new("无可用 Tag 集合")
                            .color(egui::Color32::from_rgb(120, 120, 120))
                            .size(12.0));
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for collection in tag_registry.collections.values() {
                                // 找或创建该集合的规则
                                let rule_idx = tag_filter.simple_rules.iter().position(|r| r.collection == collection.name);
                                let rule_exists = rule_idx.is_some();

                                let header = if rule_exists {
                                    format!("▼ {} (已筛选)", collection.display_name)
                                } else {
                                    format!("▷ {} (全部显示)", collection.display_name)
                                };

                                ui.collapsing(header, |ui| {
                                    for opt in &collection.options {
                                        let is_checked = rule_idx
                                            .map(|idx| tag_filter.simple_rules[idx].allowed_values.contains(&opt.key))
                                            .unwrap_or(true); // 无规则时 = 全选

                                        let mut checked = is_checked;
                                        let old_checked = checked;
                                        ui.checkbox(&mut checked, &opt.label);

                                        if checked != old_checked {
                                            if checked {
                                                // 选中：加入 allowlist
                                                if let Some(idx) = rule_idx {
                                                    tag_filter.simple_rules[idx].allowed_values.insert(opt.key.clone());
                                                } else if let Some(idx) = tag_filter.simple_rules.iter().position(|r| r.collection == collection.name) {
                                                    tag_filter.simple_rules[idx].allowed_values.insert(opt.key.clone());
                                                } else {
                                                    let mut allowed = std::collections::HashSet::new();
                                                    allowed.insert(opt.key.clone());
                                                    tag_filter.simple_rules.push(crate::data::tag::SimpleFilterRule {
                                                        collection: collection.name.clone(),
                                                        allowed_values: allowed,
                                                    });
                                                }
                                            } else {
                                                // 取消选中：从 allowlist 移除
                                                if let Some(idx) = rule_idx {
                                                    tag_filter.simple_rules[idx].allowed_values.remove(&opt.key);
                                                    if tag_filter.simple_rules[idx].allowed_values.is_empty() {
                                                        tag_filter.simple_rules.remove(idx);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                });
                            }
                        });
                    }
                }
                FilterMode::Advanced => {
                    // 复杂模式：表达式编辑
                    ui.label(egui::RichText::new("布尔表达式:").size(12.0));
                    let mut expr_text = tag_filter.advanced_text.clone();
                    let response = ui.text_edit_singleline(&mut expr_text);
                    if response.changed() {
                        tag_filter.advanced_text = expr_text.clone();
                    }

                    ui.horizontal(|ui| {
                        if ui.button("应用").clicked() {
                            let _ = tag_filter.parse_expression(&expr_text);
                        }
                        if ui.button("清除").clicked() {
                            tag_filter.expression = None;
                            tag_filter.advanced_text.clear();
                        }
                    });

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("语法: collection=value, AND, OR, NOT(!), 括号")
                        .color(egui::Color32::from_rgb(120, 120, 120))
                        .size(11.0));
                    ui.label(egui::RichText::new("示例: semantic=road AND !instance=car_01")
                        .color(egui::Color32::from_rgb(100, 100, 100))
                        .size(11.0));

                    if let Some(ref _expr) = tag_filter.expression {
                        ui.separator();
                        ui.label(egui::RichText::new("✓ 表达式已解析")
                            .color(egui::Color32::from_rgb(100, 200, 100))
                            .size(12.0));
                    }
                }
            }

            ui.add_space(8.0);
            ui.separator();
            if ui.button("关闭面板").clicked() {
                ui_states.show_label_panel = false;
            }
        });
}

/// 将当前筛选规则格式化为表达式文本
fn format_tag_filter_expr(filter: &TagFilter) -> String {
    match &filter.mode {
        FilterMode::PassThrough => String::new(),
        FilterMode::Simple => {
            let parts: Vec<String> = filter.simple_rules.iter().map(|rule| {
                let vals: Vec<&str> = rule.allowed_values.iter().map(|v| v.as_str()).collect();
                format!("{} IN ({})", rule.collection, vals.join(","))
            }).collect();
            parts.join(" AND ")
        }
        FilterMode::Advanced => filter.advanced_text.clone(),
    }
}
