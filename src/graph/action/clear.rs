use bevy::prelude::*;

use crate::graph::action::spawn::SpawnedEntity;

// 定义清除所有对象的消息
#[derive(Message)]
pub struct ClearAllMessage;

// 清除系统，监听清除消息并删除所有带有SpawnedEntity组件的实体
pub fn clear_all_entities(
    mut commands: Commands,
    query: Query<Entity, With<SpawnedEntity>>,
    mut message_reader: MessageReader<ClearAllMessage>,
) {
    // 检查是否有清除消息
    if message_reader.read().count() > 0 {  // 如果有消息被接收
        info!("正在清除场景中的所有对象");
        // 删除所有带有SpawnedEntity标记的实体
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
        info!("清除完成");
    }
}