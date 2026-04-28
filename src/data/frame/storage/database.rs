//! 数据库管理和公共 API

use sea_orm::entity::prelude::*;
use sea_orm::{Statement, DatabaseConnection, Set, QuerySelect, QueryOrder};
use std::path::{Path, PathBuf};
use tokio::fs;
use chrono::Utc;

use super::SerializableKeyFrame;
use crate::data::frame::{KeyFrame, Inpto};
use std::collections::HashMap;

// ============================================================================
// 数据库实体定义
// ============================================================================

mod frames_entity {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "frames")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub frame_id: i32,
        pub timestamp: i64,
        pub entity_count: i32,
        pub data_path: String,
        pub created_at: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

// ============================================================================
// 数据库管理层
// ============================================================================

struct FrameDatabase {
    conn: DatabaseConnection,
    data_dir: PathBuf,
}

impl FrameDatabase {
    async fn new(db_path: &Path, data_dir: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| format!("创建数据库目录失败: {}", e))?;
        }
        fs::create_dir_all(data_dir).await.map_err(|e| format!("创建数据目录失败: {}", e))?;

        let db_path_str = format!("sqlite:{}", db_path.display());
        let conn = sea_orm::Database::connect(&db_path_str).await.map_err(|e| format!("打开数据库失败: {}", e))?;

        let mut db = FrameDatabase { conn, data_dir: data_dir.to_path_buf() };
        db.init_tables().await?;
        Ok(db)
    }

    async fn init_tables(&mut self) -> Result<(), String> {
        self.conn.execute(Statement::from_string(
            self.conn.get_database_backend(),
            r#"CREATE TABLE IF NOT EXISTS frames (
                frame_id INTEGER PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                entity_count INTEGER NOT NULL,
                data_path TEXT NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )"#.to_owned(),
        )).await.map_err(|e| format!("创建表失败: {}", e))?;
        self.create_index_if_not_exists("idx_frames_timestamp", "frames", "timestamp").await?;
        Ok(())
    }

    async fn create_index_if_not_exists(&self, index_name: &str, table_name: &str, column_name: &str) -> Result<(), String> {
        let exists_stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"SELECT name FROM sqlite_master WHERE type='index' AND name=?;"#,
            vec![index_name.into()],
        );
        if self.conn.query_one(exists_stmt).await.ok().flatten().is_none() {
            self.conn.execute(Statement::from_string(
                self.conn.get_database_backend(),
                format!("CREATE INDEX {} ON {}({})", index_name, table_name, column_name),
            )).await.map_err(|e| format!("创建索引失败: {}", e))?;
        }
        Ok(())
    }

    async fn save_frame_record(&self, frame_id: u32, timestamp: u64, entity_count: u32, data_path: &str) -> Result<(), String> {
        use frames_entity::{Entity, ActiveModel};
        Entity::insert(ActiveModel {
            frame_id: Set(frame_id as i32),
            timestamp: Set(timestamp as i64),
            entity_count: Set(entity_count as i32),
            data_path: Set(data_path.to_string()),
            created_at: Set(Utc::now().timestamp() as i64),
        }).exec_without_returning(&self.conn).await.map_err(|e| format!("保存帧记录失败: {}", e))?;
        Ok(())
    }

    async fn get_frame_record(&self, frame_id: u32) -> Result<Option<(u64, u32, String)>, String> {
        use frames_entity::Entity;
        let model = Entity::find()
            .filter(frames_entity::Column::FrameId.eq(frame_id as i32))
            .one(&self.conn).await.map_err(|e| format!("查询帧记录失败: {}", e))?;
        Ok(model.map(|m| (m.timestamp as u64, m.entity_count as u32, m.data_path)))
    }

    async fn get_all_frame_ids(&self) -> Result<Vec<u32>, String> {
        use frames_entity::{Entity, Column};
        let models = Entity::find().select_only().column(Column::FrameId)
            .order_by_asc(Column::Timestamp).all(&self.conn).await
            .map_err(|e| format!("查询所有帧ID失败: {}", e))?;
        Ok(models.into_iter().map(|m: frames_entity::Model| m.frame_id as u32).collect())
    }

    async fn delete_frame_record(&self, frame_id: u32) -> Result<String, String> {
        use frames_entity::{Entity, Column};
        let data_path = self.get_frame_record(frame_id).await?.map(|(_, _, path)| path);
        Entity::delete_many().filter(Column::FrameId.eq(frame_id as i32))
            .exec(&self.conn).await.map_err(|e| format!("删除帧记录失败: {}", e))?;
        Ok(data_path.unwrap_or_default())
    }

    async fn clear_all_records(&self) -> Result<Vec<String>, String> {
        let all_ids = self.get_all_frame_ids().await?;
        let mut data_paths = Vec::new();
        for frame_id in all_ids {
            if let Ok(Some((_, _, path))) = self.get_frame_record(frame_id).await {
                data_paths.push(path);
            }
        }
        frames_entity::Entity::delete_many().exec(&self.conn).await.map_err(|e| format!("清空记录失败: {}", e))?;
        Ok(data_paths)
    }

    async fn get_stats(&self) -> Result<(u64, u64, u64), String> {
        let result = self.conn.query_one(Statement::from_string(
            self.conn.get_database_backend(),
            r#"SELECT COUNT(*) as total, MIN(timestamp) as first, MAX(timestamp) as last FROM frames"#.to_owned(),
        )).await.map_err(|e| format!("查询统计信息失败: {}", e))?;
        if let Some(row) = result {
            let total: i64 = row.try_get("", "total").map_err(|e| e.to_string())?;
            let first: i64 = row.try_get("", "first").map_err(|e| e.to_string())?;
            let last: i64 = row.try_get("", "last").map_err(|e| e.to_string())?;
            Ok((total as u64, first as u64, last as u64))
        } else { Ok((0, 0, 0)) }
    }
}

// ============================================================================
// 转换实现
// ============================================================================

impl From<SerializableKeyFrame> for KeyFrame {
    fn from(serializable: SerializableKeyFrame) -> Self {
        let mut keyframe = KeyFrame::new(serializable.timestamp);
        let mut ids = HashMap::new();
        let mut packs = Vec::new();
        for entity in serializable.entities {
            let inpto = Inpto::new(
                expto::rdmp::ExMesh::default(),
                entity.material,
                entity.transform.into(),
            );
            ids.insert(entity.entity_id, packs.len());
            packs.push(inpto);
        }
        keyframe.ids = ids;
        keyframe.packs = packs;
        keyframe
    }
}

// ============================================================================
// 数据库存储管理器
// ============================================================================

pub struct FrameStorage {
    db: FrameDatabase,
}

impl FrameStorage {
    pub async fn new(base_path: &Path) -> Result<Self, String> {
        let db_path = base_path.join("frames.db");
        let data_dir = base_path.join("data");
        let db = FrameDatabase::new(&db_path, &data_dir).await?;
        Ok(FrameStorage { db })
    }

    pub async fn new_default() -> Result<Self, String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
            .parent().ok_or("无法确定可执行文件所在目录".to_string())?
            .to_path_buf();
        Self::new(&exe_path.join("storage")).await
    }

    pub async fn save_keyframe(&self, keyframe: &KeyFrame, frame_id: u32) -> Result<(), String> {
        let serializable: SerializableKeyFrame = keyframe.into();
        let file_path = self.db.data_dir.join(format!("frame_{}.bin", frame_id));
        let data = bincode::serialize(&serializable).map_err(|e| format!("序列化帧数据失败: {}", e))?;
        fs::write(&file_path, data).await.map_err(|e| format!("写入帧数据失败: {}", e))?;
        self.db.save_frame_record(frame_id, keyframe.timestamp, keyframe.entity_count() as u32, &file_path.to_string_lossy()).await?;
        log::info!("帧 {} 已保存到磁盘 ({} 个实体)", frame_id, keyframe.entity_count());
        Ok(())
    }

    pub async fn load_keyframe(&self, frame_id: u32) -> Result<KeyFrame, String> {
        let (_, _, data_path) = self.db.get_frame_record(frame_id).await?.ok_or(format!("帧 {} 未找到", frame_id))?;
        let data = fs::read(&data_path).await.map_err(|e| format!("读取帧数据失败: {}", e))?;
        let serializable: SerializableKeyFrame = bincode::deserialize(&data).map_err(|e| format!("反序列化帧数据失败: {}", e))?;
        let keyframe: KeyFrame = serializable.into();
        log::info!("帧 {} 已从磁盘加载", frame_id);
        Ok(keyframe)
    }

    pub async fn save_all_keyframes(&self, keyframes: &[KeyFrame]) -> Result<(), String> {
        for (index, keyframe) in keyframes.iter().enumerate() {
            self.save_keyframe(keyframe, index as u32).await?;
        }
        Ok(())
    }

    pub async fn delete_frame(&self, frame_id: u32) -> Result<(), String> {
        let data_path = self.db.delete_frame_record(frame_id).await?;
        if !data_path.is_empty() { let _ = fs::remove_file(&data_path).await; }
        Ok(())
    }

    pub async fn clear_all(&self) -> Result<(), String> {
        let data_paths = self.db.clear_all_records().await?;
        for data_path in data_paths { let _ = fs::remove_file(&data_path).await; }
        if self.db.data_dir.exists() {
            let mut entries = fs::read_dir(&self.db.data_dir).await.map_err(|e| format!("读取数据目录失败: {}", e))?;
            while let Some(entry) = entries.next_entry().await.map_err(|e| format!("读取目录项失败: {}", e))? {
                let _ = fs::remove_file(entry.path()).await;
            }
        }
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<(u64, u64, u64), String> { self.db.get_stats().await }
    pub async fn get_all_frame_ids(&self) -> Result<Vec<u32>, String> { self.db.get_all_frame_ids().await }
}
