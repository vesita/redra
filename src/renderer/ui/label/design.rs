
use bevy::prelude::*;
use bevy_egui::prelude::*;

use crate::renderer::ui::UIStates;

/// 悬浮标签信息
#[derive(Clone, Debug)]
pub struct LabelInfo {
    pub entity_id: u64,
    pub text: String,
    pub world_position: Vec3,
}

/// 悬浮标签资源 - 管理当前显示的标签
#[derive(Resource, Default)]
pub struct HoverLabel {
    pub current: Option<LabelInfo>,
}

impl HoverLabel {
    /// 显示标签（直接设置，不做切换）
    pub fn show(&mut self, entity_id: u64, text: String, world_position: Vec3) {
        self.current = Some(LabelInfo {
            entity_id,
            text,
            world_position,
        });
    }

    /// 隐藏标签
    pub fn hide(&mut self) {
        self.current = None;
    }
}

/// 渲染悬浮标签系统
pub fn show_hover_label(
    mut contexts: EguiContexts,
    hover_label: Res<HoverLabel>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
) {
    let Some(label_info) = &hover_label.current else {
        return;
    };

    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };

    // 获取相机并转换世界坐标到屏幕坐标
    let (camera, camera_transform) = match camera_query.iter().next() {
        Some(cam) => cam,
        None => return,
    };

    // 将世界坐标转换为屏幕坐标
    let screen_pos = match camera.world_to_viewport(
        camera_transform,
        label_info.world_position,
    ) {
        Ok(pos) => pos,
        Err(_) => return, // 转换失败则不显示
    };

    // 计算标签位置（偏移一点，避免遮挡实体）
    let label_width = 200.0;
    let label_height = 60.0;
    let final_x = screen_pos.x - label_width / 2.0;
    let final_y = screen_pos.y - label_height - 10.0;

    egui::Window::new("")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_pos(egui::pos2(final_x, final_y))
        .fixed_size(egui::vec2(label_width, label_height))
        .interactable(false) // 禁用交互，防止遮挡拾取
        .order(egui::Order::Foreground) // 前景层级
        .show(egui_ctx, |ui| {
            ui.style_mut().visuals.window_fill = egui::Color32::from_rgba_premultiplied(30, 30, 40, 230);
            ui.style_mut().visuals.window_stroke = egui::Stroke::NONE;
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 4.0);
            
            ui.vertical_centered(|ui| {
                // 实体ID
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("ID:")
                            .color(egui::Color32::from_rgb(150, 150, 150))
                            .size(12.0)
                    );
                    ui.label(
                        egui::RichText::new(format!("{}", label_info.entity_id))
                            .color(egui::Color32::from_rgb(200, 200, 255))
                            .size(12.0)
                    );
                });
                
                // 标签文本
                ui.label(
                    egui::RichText::new(&label_info.text)
                        .color(egui::Color32::WHITE)
                        .size(14.0)
                );
            });
        });
}

/// 标签面板显示系统
/// 
/// 这是一个普通系统，根据 UIStates 的状态决定是否显示面板
pub fn show_label_panel(
    mut contexts: EguiContexts,
    mut ui_states: ResMut<UIStates>,
) {
    // 如果面板未启用，直接返回
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
            // 标题区域
            ui.heading("标签管理");
            ui.separator();

            // 搜索框
            let mut search_text = String::new();
            ui.horizontal(|ui| {
                ui.label("搜索:");
                ui.text_edit_singleline(&mut search_text);
            });
            
            ui.add_space(8.0);

            // 标签列表区域
            ui.group(|ui| {
                ui.set_min_height(200.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // 示例标签列表（后续可从资源中读取实际数据）
                    let sample_labels = vec![
                        ("标签 1", "描述信息 1"),
                        ("标签 2", "描述信息 2"),
                        ("标签 3", "描述信息 3"),
                    ];

                    for (i, (name, desc)) in sample_labels.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // 标签名称
                            ui.label(format!("{}. {}", i + 1, name));
                            
                            // 操作按钮
                            if ui.small_button("编辑").clicked() {
                                // TODO: 编辑标签
                                log::debug!("编辑标签: {}", name);
                            }
                            
                            if ui.small_button("删除").clicked() {
                                // TODO: 删除标签
                                log::debug!("删除标签: {}", name);
                            }
                        });
                        
                        // 标签描述
                        ui.indent("", |ui| {
                            ui.label(format!("   {}", desc));
                        });
                        
                        ui.add_space(4.0);
                    }
                });
            });

            ui.add_space(8.0);

            // 底部操作按钮
            ui.horizontal(|ui| {
                if ui.button("新建标签").clicked() {
                    // TODO: 创建新标签
                    log::debug!("创建新标签");
                }
                
                if ui.button("导入").clicked() {
                    // TODO: 导入标签
                    log::debug!("导入标签");
                }
                
                if ui.button("导出").clicked() {
                    // TODO: 导出标签
                    log::debug!("导出标签");
                }
            });

            ui.add_space(8.0);
            ui.separator();

            // 关闭按钮
            if ui.button("关闭面板").clicked() {
                ui_states.show_label_panel = false;
            }
        });
}