use bevy::prelude::*;
use redra_storage::{FrameStorage, KeyframeManager};
use expto::rdmp::Unit;
use parser::FrameAssembler;
use std::time::Instant;

/// 录制状态资源
#[derive(Resource)]
pub struct RecordingState {
    storage: Option<FrameStorage>,
    keyframe_mgr: KeyframeManager,
    assembler: FrameAssembler,
    is_recording: bool,
    start_time: Instant,
    recorded_frames: u64,
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            storage: None, // 稍后手动初始化
            keyframe_mgr: KeyframeManager::new(),
            assembler: FrameAssembler::new(),
            is_recording: false,
            start_time: Instant::now(),
            recorded_frames: 0,
        }
    }

    /// 检查存储是否已初始化
    pub fn is_storage_initialized(&self) -> bool {
        self.storage.is_some()
    }

    /// 初始化存储（异步）
    pub async fn initialize_storage(&mut self) -> Result<(), String> {
        let storage = FrameStorage::new_default().await?;
        self.storage = Some(storage);
        log::info!("✅ 录制系统存储初始化完成");
        Ok(())
    }

    /// 开始录制
    pub fn start_recording(&mut self) {
        self.is_recording = true;
        self.start_time = Instant::now();
        self.recorded_frames = 0;
        log::info!("🔴 开始录制");
    }

    /// 停止录制
    pub fn stop_recording(&mut self) {
        self.is_recording = false;
        log::info!("⏹️ 停止录制，共录制 {} 帧", self.recorded_frames);
    }

    /// 检查是否正在录制
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// 获取已录制的帧数
    pub fn recorded_frames(&self) -> u64 {
        self.recorded_frames
    }

    /// 录制 Unit 数据
    pub async fn record_unit(&mut self, unit: &Unit) -> Result<(), String> {
        if !self.is_recording {
            return Ok(());
        }

        let storage = self.storage.as_ref()
            .ok_or("存储未初始化")?;

        let frame_id = self.recorded_frames as u32;
        
        // 保存到存储
        storage.save_unit_frame(unit, frame_id).await?;
        
        // 更新关键帧管理器
        if let Some(stamp) = &unit.stamp {
            let current_time = stamp.timestamp as f64 / 1000.0;
            if self.keyframe_mgr.should_create_keyframe(current_time) {
                log::debug!("创建关键帧 at time {}", current_time);
                self.keyframe_mgr.reset_with_time(current_time);
            }
            self.keyframe_mgr.increment_command_count();
        }
        
        self.recorded_frames += 1;
        
        Ok(())
    }

    /// 批量录制 Units 并组装为 FrameData
    pub async fn record_units_batch(&mut self, units: &[Unit]) -> Result<(), String> {
        if !self.is_recording || units.is_empty() {
            return Ok(());
        }

        let storage = self.storage.as_ref()
            .ok_or("存储未初始化")?;

        // 获取时间戳
        let timestamp_ms = units.first()
            .and_then(|u| u.stamp.as_ref())
            .map(|s| s.timestamp)
            .unwrap_or_else(|| {
                self.start_time.elapsed().as_millis() as u64
            });

        // 组装帧数据
        let frame_data = self.assembler.assemble_frame(units, timestamp_ms);
        
        // 保存 FrameData
        storage.save_frame_data(&frame_data).await?;
        
        // 更新统计
        self.recorded_frames += 1;
        
        log::debug!("录制帧 {}: {} units", frame_data.id, units.len());
        
        Ok(())
    }

    /// 获取存储引用
    pub fn storage(&self) -> Option<&FrameStorage> {
        self.storage.as_ref()
    }
}

/// 录制插件
pub struct RecorderPlugin;

impl Plugin for RecorderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RecordingState::new());
    }
}
