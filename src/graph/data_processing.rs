use bevy::prelude::*;

// 导入模块
pub mod actions;
pub mod entities;

// 定义数据处理插件
pub struct DataProcessingPlugin;

impl Plugin for DataProcessingPlugin {
    fn build(&self, app: &mut App) {
        // 注册资源
        app
            .init_resource::<actions::record::DataRecorder>()
            .init_resource::<actions::record::PlaybackManager>()
            .add_systems(Update, (
                actions::record::record_data_frames,
                update_playback.after(actions::record::record_data_frames),
            ));
    }
}

// 更新回放系统
fn update_playback(
    mut playback: ResMut<actions::record::PlaybackManager>,
    recorder: Res<actions::record::DataRecorder>,
    _time: Res<Time>,
) {
    if !playback.is_playing {
        return;
    }

    // 计算自上次更新以来的时间
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let elapsed = now - playback.last_update_time;
    let interval = (playback.frame_interval_ms as f32 / playback.playback_speed) as u64;

    if elapsed >= interval {
        // 计算应该前进多少帧
        let frames_to_advance = (elapsed / interval) as usize;
        
        for _ in 0..frames_to_advance {
            if playback.current_frame_index < recorder.frames.len().saturating_sub(1) {
                playback.current_frame_index += 1;
            } else if playback.loop_playback {
                playback.current_frame_index = 0;
            } else {
                // 播放到结尾，暂停播放
                playback.is_playing = false;
                break;
            }
        }

        // 更新最后更新时间
        playback.last_update_time = now - (elapsed % interval);
    }
}