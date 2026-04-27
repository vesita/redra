use bevy::prelude::*;

use crate::data::frame::FrameManager;

/// 帧播放控制插件
pub struct FramePlaybackPlugin;

impl Plugin for FramePlaybackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, auto_advance_frame);
    }
}

/// 播放状态资源
#[derive(Resource)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub playback_speed: f32,
    accumulated_time: f32,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self { is_playing: false, playback_speed: 30.0, accumulated_time: 0.0 }
    }
}

impl PlaybackState {
    pub fn new() -> Self {
        Self { is_playing: false, playback_speed: 30.0, accumulated_time: 0.0 }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
        log::info!("开始播放");
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
        log::info!("暂停播放");
    }

    pub fn toggle(&mut self) {
        if self.is_playing { self.pause(); } else { self.play(); }
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(1.0);
        log::info!("播放速度设置为 {} FPS", self.playback_speed);
    }
}

fn auto_advance_frame(
    mut frame_manager: ResMut<FrameManager>,
    mut playback_state: ResMut<PlaybackState>,
    time: Res<Time>,
) {
    if !playback_state.is_playing { return; }
    playback_state.accumulated_time += time.delta_secs();
    let frame_interval = 1.0 / playback_state.playback_speed;
    if playback_state.accumulated_time >= frame_interval {
        if frame_manager.next_frame() {
            log::debug!("自动切换到第 {} 帧", frame_manager.current_frame_index());
        } else {
            playback_state.pause();
            log::info!("播放完毕");
        }
        playback_state.accumulated_time = 0.0;
    }
}
