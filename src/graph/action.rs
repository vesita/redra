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
    
    info!("正在初始化帧存储，路径: {:?}", base_path);
    
    match FrameStorage::new(&base_path) {
        Ok(storage) => {
            recorder.storage = Some(std::sync::Arc::new(std::sync::Mutex::new(storage)));
            info!("SQLite存储初始化成功");
            
            // 显示统计信息
            if let Ok(stats) = recorder.storage.as_ref().unwrap().lock().unwrap().database().get_stats() {
                info!(
                    "数据库统计: {} 帧, {} 总点数",
                    stats.total_frames,
                    stats.total_points
                );
            }
        }
        Err(e) => {
            error!("初始化SQLite存储失败: {}。使用纯内存模式。", e);
            recorder.storage = None;
        }
    }
}