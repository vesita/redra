use bevy::prelude::*;
use log::{info, error, warn};

// 导入模块
pub mod actions;
pub mod entities;

// 定义数据处理插件
pub struct DataProcessingPlugin;

impl Plugin for DataProcessingPlugin {
    fn build(&self, app: &mut App) {
        // 注册资源
        app
            .init_resource::<actions::record::DataRecorder>()
            .init_resource::<actions::record::PlaybackManager>()
            .add_systems(Startup, initialize_storage)
            .add_systems(Update, (
                actions::record::record_data_frames,
                actions::record::replay_data_frames.after(actions::record::record_data_frames),
                actions::record::debug_replayed_entities.after(actions::record::replay_data_frames),
            ));
    }
}

/// 初始化 SQLite 存储系统
fn initialize_storage(mut recorder: ResMut<actions::record::DataRecorder>) {
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
            // 清空数据库，确保启动时没有历史数据
            if let Err(e) = storage.database().clear_all_frames() {
                warn!("⚠️ 清空数据库失败: {}（可能是新数据库）", e);
            } else {
                info!("✅ 已清空数据库，启动时为干净状态");
            }
            
            recorder.storage = Some(std::sync::Arc::new(std::sync::Mutex::new(storage)));
            info!("✅ SQLite存储初始化成功");
        }
        Err(e) => {
            error!("❌ 初始化SQLite存储失败: {}。使用纯内存模式。", e);
            recorder.storage = None;
        }
    }
}