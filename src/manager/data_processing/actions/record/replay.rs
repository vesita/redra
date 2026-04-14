use bevy::prelude::*;
use redra_parser::InternalPointCloudPack;
use redra_storage::FrameStorage;
use crate::manager::data_processing::actions::record::models::{PlaybackManager, ReplayedEntity, DataRecorder};
use crate::graph::materials::MaterialManager;

/// 用于跟踪上一次帧数的资源
#[derive(Resource, Default)]
pub struct LastFrameCount {
    pub count: usize,
}

/// 检测数据变化并自动激活回放
pub fn auto_activate_playback(
    recorder: Res<DataRecorder>,
    mut playback: ResMut<PlaybackManager>,
    mut last_frame_count: ResMut<LastFrameCount>,
) {
    let current_frame_count = recorder.total_frames();
    
    // 如果帧数增加了，激活回放
    if current_frame_count > 0 && current_frame_count > last_frame_count.count {
        log::info!(
            "Detected new data - frames: {} -> {}, activating playback", 
            last_frame_count.count, current_frame_count
        );
        
        // 更新最后帧数
        last_frame_count.count = current_frame_count;
        
        // 如果还没有播放，开始播放
        if !playback.is_playing {
            playback.play();
            // 跳转到最新帧
            playback.seek_to(current_frame_count - 1);
        } else {
            // 如果已经在播放，跳转到最新帧
            playback.seek_to(current_frame_count - 1);
        }
    }
}

/// 回放数据帧系统
/// 根据 PlaybackManager 的当前帧索引，从 DataRecorder 获取数据并生成实体
pub fn replay_data_frames(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_manager: Res<MaterialManager>,
    recorder: Res<DataRecorder>,
    mut playback: ResMut<PlaybackManager>,
    existing_entities: Query<Entity, With<ReplayedEntity>>,
    time: Res<Time>,
) {
    // 修复1：如果既不在播放状态，也没有手动切换帧，则不处理
    if !playback.is_playing && !playback.manual_frame_change {
        return;
    }

    let total_frames = recorder.total_frames();
    
    log::info!("Replay system check - is_playing={}, manual_change={}, total_frames={}, current_index={}", 
               playback.is_playing, playback.manual_frame_change, total_frames, playback.current_frame_index);
    
    if total_frames == 0 {
        log::warn!("No frame data available");
        playback.clear_manual_flag();  // 清除标志
        return;
    }

    // 修复2：如果是手动切换帧（无论是否暂停），立即渲染
    let should_render = if playback.manual_frame_change {
        log::info!("Detected manual frame switch, render frame #{}", playback.current_frame_index);
        true
    } else {
        // 否则，检查时间间隔（仅在播放状态下）
        if !playback.is_playing {
            return;
        }
        
        let current_time_ms = (time.elapsed_secs() * 1000.0) as u64;
        let adjusted_interval = ((playback.frame_interval_ms as f32) / playback.playback_speed) as u64;
        
        log::debug!("Time check - current_time={}, last_update={}, interval={}, speed={}", 
                    current_time_ms, playback.last_update_time, adjusted_interval, playback.playback_speed);
        
        // 首次播放时初始化时间戳，并立即渲染第一帧
        if playback.last_update_time == 0 {
            playback.last_update_time = current_time_ms.saturating_sub(adjusted_interval);
            log::info!("Initialize playback timestamp, render first frame immediately");
            true
        } else {
            // 使用 saturating_sub 防止下溢
            let elapsed = current_time_ms.saturating_sub(playback.last_update_time);
            
            log::debug!("Elapsed time: {} ms, required interval: {} ms", elapsed, adjusted_interval);
            
            if elapsed < adjusted_interval {
                log::debug!("Time not reached, skip this frame");
                return;
            }
            
            log::info!("Time reached, prepare to render frame #{}", playback.current_frame_index);
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
        log::debug!("Cleared {} old entities", cleared_count);
    }

    // 获取当前帧的数据
    let frame_data = get_current_frame_data(&recorder, playback.current_frame_index);
    
    if let Some(frame_packs) = frame_data {
        log::info!(
            "Replay frame #{}/{} - contains {} packages",
            playback.current_frame_index,
            total_frames.saturating_sub(1),  // 显示最大索引
            frame_packs.len()
        );

        // 为每个 RDPack 生成实体
        for (pack_idx, pack) in frame_packs.iter().enumerate() {
            match pack {
                redra_parser::RDPack::PointCloud(point_cloud) => {
                    log::debug!("Process point cloud package #{}", pack_idx);
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
                redra_parser::RDPack::SpawnShape(shape_pack) => {
                    log::debug!("Process shape package #{}", pack_idx);
                    // 直接生成形状（shape_pack 是 &Box<RDShapePack>，需要解引用）
                    crate::manager::data_processing::actions::spawn::handle_internal_shape_pack(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &(**shape_pack),
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
                    log::debug!("Skip unsupported package type #{}", pack_idx);
                }
            }
        }
    } else {
        log::warn!(
            "Failed to load data for frame #{}",
            playback.current_frame_index
        );
    }

    // 修复3：只在非手动切换且正在播放时才更新时间戳和前进到下一帧
    if !playback.manual_frame_change && playback.is_playing {
        let current_time_ms = (time.elapsed_secs() * 1000.0) as u64;
        playback.last_update_time = current_time_ms;
        
        // 前进到下一帧
        let next_frame_index = playback.current_frame_index + 1;
        
        // 检查是否到达最后一帧
        if next_frame_index < total_frames {
            playback.current_frame_index = next_frame_index;
        } else {
            // 到达最后一帧
            if playback.loop_playback {
                playback.current_frame_index = 0; // 循环回到第一帧
            } else {
                playback.pause();
                log::info!("Playback completed");
                return; // 立即返回，不再继续处理
            }
        }

        // 如果已经播放完所有帧
        if playback.current_frame_index >= total_frames {
            if playback.loop_playback {
                playback.current_frame_index = 0;
            } else {
                playback.pause();
                log::info!("Playback completed");
            }
        }
    } else {
        // 手动切换后，清除标志位
        playback.clear_manual_flag();
        log::debug!("Manual frame switch completed, clear flag");
    }
}

/// 获取当前帧的数据（支持 SQLite 和内存双模式）
fn get_current_frame_data(
    recorder: &DataRecorder,
    frame_index: usize,
) -> Option<Vec<redra_parser::RDPack>> {
    // 优先从内存中获取
    if let Some(frame) = recorder.frames.get(frame_index) {
        log::debug!("Load frame #{} from memory", frame_index);
        return Some(frame.points.clone());
    }

    log::debug!("Frame #{} not found in memory, try loading from SQLite...", frame_index);

    // 如果内存中没有，尝试从 SQLite 加载
    #[cfg(feature = "storage")]
    if let Some(ref storage_arc) = recorder.storage {
        if let Ok(storage) = storage_arc.lock() {
            // 获取该索引对应的帧元数据
            if let Ok(all_frames) = storage.database().get_all_frames() {
                log::debug!("SQLite contains {} frames", all_frames.len());
                
                if let Some(metadata) = all_frames.get(frame_index) {
                    log::debug!("Found metadata for frame #{}: frame_id={}", frame_index, metadata.frame_id);
                    
                    // 从 SQLite 加载二进制数据
                    if let Ok(binary_data) = storage.load_frame(metadata.frame_id) {
                        log::debug!("Successfully loaded binary data ({} bytes)", binary_data.len());
                        
                        // 反序列化为 RDPack
                        if let Ok(packs) = crate::manager::data_processing::actions::record::serialization::deserialize_point_cloud(&binary_data) {
                            log::debug!("Successfully deserialized {} packages", packs.len());
                            return Some(packs);
                        } else {
                            log::error!("Failed to deserialize frame #{}", frame_index);
                        }
                    } else {
                        log::error!("Failed to load binary data for frame #{} from SQLite", frame_index);
                    }
                } else {
                    log::error!("SQLite does not contain frame with index {} (total {} frames)", frame_index, all_frames.len());
                }
            } else {
                log::error!("Failed to get SQLite frame list");
            }
        } else {
            log::error!("Failed to acquire storage lock");
        }
    } else {
        log::warn!("SQLite storage feature not enabled");
    }

    None
}

/// 从点云数据包生成点实体
fn spawn_point_cloud_from_pack(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    _material_manager: &MaterialManager,
    point_cloud: &InternalPointCloudPack,
    frame_index: usize,
) {
    use bevy::prelude::*;
    
    log::info!(
        "Render point cloud frame #{}, total {} points",
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
        "Point cloud bounds - X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}], Z:[{:.2}, {:.2}]",
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
            log::debug!("  Point {}: ({:.2}, {:.2}, {:.2})", i, x, y, z);
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
    
    log::info!("Successfully generated {} point entities", spawned_count);
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
        log::warn!("DEBUG: No ReplayedEntity entities in current scene");
        log::warn!("   - is_playing: {}", playback.is_playing);
        log::warn!("   - current_frame_index: {}", playback.current_frame_index);
        return;
    }
    
    log::info!("DEBUG: Detected {} ReplayedEntity entities", entity_count);
    
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
    
    log::info!("DEBUG: Frame distribution statistics:");
    for (frame_idx, count) in frame_stats.iter() {
        log::info!("   - Frame #{}: {} entities", frame_idx, count);
    }
    
    log::info!("DEBUG: Sample entity positions (first 3):");
    for (entity, frame_idx, pos, name) in sample_positions {
        log::info!("   - Entity {:?} (frame #{}, {}): ({:.2}, {:.2}, {:.2})", 
                   entity, frame_idx, name, pos.x, pos.y, pos.z);
    }
}

/// 从存储加载指定帧
pub fn load_frame_from_storage(
    _storage: &FrameStorage,
    _frame_index: usize,
) -> Result<Vec<redra_parser::RDPack>, Box<dyn std::error::Error>> {
    // 因为 redra_storage 中可能没有 get_frame_by_index 方法，我们尝试使用其他方法
    // 这里暂时返回错误，需要根据实际的 redra_storage API 来实现
    unimplemented!("Need to implement based on actual redra_storage API")
}