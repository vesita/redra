use bevy::prelude::*;

use super::design::{HoverLabel, TagEditResult};
use crate::data::frame::FrameManager;
use crate::render::frame_renderer::EntityMap;
use crate::render::interaction::InteractionMessage;

pub fn label_ui_observe(
    im: Res<InteractionMessage>,
    mut hover_label: ResMut<HoverLabel>,
    frame_manager: Res<FrameManager>,
    entity_map: Res<EntityMap>,
    transform_query: Query<&Transform>,
) {
    let Some(id) = im.selected else {
        if hover_label.current.is_some() { hover_label.hide(); }
        return;
    };

    if hover_label.current.as_ref().is_some_and(|c| c.entity_id == id) { return; }

    let Some(entity) = entity_map.map.get(&id) else { return; };
    let Ok(transform) = transform_query.get(*entity) else { return; };

    hover_label.show(id, format_tag_display(&get_tag_from_frame_manager(&frame_manager, id)), transform.translation);
}

/// 处理 Tag 编辑结果，写入 FrameManager 并刷新 hover label
pub fn apply_tag_edit(
    mut edit_result: ResMut<TagEditResult>,
    mut frame_manager: ResMut<FrameManager>,
    mut hover_label: ResMut<HoverLabel>,
    entity_map: Res<EntityMap>,
    transform_query: Query<&Transform>,
) {
    let Some((entity_id, new_text)) = edit_result.pending.take() else { return };

    if let Some(kf) = frame_manager.get_current_keyframe_mut() {
        kf.update_entity_tag(entity_id, new_text.clone());
        log::info!("实体 {} 的 Tag 已更新为: {}", entity_id, new_text);
    }

    // 刷新 hover label 显示
    if let Some(entity) = entity_map.map.get(&entity_id) {
        if let Ok(transform) = transform_query.get(*entity) {
            hover_label.show(entity_id, new_text, transform.translation);
        }
    }
}

fn format_tag_display(tag: &expto::rdmp::Tag) -> String { tag.text.clone() }

fn get_tag_from_frame_manager(frame_manager: &FrameManager, entity_id: u64) -> expto::rdmp::Tag {
    if let Some(keyframe) = frame_manager.get_current_keyframe() {
        if let Some(inpto) = keyframe.get_entity(entity_id) {
            if let Some(ref tag) = inpto.tag { return tag.clone(); }
        }
    }
    expto::rdmp::Tag { text: String::new(), offset: None, style: None }
}
