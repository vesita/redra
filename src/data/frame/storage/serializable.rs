//! 序列化转换实现

use crate::data::frame::KeyFrame;
use super::{EntityData, SerializableKeyFrame};

impl From<&KeyFrame> for SerializableKeyFrame {
    fn from(keyframe: &KeyFrame) -> Self {
        let entities = keyframe.iter_entities()
            .map(|(entity_id, inpto)| EntityData {
                entity_id,
                mesh: inpto.mesh.clone().into(),
                material: inpto.material.clone(),
                transform: inpto.transform.into(),
            })
            .collect();
        Self { timestamp: keyframe.timestamp, entities }
    }
}
