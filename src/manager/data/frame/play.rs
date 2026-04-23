use bevy::prelude::*;

use crate::manager::data::frame::FrameManager;

/// 帧播放控制插件
/// 
/// 职责：
/// - 提供帧播放控制（播放/暂停/跳转）
/// - 自动推进帧索引（如果处于播放状态）
/// - 不直接操作实体，由 FrameRenderer 负责渲染
pub struct FramePlaybackPlugin;

impl Plugin for FramePlaybackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, auto_advance_frame);
    }
}

/// 播放状态资源
#[derive(Resource)]
pub struct PlaybackState {
    /// 是否正在播放
    pub is_playing: bool,
    /// 播放速度（帧/秒）
    pub playback_speed: f32,
    /// 累积的时间（用于控制帧率）
    accumulated_time: f32,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: false,
            playback_speed: 30.0, // 默认30 FPS
            accumulated_time: 0.0,
        }
    }
}

impl PlaybackState {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            playback_speed: 30.0, // 默认30 FPS
            accumulated_time: 0.0,
        }
    }

    /// 开始播放
    pub fn play(&mut self) {
        self.is_playing = true;
        log::info!("开始播放");
    }

    /// 暂停播放
    pub fn pause(&mut self) {
        self.is_playing = false;
        log::info!("暂停播放");
    }

    /// 切换播放/暂停状态
    pub fn toggle(&mut self) {
        if self.is_playing {
            self.pause();
        } else {
            self.play();
        }
    }

    /// 设置播放速度
    pub fn set_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(1.0);
        log::info!("播放速度设置为 {} FPS", self.playback_speed);
    }
}

/// 自动推进帧（如果处于播放状态）
fn auto_advance_frame(
    mut frame_manager: ResMut<FrameManager>,
    mut playback_state: ResMut<PlaybackState>,
    time: Res<Time>,
) {
    if !playback_state.is_playing {
        return;
    }

    // 累积时间
    playback_state.accumulated_time += time.delta_secs();

    // 计算帧间隔
    let frame_interval = 1.0 / playback_state.playback_speed;

    // 如果累积时间超过帧间隔，切换到下一帧
    if playback_state.accumulated_time >= frame_interval {
        if frame_manager.next_frame() {
            log::debug!(
                "自动切换到第 {} 帧",
                frame_manager.current_frame_index()
            );
        } else {
            // 到达最后一帧，停止播放或循环
            playback_state.pause();
            log::info!("播放完毕");
        }
        playback_state.accumulated_time = 0.0;
    }
}
