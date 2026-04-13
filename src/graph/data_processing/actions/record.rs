use bevy::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use log::{debug, error};
use crate::module::parser::core::{RDPack, PointCloudPack};
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
            RDPack::PointCloud(point_cloud) => {
                buffer.push(3);  // 类型标记
                buffer.extend_from_slice(&point_cloud.frame_id.to_le_bytes());
                buffer.extend_from_slice(&point_cloud.timestamp.to_le_bytes());
                buffer.extend_from_slice(&(point_cloud.points.len() as u32).to_le_bytes());
                for &(x, y, z) in &point_cloud.points {
                    buffer.extend_from_slice(&x.to_le_bytes());
                    buffer.extend_from_slice(&y.to_le_bytes());
                    buffer.extend_from_slice(&z.to_le_bytes());
                }
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
            3 => {
                // RDPack::PointCloud
                if offset + 12 > buffer.len() {
                    return Err("缓冲区太短，无法读取 PointCloud 头部".into());
                }
                
                let frame_id = u32::from_le_bytes([
                    buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]
                ]);
                offset += 4;
                
                let timestamp = u64::from_le_bytes([
                    buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3],
                    buffer[offset + 4], buffer[offset + 5], buffer[offset + 6], buffer[offset + 7]
                ]);
                offset += 8;
                
                let point_count = u32::from_le_bytes([
                    buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]
                ]) as usize;
                offset += 4;
                
                if offset + point_count * 12 > buffer.len() {
                    return Err("缓冲区太短，无法读取点数据".into());
                }
                
                let mut points = Vec::with_capacity(point_count);
                for _ in 0..point_count {
                    let x = f32::from_le_bytes([
                        buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]
                    ]);
                    offset += 4;
                    
                    let y = f32::from_le_bytes([
                        buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]
                    ]);
                    offset += 4;
                    
                    let z = f32::from_le_bytes([
                        buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]
                    ]);
                    offset += 4;
                    
                    points.push((x, y, z));
                }
                
                packs.push(RDPack::PointCloud(PointCloudPack {
                    frame_id,
                    timestamp,
                    points,
                }));
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

/// 录制模式枚举
#[derive(Debug, Clone, PartialEq)]
pub enum RecordingMode {
    Off,                    // 关闭录制
    MemoryOnly,            // 仅内存（不保存到磁盘）
    AutoSave,              // 自动保存到 SQLite
}

/// 数据记录器 - 负责接收和存储点云数据帧
#[derive(Resource)]
pub struct DataRecorder {
    pub frames: Vec<DataFrame>,     // 已记录的完整帧（内存缓存）
    pub current_builder: Option<FrameBuilder>,  // 当前正在构建的帧
    pub recording_mode: RecordingMode,  // 录制模式
    pub current_sequence: u64,      // 当前序列号
    pub current_frame_id: u32,      // 当前帧 ID
    pub recording_start_time: u64,  // 录制开始时间
    pub total_points_received: u64, // 接收到的总点数
    
    // SQLite 持久化存储（使用 Arc<Mutex<>> 保证线程安全）
    pub storage: Option<Arc<Mutex<FrameStorage>>>,
    
    // 待持久化的帧缓冲区（仅在 MemoryOnly 模式下使用）
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
            recording_mode: RecordingMode::Off,  // 默认关闭录制
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
        // 如果录制关闭，直接忽略
        if self.recording_mode == RecordingMode::Off {
            return;
        }

        // 根据 RDPack 类型处理
        match pack {
            RDPack::PointCloud(point_cloud) => {
                // 处理点云数据包
                let point_count = point_cloud.points.len();
                self.total_points_received += point_count as u64;
                
                log::debug!("接收到点云数据包 - 帧ID: {}, 点数: {}", 
                       point_cloud.frame_id, point_count);
                
                // 如果当前没有活动的帧，或者帧ID不同，创建新帧
                let should_start_new_frame = match &self.current_builder {
                    None => true,
                    Some(builder) => builder.frame_id != point_cloud.frame_id,
                };
                
                if should_start_new_frame {
                    // 完成当前帧（如果有）
                    if self.current_builder.is_some() {
                        self.finish_current_frame();
                    }
                    // 开始新帧
                    self.start_frame(point_cloud.frame_id, Some(point_count as u32));
                }
                
                // 添加点到当前帧 - 直接存储 PointCloud 结构
                if let Some(ref mut builder) = self.current_builder {
                    // 将整个点云包作为一个单元添加
                    builder.add_point(RDPack::PointCloud(point_cloud.clone()));
                    
                    // 检查帧是否完成
                    if builder.is_complete() {
                        self.finish_current_frame();
                    }
                }
            }
            RDPack::SpawnShape(_) | RDPack::SpawnFormat(_) | RDPack::Message(_) => {
                // ✅ 修复：处理非点云数据包（SpawnShape, SpawnFormat, Message）
                log::debug!("📦 接收到非点云数据包: {:?}", pack);
                
                // 如果没有当前帧构建器，创建一个新帧
                if self.current_builder.is_none() {
                    let frame_id = self.current_frame_id;
                    self.current_frame_id += 1;
                    log::info!("🆕 为形状数据创建新帧 #{}", frame_id);
                    self.start_frame(frame_id, None);
                }
                
                // 添加到当前帧
                if let Some(ref mut builder) = self.current_builder {
                    // 将整个包作为一个单元添加
                    builder.add_point(pack.clone());
                    
                    // 检查帧是否完成（对于形状数据，可以立即完成）
                    if builder.is_complete() {
                        self.finish_current_frame();
                    }
                }
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
                log::debug!(
                    "完成帧 #{} (序列号：{}), 点数：{}",
                    id,
                    frame.sequence_number,
                    frame.points.len()
                );
            }
            
            // 根据录制模式处理
            match self.recording_mode {
                RecordingMode::AutoSave => {
                    // 自动保存到 SQLite
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
                            log::error!("持久化帧 {} 失败: {}", frame.frame_id, e);
                        }
                        
                        // 保留一份在内存中用于 UI 显示和回放
                        self.frames.push(frame.clone());
                    } else {
                        log::warn!("⚠️ AutoSave 模式但未初始化存储，降级为 MemoryOnly");
                        self.frames.push(frame);
                    }
                }
                RecordingMode::MemoryOnly => {
                    // 仅保存在内存中
                    self.frames.push(frame);
                }
                RecordingMode::Off => {
                    // 不应该到达这里，因为 add_point_to_frame 已经检查过
                    log::warn!("⚠️ 录制关闭但仍然尝试完成帧");
                }
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
                
                log::debug!(
                    "强制完成帧 #{} (序列号：{}), 点数：{}",
                    frame.frame_id,
                    frame.sequence_number,
                    frame.points.len()
                );

                // 根据录制模式处理
                match self.recording_mode {
                    RecordingMode::AutoSave => {
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
                                log::error!("持久化帧 {} 失败: {}", frame.frame_id, e);
                            }
                            
                            self.frames.push(frame.clone());
                        } else {
                            self.frames.push(frame);
                        }
                    }
                    RecordingMode::MemoryOnly => {
                        self.frames.push(frame);
                    }
                    RecordingMode::Off => {
                        log::warn!("⚠️ 录制关闭但仍然尝试强制完成帧");
                    }
                }

                self.current_sequence += 1;
                self.current_frame_id += 1;
            }
        }
    }

    /// 清除所有记录的帧（内存和数据库）
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
        
        // 如果启用了存储，也清空数据库
        if let Some(ref storage_arc) = self.storage {
            if let Ok(mut storage) = storage_arc.lock() {
                if let Err(e) = storage.database().clear_all_frames() {
                    log::error!("清空数据库失败: {}", e);
                } else {
                    log::info!("✅ 已清空数据库中的所有帧");
                }
            }
        }
        
        log::info!("✅ 已清空所有录制的帧数据");
    }

    /// 手动保存当前内存中的帧到 SQLite
    pub fn save_to_disk(&mut self) -> Result<usize, String> {
        if self.frames.is_empty() {
            return Err("没有可保存的帧".to_string());
        }
        
        if self.storage.is_none() {
            return Err("未初始化 SQLite 存储".to_string());
        }
        
        let storage_arc = self.storage.as_ref().unwrap();
        let mut saved_count = 0;
        
        {
            let storage = storage_arc.lock().map_err(|e| format!("锁定存储失败: {}", e))?;
            
            for frame in &self.frames {
                let buffer = serialize_point_cloud(&frame.points);
                
                let metadata = FrameMetadata {
                    frame_id: frame.frame_id,
                    sequence_number: frame.sequence_number,
                    timestamp: frame.timestamp,
                    point_count: frame.points.len() as u32,
                    frame_type: frame.frame_type.to_string(),
                    data_path: String::new(),
                };
                
                if let Err(e) = storage.save_frame(&buffer, metadata) {
                    log::error!("保存帧 {} 失败: {}", frame.frame_id, e);
                } else {
                    saved_count += 1;
                }
            }
        }
        
        log::info!("✅ 成功保存 {} 帧到磁盘", saved_count);
        Ok(saved_count)
    }

    /// 设置录制模式
    pub fn set_recording_mode(&mut self, mode: RecordingMode) {
        let old_mode = self.recording_mode.clone();
        self.recording_mode = mode;
        
        log::info!("📹 录制模式切换: {:?} → {:?}", old_mode, self.recording_mode);
        
        // 如果切换到关闭模式，完成当前正在构建的帧
        if self.recording_mode == RecordingMode::Off {
            if self.current_builder.is_some() {
                log::info!("⚠️ 录制关闭，丢弃当前正在构建的帧");
                self.current_builder = None;
            }
        }
    }

    /// 获取当前录制模式
    pub fn get_recording_mode(&self) -> &RecordingMode {
        &self.recording_mode
    }

    /// 获取总帧数（优先使用内存中的数据）
    pub fn total_frames(&self) -> usize {
        // 优先返回内存中的帧数（更实时、更准确）
        let memory_count = self.frames.len();
        
        if memory_count > 0 {
            return memory_count;
        }
        
        // 如果内存中没有，再查询数据库
        if let Some(ref storage_arc) = self.storage {
            if let Ok(storage) = storage_arc.lock() {
                if let Ok(stats) = storage.database().get_stats() {
                    return stats.total_frames as usize;
                }
            }
        }
        
        0
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

    /// 获取帧元数据列表（用于 UI 显示）
    /// 
    /// 返回格式: Vec<(frame_id, sequence_number, timestamp, point_count, frame_type)>
    /// 支持 SQLite 和内存双模式
    pub fn get_frame_metadata_list(&self) -> Vec<(u32, u64, u64, u32, String)> {
        // 优先从 SQLite 数据库获取（如果可用）
        if let Some(ref storage_arc) = self.storage {
            if let Ok(storage) = storage_arc.lock() {
                if let Ok(frames) = storage.database().get_all_frames() {
                    return frames.iter().map(|f| {
                        (f.frame_id, f.sequence_number, f.timestamp, f.point_count, f.frame_type.clone())
                    }).collect();
                }
            }
        }
        
        // 回退到内存模式
        self.frames.iter().map(|f| {
            (f.frame_id, f.sequence_number, f.timestamp, f.points.len() as u32, f.frame_type.to_string())
        }).collect()
    }

    /// 根据搜索条件过滤帧元数据
    /// 
    /// # 参数
    /// * `filter` - 搜索关键词，匹配 frame_id 或 frame_type
    /// 
    /// # 返回
    /// 过滤后的帧元数据列表
    pub fn search_frames(&self, filter: &str) -> Vec<(u32, u64, u64, u32, String)> {
        let all_frames = self.get_frame_metadata_list();
        
        if filter.is_empty() {
            return all_frames;
        }
        
        let filter_lower = filter.to_lowercase();
        all_frames.into_iter().filter(|(frame_id, _, _, _, frame_type)| {
            // 匹配 frame_id
            frame_id.to_string().contains(&filter_lower) ||
            // 匹配 frame_type
            frame_type.to_lowercase().contains(&filter_lower)
        }).collect()
    }

    /// 分页获取帧元数据
    /// 
    /// # 参数
    /// * `page` - 页码（从 0 开始）
    /// * `page_size` - 每页数量
    /// 
    /// # 返回
    /// (当前页的帧列表, 总页数)
    pub fn get_frames_paginated(
        &self,
        page: usize,
        page_size: usize,
    ) -> (Vec<(u32, u64, u64, u32, String)>, usize) {
        let all_frames = self.get_frame_metadata_list();
        let total_frames = all_frames.len();
        let total_pages = (total_frames + page_size - 1) / page_size.max(1);
        
        let start = page * page_size;
        let end = (start + page_size).min(total_frames);
        
        let page_frames = if start < total_frames {
            all_frames[start..end].to_vec()
        } else {
            Vec::new()
        };
        
        (page_frames, total_pages)
    }

    /// 获取内存中的帧数
    pub fn memory_frame_count(&self) -> usize {
        self.frames.len()
    }

    /// 获取数据库中的帧数
    pub fn database_frame_count(&self) -> usize {
        if let Some(ref storage_arc) = self.storage {
            if let Ok(storage) = storage_arc.lock() {
                if let Ok(stats) = storage.database().get_stats() {
                    return stats.total_frames as usize;
                }
            }
        }
        0
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
    pub manual_frame_change: bool,  // 用户是否手动切换了帧（用于暂停时渲染和防止自动跳帧）
    
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
            manual_frame_change: false,
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
        // 注意：不在这里设置 last_update_time，让 replay_data_frames 系统首次运行时自动初始化
        // 这样可以避免使用不同的时间系统（SystemTime vs Time.elapsed）
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
        self.manual_frame_change = true;  // 标记为手动切换
    }

    /// 前进一帧
    pub fn next_frame(&mut self, total_frames: usize) {
        if self.current_frame_index < total_frames.saturating_sub(1) {
            self.current_frame_index += 1;
        } else if self.loop_playback {
            self.current_frame_index = 0;
        }
        self.manual_frame_change = true;  // 标记为手动切换
    }

    /// 后退一帧
    pub fn previous_frame(&mut self) {
        if self.current_frame_index > 0 {
            self.current_frame_index -= 1;
        }
        self.manual_frame_change = true;  // 标记为手动切换
    }
    
    /// 清除手动切换标志（由回放系统在渲染后调用）
    pub fn clear_manual_flag(&mut self) {
        self.manual_frame_change = false;
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
    // 如果录制关闭，直接返回
    if recorder.recording_mode == RecordingMode::Off {
        return;
    }

    // 接收所有可用的数据包并记录到当前帧
    let mut received_count = 0;
    while let Ok(pack) = channel.receiver.try_recv() {
        log::debug!("📥 接收到数据包");
        recorder.add_point_to_frame(pack);
        received_count += 1;
    }
    
    if received_count > 0 {
        log::info!("✅ 本帧接收 {} 个数据包，总计接收 {} 个点，内存中 {} 帧", 
               received_count, 
               recorder.total_points_received,
               recorder.memory_frame_count());
    }
}

/// 回放标记组件 - 标记由回放系统生成的实体
#[derive(Component)]
pub struct ReplayedEntity {
    pub frame_index: usize,
}

/// 回放数据帧系统
/// 根据 PlaybackManager 的当前帧索引，从 DataRecorder 获取数据并生成实体
pub fn replay_data_frames(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_manager: Res<crate::graph::materials::MaterialManager>,
    recorder: Res<DataRecorder>,
    mut playback: ResMut<PlaybackManager>,
    existing_entities: Query<Entity, With<ReplayedEntity>>,
    time: Res<Time>,
) {
    // ✅ 修复1：如果既不在播放状态，也没有手动切换帧，则不处理
    if !playback.is_playing && !playback.manual_frame_change {
        return;
    }

    let total_frames = recorder.total_frames();
    
    log::info!("🔄 回放系统检查 - is_playing={}, manual_change={}, total_frames={}, current_index={}", 
               playback.is_playing, playback.manual_frame_change, total_frames, playback.current_frame_index);
    
    if total_frames == 0 {
        log::warn!("⚠️ 没有可回放的帧数据");
        playback.clear_manual_flag();  // 清除标志
        return;
    }

    // ✅ 修复2：如果是手动切换帧（无论是否暂停），立即渲染
    let should_render = if playback.manual_frame_change {
        log::info!("👆 检测到手动帧切换，立即渲染帧 #{}", playback.current_frame_index);
        true
    } else {
        // 否则，检查时间间隔（仅在播放状态下）
        if !playback.is_playing {
            return;
        }
        
        let current_time_ms = (time.elapsed_secs() * 1000.0) as u64;
        let adjusted_interval = (playback.frame_interval_ms as f32 / playback.playback_speed) as u64;
        
        log::debug!("⏱️ 时间检查 - current_time={}, last_update={}, interval={}, speed={}", 
                    current_time_ms, playback.last_update_time, adjusted_interval, playback.playback_speed);
        
        // 首次播放时初始化时间戳，并立即渲染第一帧
        if playback.last_update_time == 0 {
            playback.last_update_time = current_time_ms.saturating_sub(adjusted_interval);
            log::info!("⏱️ 初始化播放时间戳，立即渲染第一帧");
            true
        } else {
            // 使用 saturating_sub 防止下溢
            let elapsed = current_time_ms.saturating_sub(playback.last_update_time);
            
            log::debug!("⏱️ 经过时间: {} ms, 需要间隔: {} ms", elapsed, adjusted_interval);
            
            if elapsed < adjusted_interval {
                log::debug!("⏸️ 时间未到，跳过本帧");
                return;
            }
            
            log::info!("⏰ 时间到达，准备渲染帧 #{}", playback.current_frame_index);
            true
        }
    };
    
    if !should_render {
        return;
    }

    // 清除上一帧的所有实体
    let cleared_count = existing_entities.iter().count();
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn();
    }
    if cleared_count > 0 {
        log::debug!("🗑️ 清除了 {} 个旧实体", cleared_count);
    }

    // 获取当前帧的数据
    let frame_data = get_current_frame_data(&recorder, playback.current_frame_index);
    
    if let Some(frame_packs) = frame_data {
        log::info!(
            "🎬 回放帧 #{} / {}, 包含 {} 个数据包",
            playback.current_frame_index,
            total_frames,
            frame_packs.len()
        );

        // 为每个 RDPack 生成实体
        for (pack_idx, pack) in frame_packs.iter().enumerate() {
            match pack {
                RDPack::PointCloud(point_cloud) => {
                    log::debug!("📦 处理点云数据包 #{}", pack_idx);
                    // 将点云数据转换为点形状并生成
                    spawn_point_cloud_from_pack(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &material_manager,
                        point_cloud,
                        playback.current_frame_index,
                    );
                }
                RDPack::SpawnShape(shape_pack) => {
                    log::debug!("📦 处理形状数据包 #{}", pack_idx);
                    // 直接生成形状（shape_pack 是 &Box<RDShapePack>，需要解引用）
                    crate::graph::data_processing::actions::spawn::spawn_shape(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &material_manager,
                        (**shape_pack).clone(),
                    );
                    
                    // 添加回放标记
                    commands.spawn((
                        ReplayedEntity {
                            frame_index: playback.current_frame_index,
                        },
                        Name::new(format!("Shape_{}_{}", playback.current_frame_index, pack_idx)),
                    ));
                }
                _ => {
                    log::debug!("⚠️ 跳过不支持的数据包类型 #{}", pack_idx);
                }
            }
        }
    } else {
        log::warn!(
            "❌ 无法加载帧 #{} 的数据",
            playback.current_frame_index
        );
    }

    // ✅ 修复3：只在非手动切换且正在播放时才更新时间戳和前进到下一帧
    if !playback.manual_frame_change && playback.is_playing {
        let current_time_ms = (time.elapsed_secs() * 1000.0) as u64;
        playback.last_update_time = current_time_ms;
        
        // 前进到下一帧
        playback.next_frame(total_frames);

        // 如果已经播放完所有帧
        if playback.current_frame_index >= total_frames {
            if playback.loop_playback {
                playback.current_frame_index = 0;
            } else {
                playback.pause();
                log::info!("▶️ 回放完成");
            }
        }
    } else {
        // 手动切换后，清除标志位
        playback.clear_manual_flag();
        log::debug!("✅ 手动帧切换完成，清除标志位");
    }
}

/// 获取当前帧的数据（支持 SQLite 和内存双模式）
fn get_current_frame_data(
    recorder: &DataRecorder,
    frame_index: usize,
) -> Option<Vec<RDPack>> {
    // 优先从内存中获取
    if let Some(frame) = recorder.frames.get(frame_index) {
        log::debug!("✅ 从内存加载帧 #{}", frame_index);
        return Some(frame.points.clone());
    }

    log::debug!("⚠️ 内存中未找到帧 #{}, 尝试从 SQLite 加载...", frame_index);

    // 如果内存中没有，尝试从 SQLite 加载
    #[cfg(feature = "storage")]
    if let Some(ref storage_arc) = recorder.storage {
        if let Ok(storage) = storage_arc.lock() {
            // 获取该索引对应的帧元数据
            if let Ok(all_frames) = storage.database().get_all_frames() {
                log::debug!("📊 SQLite 中共有 {} 帧", all_frames.len());
                
                if let Some(metadata) = all_frames.get(frame_index) {
                    log::debug!("📦 找到帧 #{} 的元数据: frame_id={}", frame_index, metadata.frame_id);
                    
                    // 从 SQLite 加载二进制数据
                    if let Ok(binary_data) = storage.load_frame(metadata.frame_id) {
                        log::debug!("📥 成功加载二进制数据 ({} bytes)", binary_data.len());
                        
                        // 反序列化为 RDPack
                        if let Ok(packs) = deserialize_point_cloud(&binary_data) {
                            log::debug!("✅ 成功反序列化 {} 个数据包", packs.len());
                            return Some(packs);
                        } else {
                            log::error!("❌ 反序列化帧 #{} 失败", frame_index);
                        }
                    } else {
                        log::error!("❌ 从 SQLite 加载帧 #{} 的二进制数据失败", frame_index);
                    }
                } else {
                    log::error!("❌ SQLite 中不存在索引为 {} 的帧（总共 {} 帧）", frame_index, all_frames.len());
                }
            } else {
                log::error!("❌ 获取 SQLite 帧列表失败");
            }
        } else {
            log::error!("❌ 无法获取存储锁");
        }
    } else {
        log::warn!("⚠️ 未启用 SQLite 存储功能");
    }

    None
}

/// 从点云数据包生成点实体
fn spawn_point_cloud_from_pack(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    _material_manager: &crate::graph::materials::MaterialManager,
    point_cloud: &crate::module::parser::core::PointCloudPack,
    frame_index: usize,
) {
    use bevy::prelude::*;
    
    log::info!(
        "🔵 渲染点云帧 #{}，共 {} 个点",
        frame_index,
        point_cloud.points.len()
    );
    
    // 计算点云的边界框用于调试
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;
    
    for &(x, y, z) in &point_cloud.points {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
        min_z = min_z.min(z);
        max_z = max_z.max(z);
    }
    
    log::info!(
        "📊 点云边界框 - X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}], Z:[{:.2}, {:.2}]",
        min_x, max_x, min_y, max_y, min_z, max_z
    );
    
    // 为每个点生成一个球体（增大半径以便观察）
    let sphere_mesh = meshes.add(Sphere::new(0.15).mesh());
    
    // 使用高亮黄色材质，并禁用背面剔除以确保可见性
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0), // 黄色，更容易看到
        emissive: LinearRgba::rgb(1.0, 1.0, 0.0), // 更强的自发光效果
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });
    
    let mut spawned_count = 0;
    for (i, &(x, y, z)) in point_cloud.points.iter().enumerate() {
        // 打印前几个点的坐标用于调试
        if i < 5 {
            log::debug!("  点 {}: ({:.2}, {:.2}, {:.2})", i, x, y, z);
        }
        
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(x, y, z),
            ReplayedEntity {
                frame_index,
            },
            Name::new(format!("Point_{}_{}", frame_index, i)),
        ));
        
        spawned_count += 1;
    }
    
    log::info!("✅ 成功生成 {} 个点实体", spawned_count);
}

/// 调试系统：检测和报告所有回放实体的状态
pub fn debug_replayed_entities(
    replayed_entities: Query<(Entity, &ReplayedEntity, &Transform, Option<&Name>), With<ReplayedEntity>>,
    time: Res<Time>,
    playback: Res<PlaybackManager>,
) {
    // 每2秒输出一次实体状态
    let current_time = time.elapsed_secs();
    static mut LAST_LOG_TIME: f32 = 0.0;
    
    unsafe {
        if current_time - LAST_LOG_TIME < 2.0 {
            return;
        }
        LAST_LOG_TIME = current_time;
    }
    
    let entity_count = replayed_entities.iter().count();
    
    if entity_count == 0 {
        log::warn!("⚠️ [DEBUG] 当前场景中没有 ReplayedEntity 实体");
        log::warn!("   - is_playing: {}", playback.is_playing);
        log::warn!("   - current_frame_index: {}", playback.current_frame_index);
        return;
    }
    
    log::info!("🔍 [DEBUG] 检测到 {} 个 ReplayedEntity 实体", entity_count);
    
    // 统计不同帧的实体数量
    let mut frame_stats: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut sample_positions = Vec::new();
    
    for (entity, replayed, transform, name) in replayed_entities.iter() {
        *frame_stats.entry(replayed.frame_index).or_insert(0) += 1;
        
        // 收集前3个实体的位置信息
        if sample_positions.len() < 3 {
            sample_positions.push((
                entity,
                replayed.frame_index,
                transform.translation,
                name.map(|n| n.as_str()).unwrap_or("unnamed"),
            ));
        }
    }
    
    log::info!("📊 [DEBUG] 帧分布统计:");
    for (frame_idx, count) in frame_stats.iter() {
        log::info!("   - 帧 #{}: {} 个实体", frame_idx, count);
    }
    
    log::info!("📍 [DEBUG] 示例实体位置（前3个）:");
    for (entity, frame_idx, pos, name) in sample_positions {
        log::info!("   - Entity {:?} (帧 #{}, {}): ({:.2}, {:.2}, {:.2})", 
                   entity, frame_idx, name, pos.x, pos.y, pos.z);
    }
}

