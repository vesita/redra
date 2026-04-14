use bevy::prelude::*;
use redra_net::RDChannel;
use crate::manager::data_processing::actions::record::models::{DataRecorder, RecordingMode};
use std::time::{Duration, Instant};

// 添加一个资源来跟踪最后接收数据的时间
#[derive(Resource)]
pub struct LastReceiveTime {
    time: Instant,
}

impl Default for LastReceiveTime {
    fn default() -> Self {
        Self {
            time: Instant::now(),
        }
    }
}

/// 记录数据帧系统
/// 从 channel 接收数据并按照帧结构组织
pub fn record_data_frames(
    mut recorder: ResMut<DataRecorder>,
    mut channel: ResMut<RDChannel>,
    mut last_receive_time: Option<ResMut<LastReceiveTime>>,
    time: Res<Time>,
) {
    // 如果录制关闭，直接返回
    if recorder.recording_mode == RecordingMode::Off {
        return;
    }

    // 初始化最后接收时间资源（如果不存在）
    let mut should_init_time = last_receive_time.is_none();
    let mut last_receive_time_cell = last_receive_time.as_deref_mut();
    
    if should_init_time {
        return; // 需要在下一次运行时重新检查资源
    }

    // 接收所有可用的数据包并记录到当前帧
    let mut received_count = 0;
    while let Ok(pack) = channel.receiver.try_recv() {
        log::debug!("Received data package");
        
        if let Some(ref mut last_time) = last_receive_time_cell {
            last_time.time = Instant::now();
        }
        
        recorder.add_point_to_frame(pack);
        received_count += 1;
    }
    
    if received_count > 0 {
        log::info!("Received {} packages this frame, total received {} points, {} frames in memory", 
               received_count, 
               recorder.total_points_received,
               recorder.memory_frame_count());
    }
    
    // 检查是否需要强制完成当前帧（如果超过一定时间没有新数据）
    if let Some(ref mut last_time) = last_receive_time_cell {
        let elapsed = last_time.time.elapsed();
        
        // 如果超过1秒没有新数据，强制完成当前帧（如果有的话）
        if elapsed > Duration::from_secs(1) {
            if recorder.current_builder.is_some() && recorder.current_builder.as_ref().unwrap().points.len() > 0 {
                log::info!("Force finishing current frame due to timeout, elapsed: {:?}", elapsed);
                recorder.force_finish_frame();
            }
        }
    }
}