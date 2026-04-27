use bevy::prelude::*;

use super::design::HoverLabel;
use crate::manager::data::frame::FrameManager;
use crate::renderer::frame_renderer::EntityMap;
use crate::renderer::interaction::InteractionMessage;


pub fn label_ui_observe(
    im: Res<InteractionMessage>,
    mut hover_label: ResMut<HoverLabel>,
    frame_manager: Res<FrameManager>,
    entity_map: Res<EntityMap>,
    transform_query: Query<&Transform>,
) {
    // 以 InteractionMessage.selected 为唯一真实来源
    let Some(id) = im.selected else {
        // 选中被清空 → 隐藏标签
        if hover_label.current.is_some() {
            hover_label.hide();
        }
        return;
    };

    // 正在显示同一个实体 → 无需更新
    if hover_label.current.as_ref().is_some_and(|c| c.entity_id == id) {
        return;
    }

    let Some(entity) = entity_map.map.get(&id) else {
        return;
    };

    let Ok(transform) = transform_query.get(*entity) else {
        log::warn!("无法获取实体{}的Transform", id);
        return;
    };

    hover_label.show(
        id,
        format_tag_display(&get_tag_from_frame_manager(&frame_manager, id)),
        transform.translation,
    );
}


/// 格式化Tag显示文本
fn format_tag_display(tag: &expto::rdmp::Tag) -> String {
    // Tag只有一个text字段
    tag.text.clone()
}

/// 从 FrameManager 获取实体的 Tag
fn get_tag_from_frame_manager(frame_manager: &FrameManager, entity_id: u64) -> expto::rdmp::Tag {
    // 尝试从当前关键帧获取实体
    if let Some(keyframe) = frame_manager.get_current_keyframe() {
        if let Some(inpto) = keyframe.get_entity(entity_id) {
            if let Some(ref tag) = inpto.tag {
                return tag.clone();
            }
        }
    }
    
    // 默认返回空 Tag
    expto::rdmp::Tag {
        text: String::new(),
        offset: None,
        style: None,
    }
}
