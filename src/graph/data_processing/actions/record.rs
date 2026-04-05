use bevy::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use log::{debug, error};
use crate::module::parser::core::RDPack;
use crate::graph::communicate::channels::RDChannel;
use redra_storage::{FrameMetadata, FrameType, FrameStorage};

/// 序列化点云数据为二进制格式
fn serialize_point_cloud(packs: &[RDPack]) -> Vec<u8> {
    // 简单的二进制序列化（稍后可以用 bincode 或 protobuf 改进）
    let mut buffer = Vec::new();
    
    // 写入点的数量
    buffer.extend_from_slice(&(packs.len() as u32).to_le_bytes());
    
    // 写入每个包（简化版，生产环境中应使用 bincode）
    for pack in packs {
        match pack {
            RDPack::Message(msg) => {
                buffer.push(0);  // 类型标记
                buffer.extend_from_slice(&(msg.len() as u32).to_le_bytes());
                buffer.extend_from_slice(msg.as_bytes());
            },
            RDPack::SpawnShape(_) => {
                buffer.push(1);  // 类型标记
                // TODO: 实现 Shape 序列化
            },
            RDPack::SpawnFormat(_) => {
                buffer.push(2);  // 类型标记
                // TODO: 实现 Format 序列化
            },
        }
    }
    
    buffer
}

/// 反序列化点云数据
fn deserialize_point_cloud(buffer: &[u8]) -> Result<Vec<RDPack>, Box<dyn std::error::Error>> {
    let mut offset = 0;
    
    // 读取点的数量
    if buffer.len() < 4 {
        return Err("缓冲区太短".into());
    }
    let num_points = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    offset += 4;
    
    let mut packs = Vec::with_capacity(num_points as usize);
    
    for _ in 0..num_points {
        if offset >= buffer.len() {
            return Err("意外的缓冲区结束".into());
        }
        
        let pack_type = buffer[offset];
        offset += 1;
        
        match pack_type {
            0 => {
                // RDPack::Message
                if offset + 4 > buffer.len() {
                    return Err("意外的缓冲区结束".into());
                }
                let msg_len = u32::from_le_bytes([
                    buffer[offset],
                    buffer[offset + 1],
                    buffer[offset + 2],
                    buffer[offset + 3],
                ]) as usize;
                offset += 4;
                
                if offset + msg_len > buffer.len() {
                    return Err("意外的缓冲区结束".into());
                }
                
                let msg = String::from_utf8(buffer[offset..offset + msg_len].to_vec())?;
                offset += msg_len;
                
                packs.push(RDPack::Message(msg));
            }
            1 => {
                // RDPack::SpawnShape - 尚未实现
                return Err("SpawnShape 反序列化尚未实现".into());
            }
            2 => {
                // RDPack::SpawnFormat - 尚未实现
                return Err("SpawnFormat 反序列化尚未实现".into());
            }
            _ => {
                return Err("未知的包类型".into());
            }
        }
    }
    
    Ok(packs)
}

/// 数据帧 - 带有完整元数据的帧结构
#[derive(Clone, Debug)]
pub struct DataFrame {
    pub frame_id: u32,              // 帧 ID
    pub sequence_number: u64,       // 全局序列号
    pub timestamp: u64,             // 时间戳（毫秒）
    pub points: Vec<RDPack>,        // 帧包含的所有点/形状数据
    pub is_complete: bool,          // 帧是否完整
    pub frame_type: FrameType,      // 帧类型（来自 redra_storage）
}

/// 帧构建器 - 用于累积点数据直到帧完成
#[derive(Debug)]
pub struct FrameBuilder {
    pub frame_id: u32,
    pub start_time: u64,
    pub points: Vec<RDPack>,
    pub expected_points: Option<u32>,  // 如果知道总点数
}

impl FrameBuilder {
    pub fn new(frame_id: u32) -> Self {
        Self {
            frame_id,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            points: Vec::new(),
            expected_points: None,
        }
    }

    pub fn add_point(&mut self, pack: RDPack) {
        self.points.push(pack);
    }

    pub fn is_complete(&self) -> bool {
        match self.expected_points {
            Some(expected) => self.points.len() >= expected as usize,
            None => false,  // 如果没有指定期望点数，需要显式标记完成
        }
    }

    pub fn build(self) -> DataFrame {
        let timestamp = self.start_time;
        DataFrame {
            frame_id: self.frame_id,
            sequence_number: 0,  // 将由 Recorder 设置
            timestamp,
            points: self.points,
            is_complete: true,
            frame_type: FrameType::PCD,  // 使用 redra_storage 的 FrameType
        }
    }
}

/// 记录器资源 - 管理数据帧的录制
#[derive(Resource)]
pub struct DataRecorder {
    pub frames: Vec<DataFrame>,     // 已记录的完整帧（内存缓存）
    pub current_builder: Option<FrameBuilder>,  // 当前正在构建的帧
    pub is_recording: bool,         // 是否正在录制
    pub current_sequence: u64,      // 当前序列号
    pub current_frame_id: u32,      // 当前帧 ID
    pub recording_start_time: u64,  // 录制开始时间
    pub total_points_received: u64, // 接收到的总点数
    
    // SQLite 持久化存储（使用 Arc<Mutex<>> 保证线程安全）
    pub storage: Option<Arc<Mutex<FrameStorage>>>,
    
    // 待持久化的帧缓冲区
    pub pending_persistence: Vec<DataFrame>,
}

impl Default for DataRecorder {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        Self {
            frames: Vec::new(),
            current_builder: None,
            is_recording: true,
            current_sequence: 0,
            current_frame_id: 0,
            recording_start_time: now,
            total_points_received: 0,
            storage: None,  // 将由 DataProcessingPlugin 初始化
            pending_persistence: Vec::new(),
        }
    }
}

impl DataRecorder {
    /// 创建新的记录器
    pub fn new() -> Self {
        Self::default()
    }

    /// 开始新帧
    pub fn start_frame(&mut self, frame_id: u32, expected_points: Option<u32>) {
        let mut builder = FrameBuilder::new(frame_id);
        builder.expected_points = expected_points;
        self.current_builder = Some(builder);
        self.current_frame_id = frame_id;
        debug!("开始记录帧 #{}", frame_id);
    }

    /// 添加点到当前帧
    pub fn add_point_to_frame(&mut self, pack: RDPack) {
        if !self.is_recording {
            return;
        }

        self.total_points_received += 1;

        // 如果没有活动的帧，创建一个
        if self.current_builder.is_none() {
            self.start_frame(self.current_builder.as_ref().map(|b| b.frame_id).unwrap_or(self.current_frame_id), None);
        }

        if let Some(ref mut builder) = self.current_builder {
            builder.add_point(pack);
            
            // 检查帧是否完成
            if builder.is_complete() {
                self.finish_current_frame();
            }
        }
    }

    /// 完成当前帧
    pub fn finish_current_frame(&mut self) {
        let frame_id = self.current_builder.as_ref().map(|b| b.frame_id);
        
        if let Some(builder) = self.current_builder.take() {
            let mut frame = builder.build();
            frame.sequence_number = self.current_sequence;
            
            if let Some(id) = frame_id {
                debug!(
                    "完成帧 #{} (序列号：{}), 点数：{}",
                    id,
                    frame.sequence_number,
                    frame.points.len()
                );
            }
            
            // 持久化到 SQLite（如果有存储）
            if let Some(ref storage_arc) = self.storage {
                let buffer = serialize_point_cloud(&frame.points);
                
                let metadata = FrameMetadata {
                    frame_id: frame.frame_id,
                    sequence_number: frame.sequence_number,
                    timestamp: frame.timestamp,
                    point_count: frame.points.len() as u32,
                    frame_type: frame.frame_type.to_string(),
                    data_path: String::new(),
                };
                
                let storage = storage_arc.lock().unwrap();
                if let Err(e) = storage.save_frame(&buffer, metadata) {
                    error!("持久化帧 {} 失败: {}", frame_id.unwrap_or(0), e);
                }
            } else {
                // 内存模式
                self.frames.push(frame);
            }

            self.current_sequence += 1;
            self.current_frame_id += 1;
        }
    }

    /// 强制完成当前帧（即使点数未达到期望）
    pub fn force_finish_frame(&mut self) {
        if let Some(builder) = self.current_builder.take() {
            if !builder.points.is_empty() {
                let mut frame = builder.build();
                frame.sequence_number = self.current_sequence;
                
                debug!(
                    "强制完成帧 #{} (序列号：{}), 点数：{}",
                    frame.frame_id,
                    frame.sequence_number,
                    frame.points.len()
                );

                // 持久化到 SQLite（如果有存储）
                if let Some(ref storage_arc) = self.storage {
                    let buffer = serialize_point_cloud(&frame.points);
                    
                    let metadata = FrameMetadata {
                        frame_id: frame.frame_id,
                        sequence_number: frame.sequence_number,
                        timestamp: frame.timestamp,
                        point_count: frame.points.len() as u32,
                        frame_type: frame.frame_type.to_string(),
                        data_path: String::new(),
                    };
                    
                    let storage = storage_arc.lock().unwrap();
                    if let Err(e) = storage.save_frame(&buffer, metadata) {
                        error!("持久化帧 {} 失败: {}", frame.frame_id, e);
                    }
                } else {
                    self.frames.push(frame);
                }

                self.current_sequence += 1;
                self.current_frame_id += 1;
            }
        }
    }

    /// 清除所有记录的帧
    pub fn clear(&mut self) {
        self.frames.clear();
        self.current_builder = None;
        self.current_sequence = 0;
        self.current_frame_id = 0;
        self.recording_start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.total_points_received = 0;
    }

    /// 获取总帧数
    pub fn total_frames(&self) -> usize {
        if let Some(ref storage_arc) = self.storage {
            if let Ok(storage) = storage_arc.lock() {
                if let Ok(stats) = storage.database().get_stats() {
                    return stats.total_frames as usize;
                }
            }
        }
        self.frames.len()
    }

    /// 获取总点数
    pub fn total_points(&self) -> u64 {
        self.total_points_received
    }

    /// 获取录制时长（秒）
    pub fn recording_duration(&self) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        ((now - self.recording_start_time) as f64) / 1000.0
    }
}

/// 回放管理器资源 - 控制数据帧的回放
#[derive(Resource)]
pub struct PlaybackManager {
    pub is_playing: bool,           // 是否正在播放
    pub playback_speed: f32,        // 播放速度（1.0 = 正常速度）
    pub current_frame_index: usize, // 当前帧索引
    pub last_update_time: u64,      // 上次更新时间
    pub frame_interval_ms: u64,     // 帧间隔（毫秒）
    pub loop_playback: bool,        // 是否循环播放
    
    // 回放时临时加载的帧（从 SQLite 加载）
    pub loaded_frame: Option<Vec<RDPack>>,
}

impl Default for PlaybackManager {
    fn default() -> Self {
        Self {
            is_playing: false,
            playback_speed: 1.0,
            current_frame_index: 0,
            last_update_time: 0,
            frame_interval_ms: 33, // 默认约 30FPS
            loop_playback: false,
            loaded_frame: None,
        }
    }
}

impl PlaybackManager {
    /// 创建新的回放管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 开始播放
    pub fn play(&mut self) {
        self.is_playing = true;
        self.last_update_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }

    /// 暂停播放
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// 停止播放并重置到开始
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_frame_index = 0;
        self.loaded_frame = None;
    }

    /// 设置播放速度
    pub fn set_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.1).min(10.0);
    }

    /// 跳转到指定帧
    pub fn seek_to(&mut self, frame_index: usize) {
        self.current_frame_index = frame_index;
    }

    /// 前进一帧
    pub fn next_frame(&mut self, total_frames: usize) {
        if self.current_frame_index < total_frames.saturating_sub(1) {
            self.current_frame_index += 1;
        } else if self.loop_playback {
            self.current_frame_index = 0;
        }
    }

    /// 后退一帧
    pub fn previous_frame(&mut self) {
        if self.current_frame_index > 0 {
            self.current_frame_index -= 1;
        }
    }
}

/// 从存储加载指定帧
pub fn load_frame_from_storage(
    _storage: &FrameStorage,
    _frame_index: usize,
) -> Result<Vec<RDPack>, Box<dyn std::error::Error>> {
    // 因为 redra_storage 中可能没有 get_frame_by_index 方法，我们尝试使用其他方法
    // 这里暂时返回错误，需要根据实际的 redra_storage API 来实现
    unimplemented!("需要根据 redra_storage 的实际 API 来实现此函数")
}

/// 记录数据帧系统
/// 从 channel 接收数据并按照帧结构组织
pub fn record_data_frames(
    mut recorder: ResMut<DataRecorder>,
    mut channel: ResMut<RDChannel>,
) {
    if !recorder.is_recording {
        return;
    }

    // 接收所有可用的数据包并记录到当前帧
    while let Ok(pack) = channel.receiver.try_recv() {
        recorder.add_point_to_frame(pack);
    }
}