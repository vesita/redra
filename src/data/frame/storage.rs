//! 帧数据持久化模块 — 基于 SQLite（通过 sea-orm）

#[cfg(feature = "graph")]
use bevy::prelude::*;

pub mod sql;

pub use sql::FrameStorage;

/// FrameStorage 的 Bevy 插件。
#[cfg(feature = "graph")]
pub struct FrameStoragePlugin;

#[cfg(feature = "graph")]
impl Plugin for FrameStoragePlugin {
    fn build(&self, app: &mut App) {
        let cwd = std::env::current_dir().ok();
        let exe_dir = std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()));
        let tmp_dir = Some(std::env::temp_dir());

        let mut dirs = Vec::new();
        if let Some(d) = &cwd { if !dirs.contains(d) { dirs.push(d.clone()); } }
        if let Some(d) = &exe_dir { if !dirs.contains(d) { dirs.push(d.clone()); } }
        if let Some(d) = &tmp_dir { if !dirs.contains(d) { dirs.push(d.clone()); } }

        let mut storage: Option<FrameStorage> = None;
        for dir in &dirs {
            let path = dir.join("storage.db");
            let test_path = dir.join(".redra_writable_test");
            match std::fs::write(&test_path, b"test") {
                Ok(()) => { let _ = std::fs::remove_file(&test_path); }
                Err(e) => {
                    log::warn!("跳过 {} (不可写: {})", dir.display(), e);
                    continue;
                }
            }
            match FrameStorage::new(&path) {
                Ok(s) => {
                    log::info!("数据库已打开: {}", path.display());
                    storage = Some(s);
                    break;
                }
                Err(e) => {
                    log::warn!("尝试打开 {} 失败: {}", path.display(), e);
                }
            }
        }

        if let Some(s) = storage {
            app.insert_resource(s);
        } else {
            log::error!("无法在任何位置创建数据库，文件管理功能不可用");
        }
    }
}
