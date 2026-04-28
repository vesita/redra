use bevy::prelude::*;

use super::design::HoverLabel;
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

fn format_tag_display(tag: &expto::rdmp::Tag) -> String { tag.text.clone() }

fn get_tag_from_frame_manager(frame_manager: &FrameManager, entity_id: u64) -> expto::rdmp::Tag {
    if let Some(keyframe) = frame_manager.get_current_keyframe() {
        if let Some(inpto) = keyframe.get_entity(entity_id) {
            if let Some(ref tag) = inpto.tag { return tag.clone(); }
        }
    }
    expto::rdmp::Tag { text: String::new(), offset: None, style: None }
}
