use bevy::prelude::*;

pub mod spawn;
pub mod clear;
pub mod record;

// 定义 ActionPlugin 来注册相关系统
pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<record::DataRecorder>()
            .init_resource::<record::PlaybackManager>()
            .add_systems(Update, record::record_data_frames)
            .add_systems(Startup, initialize_storage);
    }
}

/// 初始化 SQLite 存储系统
fn initialize_storage(mut recorder: ResMut<record::DataRecorder>) {
    use std::path::PathBuf;
    use redra_storage::storage::FrameStorage;
    
    // 获取用户数据目录
    let base_path = if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".redra").join("frames")
    } else {
        PathBuf::from("./redra_frames")
    };
    
    info!("Initializing frame storage at: {:?}", base_path);
    
    match FrameStorage::new(&base_path) {
        Ok(storage) => {
            recorder.storage = Some(std::sync::Arc::new(std::sync::Mutex::new(storage)));
            info!("SQLite storage initialized successfully");
            
            // 显示统计信息
            if let Ok(stats) = recorder.storage.as_ref().unwrap().lock().unwrap().database().get_stats() {
                info!(
                    "Database stats: {} frames, {} total points",
                    stats.total_frames,
                    stats.total_points
                );
            }
        }
        Err(e) => {
            error!("Failed to initialize SQLite storage: {}. Using memory-only mode.", e);
            recorder.storage = None;
        }
    }
}