use bevy::prelude::*;
use redra_storage::FrameStorage;
use expto::rdmp::Unit;
use parser::{ProtocolParser, ParsedCommand};
use std::collections::HashMap;

/// 实体ID组件（用于标记回放创建的实体）
#[derive(Component)]
pub struct ReplayEntity(pub u64);

/// 回放状态
#[derive(Resource, Default, Clone, PartialEq, Debug)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing {
        current_frame: u32,
        total_frames: u32,
    },
    Paused {
        current_frame: u32,
        total_frames: u32,
    },
}

/// 回放配置（包含可变参数）
#[derive(Resource)]
pub struct PlaybackConfig {
    pub playback_speed: f32,
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        Self {
            playback_speed: 1.0,
        }
    }
}

impl PlaybackState {
    pub fn is_playing(&self) -> bool {
        matches!(self, PlaybackState::Playing { .. })
    }

    pub fn current_frame(&self) -> Option<u32> {
        match self {
            PlaybackState::Playing { current_frame, .. } |
            PlaybackState::Paused { current_frame, .. } => Some(*current_frame),
            _ => None,
        }
    }

    pub fn total_frames(&self) -> Option<u32> {
        match self {
            PlaybackState::Playing { total_frames, .. } |
            PlaybackState::Paused { total_frames, .. } => Some(*total_frames),
            _ => None,
        }
    }
}

/// 回放管理器资源
#[derive(Resource)]
pub struct PlaybackManager {
    storage: FrameStorage,
    entity_map: HashMap<u64, Entity>,
    last_update_time: std::time::Instant,
    frame_interval_ms: u64,
}

impl PlaybackManager {
    pub fn new(storage: FrameStorage) -> Self {
        Self {
            storage,
            entity_map: HashMap::new(),
            last_update_time: std::time::Instant::now(),
            frame_interval_ms: 33, // 默认 30 FPS
        }
    }

    /// 开始回放
    pub async fn start_playback(&mut self, commands: &mut Commands<'_, '_>, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Result<(), String> {
        let stats = self.storage.get_stats().await?;
        let total_frames = stats.0 as u32;
        
        if total_frames == 0 {
            return Err("没有可回放的帧数据".to_string());
        }

        // 清除现有实体
        self.clear_all_entities(commands);
        
        // 重置实体映射
        self.entity_map.clear();
        
        log::info!("▶️ 开始回放，共 {} 帧", total_frames);
        
        Ok(())
    }

    /// 停止回放
    pub fn stop_playback(&mut self, commands: &mut Commands) {
        self.clear_all_entities(commands);
        self.entity_map.clear();
        log::info!("⏹️ 停止回放");
    }

    /// 暂停回放
    pub fn pause_playback(&mut self) {
        log::info!("⏸️ 暂停回放");
    }

    /// 恢复回放
    pub fn resume_playback(&mut self) {
        log::info!("▶️ 恢复回放");
    }

    /// 跳转到指定帧
    pub async fn jump_to_frame(
        &mut self,
        frame_index: u32,
        commands: &mut Commands<'_, '_>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Result<(), String> {
        let frame_data = self.storage.load_frame_data(frame_index).await?;
        
        // 清除当前实体
        self.clear_all_entities(commands);
        self.entity_map.clear();
        
        // 重建该帧的实体
        for unit in &frame_data.units {
            self.process_unit(unit, commands, meshes, materials)?;
        }
        
        log::info!("⏭️ 跳转到帧 {}", frame_index);
        
        Ok(())
    }

    /// 处理下一帧
    pub async fn process_next_frame(
        &mut self,
        commands: &mut Commands<'_, '_>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Result<bool, String> {
        let current_frame = match self.get_current_frame() {
            Some(frame) => frame,
            None => return Ok(false),
        };

        let stats = self.storage.get_stats().await?;
        let total_frames = stats.0 as u32;

        if current_frame >= total_frames {
            // 回放结束，循环到第一帧
            return self.jump_to_frame(0, commands, meshes, materials).await.map(|_| true);
        }

        // 加载并处理当前帧
        let frame_data = self.storage.load_frame_data(current_frame).await?;
        
        // 清除旧实体
        self.clear_all_entities(commands);
        self.entity_map.clear();
        
        // 创建新实体
        for unit in &frame_data.units {
            self.process_unit(unit, commands, meshes, materials)?;
        }
        
        Ok(true)
    }

    /// 处理单个 Unit
    fn process_unit(
        &mut self,
        unit: &Unit,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Result<(), String> {
        let parsed_cmd = ProtocolParser::parse_unit(unit);
        
        match parsed_cmd {
            ParsedCommand::Spawn { id, transform } => {
                let transform_component = if let Some(t) = transform {
                    Transform::from_translation(Vec3::from_array(t.position))
                        .with_rotation(Quat::from_euler(EulerRot::XYZ, t.rotation[0], t.rotation[1], t.rotation[2]))
                        .with_scale(Vec3::from_array(t.scale))
                } else {
                    Transform::IDENTITY
                };
                
                let entity = commands.spawn((
                    ReplayEntity(id),
                    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))), // 默认立方体
                    MeshMaterial3d(materials.add(StandardMaterial::from(Color::WHITE))),
                    transform_component,
                )).id();
                
                self.entity_map.insert(id, entity);
            },
            ParsedCommand::Update { id, transform } => {
                if let Some(&entity) = self.entity_map.get(&id) {
                    if let Some(t) = transform {
                        let new_transform = Transform::from_translation(Vec3::from_array(t.position))
                            .with_rotation(Quat::from_euler(EulerRot::XYZ, t.rotation[0], t.rotation[1], t.rotation[2]))
                            .with_scale(Vec3::from_array(t.scale));
                        
                        commands.entity(entity).insert(new_transform);
                    }
                }
            },
            ParsedCommand::Destroy { id } => {
                if let Some(&entity) = self.entity_map.get(&id) {
                    commands.entity(entity).despawn();
                    self.entity_map.remove(&id);
                }
            },
            ParsedCommand::Unknown => {
                log::warn!("收到未知命令");
            }
        }
        
        Ok(())
    }

    /// 清除所有回放实体
    fn clear_all_entities(&mut self, commands: &mut Commands<'_, '_>) {
        for (_, entity) in self.entity_map.drain() {
            commands.entity(entity).despawn();
        }
    }

    /// 获取当前帧索引
    fn get_current_frame(&self) -> Option<u32> {
        // 这里需要从 PlaybackState 资源中获取
        // 简化实现，实际应该在系统中管理
        None
    }

    /// 设置帧间隔（毫秒）
    pub fn set_frame_interval(&mut self, interval_ms: u64) {
        self.frame_interval_ms = interval_ms;
    }

    /// 获取存储引用
    pub fn storage(&self) -> &FrameStorage {
        &self.storage
    }
}

/// 回放插件
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlaybackState>()
            .init_resource::<PlaybackConfig>()
            .add_systems(Update, (
                handle_playback_controls,
                playback_tick,
            ));
    }
}

/// 处理回放控制输入
fn handle_playback_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut playback_state: ResMut<PlaybackState>,
    mut playback_config: ResMut<PlaybackConfig>,
) {
    // 空格键切换播放/暂停
    if keyboard_input.just_pressed(KeyCode::Space) {
        match playback_state.as_ref() {
            PlaybackState::Playing { current_frame, total_frames } => {
                *playback_state = PlaybackState::Paused {
                    current_frame: *current_frame,
                    total_frames: *total_frames,
                };
            },
            PlaybackState::Paused { current_frame, total_frames } => {
                *playback_state = PlaybackState::Playing {
                    current_frame: *current_frame,
                    total_frames: *total_frames,
                };
            },
            _ => {
                // 从停止状态开始播放
                *playback_state = PlaybackState::Playing {
                    current_frame: 0,
                    total_frames: 0, // 需要在初始化时设置
                };
            }
        }
    }

    // R 键重新开始
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        *playback_state = PlaybackState::Playing {
            current_frame: 0,
            total_frames: playback_state.total_frames().unwrap_or(0),
        };
    }
    
    // +/- 调整播放速度
    if keyboard_input.just_pressed(KeyCode::Equal) {
        playback_config.playback_speed = (playback_config.playback_speed + 0.1).min(4.0);
    }
    if keyboard_input.just_pressed(KeyCode::Minus) {
        playback_config.playback_speed = (playback_config.playback_speed - 0.1).max(0.1);
    }
}

/// 回放计时系统
fn playback_tick(
    time: Res<Time>,
    mut playback_state: ResMut<PlaybackState>,
) {
    if let PlaybackState::Playing { current_frame, total_frames, .. } = playback_state.as_mut() {
        // 这里应该根据时间推进帧
        // 简化实现，实际应该在 PlaybackManager 中处理
    }
}
