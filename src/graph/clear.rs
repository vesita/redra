use bevy::prelude::*;

use crate::graph::spawn::SpawnedEntity;

// 清除事件
pub struct ClearAllEvent;

// 清除系统，监听清除事件并删除所有带有SpawnedEntity组件的实体
pub fn clear_all_entities(
    mut commands: Commands,
    query: Query<Entity, With<SpawnedEntity>>,
    mut event_reader: EventReader<ClearAllEvent>,
) {
    // 检查是否有清除事件
    if event_reader.read().count() > 0 {  // 如果有事件被触发
        // 删除所有带有SpawnedEntity标记的实体
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}