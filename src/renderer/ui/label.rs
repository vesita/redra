//! 标签 UI 渲染系统
//! 
//! 负责在 3D 对象附近显示浮动文本标签
//! 通过将 3D 世界坐标投影到 2D 屏幕空间来实现

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::manager::data::frame::FrameManager;

/// 标签 UI 插件
pub struct LabelUiPlugin;

impl Plugin for LabelUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, render_labels);
    }
}

/// 渲染所有标签
fn render_labels(
    mut contexts: EguiContexts,
    frame_manager: Res<FrameManager>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(egui_ctx) = contexts.ctx_mut() else {
        return;
    };

    // 如果没有帧数据，不显示任何标签
    if !frame_manager.has_frames() {
        return;
    }

    // 获取当前关键帧
    let Some(keyframe) = frame_manager.get_current_keyframe() else {
        return;
    };

    // 获取主相机
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // 遍历所有实体并渲染标签
    for (entity_id, inpto) in keyframe.iter_entities() {
        // 只渲染有标签的实体
        let Some(tag) = &inpto.tag else {
            continue;
        };

        // 计算标签的 3D 位置（对象位置 + 偏移）
        let object_position = inpto.transform.translation;
        
        // 如果有偏移，应用偏移
        let label_position = if let Some(offset) = &tag.offset {
            let offset_vec3 = Vec3::new(offset.x, offset.y, offset.z);
            object_position + offset_vec3
        } else {
            // 默认偏移：在对象上方
            object_position + Vec3::new(0.0, 1.0, 0.0)
        };

        // 将 3D 世界坐标转换为 2D 屏幕坐标
        let Some(screen_pos) = world_to_screen(camera, camera_transform, label_position) else {
            continue;
        };

        // 检查标签是否在相机视锥体内
        if !is_in_viewport(camera, screen_pos) {
            continue;
        }

        // 渲染标签
        render_single_label(egui_ctx, screen_pos, tag, entity_id);
    }
}

/// 将 3D 世界坐标转换为 2D 屏幕坐标
fn world_to_screen(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    world_position: Vec3,
) -> Option<Vec2> {
    // 获取视口大小
    let viewport = camera.physical_viewport_rect()?;
    
    // 将世界坐标转换为相机空间
    let camera_world_transform = camera_transform.to_matrix();
    let view_matrix = camera_world_transform.inverse();
    let point_camera_space = view_matrix.transform_point3(world_position);
    
    // 检查点是否在相机前方
    if point_camera_space.z >= 0.0 {
        return None;
    }
    
    // 获取投影矩阵
    let projection_matrix = camera.clip_from_view();
    
    // 应用投影
    let clip_space = projection_matrix * point_camera_space.extend(1.0);
    
    // 透视除法
    if clip_space.w.abs() < f32::EPSILON {
        return None;
    }
    let ndc = clip_space.truncate() / clip_space.w;
    
    // 转换到屏幕空间
    let screen_x = (ndc.x + 1.0) / 2.0 * viewport.width() as f32 + viewport.min.x as f32;
    let screen_y = (1.0 - ndc.y) / 2.0 * viewport.height() as f32 + viewport.min.y as f32;
    
    Some(Vec2::new(screen_x, screen_y))
}

/// 检查屏幕位置是否在视口内
fn is_in_viewport(camera: &Camera, screen_pos: Vec2) -> bool {
    let Some(viewport) = camera.physical_viewport_rect() else {
        return false;
    };
    
    screen_pos.x >= viewport.min.x as f32
        && screen_pos.x <= viewport.max.x as f32
        && screen_pos.y >= viewport.min.y as f32
        && screen_pos.y <= viewport.max.y as f32
}

/// 渲染单个标签
fn render_single_label(
    egui_ctx: &egui::Context,
    screen_pos: Vec2,
    tag: &expto::rdmp::Tag,
    entity_id: u64,
) {
    // 获取样式配置或使用默认值
    let style = tag.style.as_ref();
    
    let font_size = style.map(|s| s.font_size).unwrap_or(14.0);
    let bg_color = egui::Color32::from_rgba_unmultiplied(
        (style.map(|s| s.bg_r).unwrap_or(0.1) * 255.0) as u8,
        (style.map(|s| s.bg_g).unwrap_or(0.1) * 255.0) as u8,
        (style.map(|s| s.bg_b).unwrap_or(0.1) * 255.0) as u8,
        (style.map(|s| s.bg_a).unwrap_or(0.8) * 255.0) as u8,
    );
    let text_color = egui::Color32::from_rgba_unmultiplied(
        (style.map(|s| s.text_r).unwrap_or(1.0) * 255.0) as u8,
        (style.map(|s| s.text_g).unwrap_or(1.0) * 255.0) as u8,
        (style.map(|s| s.text_b).unwrap_or(1.0) * 255.0) as u8,
        (style.map(|s| s.text_a).unwrap_or(1.0) * 255.0) as u8,
    );
    let corner_radius = style.map(|s| s.corner_radius).unwrap_or(4.0) as u8;

    // 使用 egui Area 在指定位置显示标签
    let area_id = egui::Id::new(format!("label_{}", entity_id));
    
    egui::Area::new(area_id)
        .fixed_pos(egui::pos2(screen_pos.x, screen_pos.y))
        .pivot(egui::Align2::CENTER_TOP) // 标签顶部中心对齐到目标点
        .show(egui_ctx, |ui| {
            egui::Frame::new()
                .fill(bg_color)
                .corner_radius(corner_radius)
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&tag.text)
                            .size(font_size)
                            .color(text_color)
                    );
                });
        });
}
