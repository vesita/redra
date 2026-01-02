use bevy::{ecs::{message::MessageWriter, system::{Local, Res}}, input::{ButtonInput, keyboard::KeyCode}};
use log::info;

use crate::module::camera::fps::{ChangeCursorModeMessage, CursorToggleMode};



/// 切换相机模式的系统
/// 按下 'M' 键可以在 Trigger 和 Flip 模式之间切换相机模式
pub fn toggle_camera_mode_system(
    mut message_writer: MessageWriter<ChangeCursorModeMessage>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_mode: Local<CursorToggleMode>
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        // 切换模式
        let new_mode = match *current_mode {
            CursorToggleMode::Trigger => CursorToggleMode::Flip,
            CursorToggleMode::Flip => CursorToggleMode::Trigger,
        };
        
        // 更新本地存储的模式
        *current_mode = new_mode;
        
        // 发送消息更改相机模式
        message_writer.write(ChangeCursorModeMessage {
            mode: new_mode,
            camera_entity: None, // 应用到所有启用的相机
        });
        
        // 打印当前模式到控制台
        match new_mode {
            CursorToggleMode::Trigger => info!("相机模式已切换为 Trigger"),
            CursorToggleMode::Flip => info!("相机模式已切换为 Flip"),
        }
    }
}