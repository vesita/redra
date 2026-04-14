use bevy::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use log::debug;
use redra_storage::{FrameType, FrameStorage};

/// 数据帧 - 带有完整元数据的帧结构
#[derive(Clone, Debug)]
pub struct DataFrame {
    pub frame_id: u32,              // 帧 ID
    pub sequence_number: u64,       // 全局序列号
    pub timestamp: u64,             // 时间戳（毫秒）
    pub points: Vec<redra_parser::RDPack>,        // 帧包含的所有点/形状数据
    pub is_complete: bool,          // 帧是否完整
    pub frame_type: FrameType,      // 帧类型（来自 redra_storage）
}

/// 帧构建器 - 用于累积点数据直到帧完成
#[derive(Debug)]
pub struct FrameBuilder {
    pub frame_id: u32,
    pub start_time: u64,
    pub points: Vec<redra_parser::RDPack>,
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

    pub fn add_point(&mut self, pack: redra_parser::RDPack) {
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
            recording_mode: RecordingMode::AutoSave,  // 默认开启自动保存录制
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
    pub fn add_point_to_frame(&mut self, pack: redra_parser::RDPack) {
        // 如果录制关闭，直接忽略
        if self.recording_mode == RecordingMode::Off {
            return;
        }

        // 根据 RDPack 类型处理
        match pack {
            redra_parser::RDPack::PointCloud(point_cloud) => {
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
                    builder.add_point(redra_parser::RDPack::PointCloud(point_cloud.clone()));
                    
                    // 检查帧是否完成
                    if builder.is_complete() {
                        self.finish_current_frame();
                    }
                }
            }
            redra_parser::RDPack::SpawnShape(_) | redra_parser::RDPack::SpawnFormat(_) | redra_parser::RDPack::Message(_) => {
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
                        let buffer = super::serialization::serialize_point_cloud(&frame.points);
                        
                        let metadata = redra_storage::FrameMetadata {
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
                            let buffer = super::serialization::serialize_point_cloud(&frame.points);
                            
                            let metadata = redra_storage::FrameMetadata {
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
                let buffer = super::serialization::serialize_point_cloud(&frame.points);
                
                let metadata = redra_storage::FrameMetadata {
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
    pub loaded_frame: Option<Vec<redra_parser::RDPack>>,
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

/// 回放标记组件 - 标记由回放系统生成的实体
#[derive(Component)]
pub struct ReplayedEntity {
    pub frame_index: usize,
}