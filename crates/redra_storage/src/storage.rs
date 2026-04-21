use sea_orm::entity::prelude::*;
use sea_orm::{Statement, DatabaseConnection, Set, QuerySelect, QueryOrder};
use std::path::{Path, PathBuf};
use std::fs;
use chrono::Utc;
use expto::rdmp::Unit;
use inpto::FrameData;

// ============================================================================
// 内部数据库实体定义（不对外暴露）
// ============================================================================

mod frames_entity {
    use super::*;

    /// 帧数据表实体 - 仅用于 SeaORM 映射
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "frames")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub frame_id: i32,
        #[sea_orm(unique)]
        pub sequence_number: i64,
        pub timestamp: i64,
        pub unit_count: i32,
        pub data_path: String,
        pub created_at: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

/// 关键帧表实体 - 仅用于 SeaORM 映射
mod keyframes_entity {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "keyframes")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub keyframe_id: i32,
        #[sea_orm(unique)]
        pub sequence_number: i64,
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
// 关键帧管理器（辅助工具，可选暴露）
// ============================================================================

/// 关键帧管理器 - 用于决定何时创建关键帧
pub struct KeyframeManager {
    last_keyframe_time: f64,
    commands_since_last_keyframe: usize,
}

impl KeyframeManager {
    /// 创建新的关键帧管理器
    pub fn new() -> Self {
        Self {
            last_keyframe_time: 0.0,
            commands_since_last_keyframe: 0,
        }
    }

    /// 检查是否应该创建关键帧
    pub fn should_create_keyframe(&self, current_time: f64) -> bool {
        const KEYFRAME_INTERVAL_SECS: f64 = 1.0;
        const MAX_COMMANDS_PER_KEYFRAME: usize = 5000;
        
        current_time - self.last_keyframe_time >= KEYFRAME_INTERVAL_SECS
            || self.commands_since_last_keyframe >= MAX_COMMANDS_PER_KEYFRAME
    }

    /// 更新命令计数
    pub fn increment_command_count(&mut self) {
        self.commands_since_last_keyframe += 1;
    }

    /// 重置关键帧时间
    pub fn reset_with_time(&mut self, current_time: f64) {
        self.last_keyframe_time = current_time;
        self.commands_since_last_keyframe = 0;
    }
}

// ============================================================================
// 数据库管理层（内部实现）
// ============================================================================

/// SQLite 数据库管理器 - 内部实现细节
struct FrameDatabase {
    conn: DatabaseConnection,
    data_dir: PathBuf,
}

impl FrameDatabase {
    /// 创建或打开数据库
    async fn new(db_path: &Path, data_dir: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建数据库目录失败: {}", e))?;
        }
        fs::create_dir_all(data_dir)
            .map_err(|e| format!("创建数据目录失败: {}", e))?;
        
        let db_path_str = format!("sqlite:{}", db_path.display());
        let conn = sea_orm::Database::connect(&db_path_str)
            .await
            .map_err(|e| format!("打开数据库失败: {}", e))?;
        
        let mut db = FrameDatabase {
            conn,
            data_dir: data_dir.to_path_buf(),
        };
        
        db.init_tables().await?;
        
        Ok(db)
    }
    
    /// 初始化数据库表
    async fn init_tables(&mut self) -> Result<(), String> {
        self.conn
            .execute(Statement::from_string(
                self.conn.get_database_backend(),
                r#"
                CREATE TABLE IF NOT EXISTS frames (
                    frame_id INTEGER PRIMARY KEY,
                    sequence_number INTEGER UNIQUE NOT NULL,
                    timestamp INTEGER NOT NULL,
                    unit_count INTEGER NOT NULL,
                    data_path TEXT NOT NULL,
                    created_at INTEGER DEFAULT (strftime('%s', 'now'))
                )
                "#.to_owned(),
            ))
            .await
            .map_err(|e| format!("创建frames表失败: {}", e))?;

        self.conn
            .execute(Statement::from_string(
                self.conn.get_database_backend(),
                r#"
                CREATE TABLE IF NOT EXISTS keyframes (
                    keyframe_id INTEGER PRIMARY KEY,
                    sequence_number INTEGER UNIQUE NOT NULL,
                    timestamp INTEGER NOT NULL,
                    entity_count INTEGER NOT NULL,
                    data_path TEXT NOT NULL,
                    created_at INTEGER DEFAULT (strftime('%s', 'now'))
                )
                "#.to_owned(),
            ))
            .await
            .map_err(|e| format!("创建keyframes表失败: {}", e))?;
        
        self.create_index_if_not_exists("idx_frames_timestamp", "frames", "timestamp").await?;
        self.create_index_if_not_exists("idx_frames_sequence", "frames", "sequence_number").await?;
        self.create_index_if_not_exists("idx_keyframes_timestamp", "keyframes", "timestamp").await?;
        self.create_index_if_not_exists("idx_keyframes_sequence", "keyframes", "sequence_number").await?;

        Ok(())
    }

    async fn create_index_if_not_exists(&self, index_name: &str, table_name: &str, column_name: &str) -> Result<(), String> {
        let exists_stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"SELECT name FROM sqlite_master WHERE type='index' AND name=?;"#,
            vec![index_name.into()]
        );
        let result = self.conn.query_one(exists_stmt).await;

        if result.is_err() || result.unwrap().is_none() {
            let create_stmt = Statement::from_string(
                self.conn.get_database_backend(),
                format!("CREATE INDEX {} ON {}({})", index_name, table_name, column_name),
            );
            self.conn
                .execute(create_stmt)
                .await
                .map_err(|e| format!("创建索引 {} 失败: {}", index_name, e))?;
        }

        Ok(())
    }
    
    /// 保存帧元数据到数据库
    async fn save_frame_record(
        &self,
        frame_id: u32,
        sequence_number: u64,
        timestamp: u64,
        unit_count: u32,
        data_path: &str,
    ) -> Result<(), String> {
        use frames_entity::{Entity, Column, ActiveModel};
        
        let active_model = ActiveModel {
            frame_id: Set(frame_id as i32),
            sequence_number: Set(sequence_number as i64),
            timestamp: Set(timestamp as i64),
            unit_count: Set(unit_count as i32),
            data_path: Set(data_path.to_string()),
            created_at: Set(Utc::now().timestamp() as i64),
        };

        match Entity::insert(active_model)
            .exec_without_returning(&self.conn)
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => {
                use sea_orm::sea_query::Expr;
                
                Entity::update_many()
                    .col_expr(Column::SequenceNumber, Expr::value(sequence_number as i64))
                    .col_expr(Column::Timestamp, Expr::value(timestamp as i64))
                    .col_expr(Column::UnitCount, Expr::value(unit_count as i32))
                    .col_expr(Column::DataPath, Expr::value(data_path.to_string()))
                    .col_expr(Column::CreatedAt, Expr::value(Utc::now().timestamp() as i64))
                    .filter(Column::FrameId.eq(frame_id as i32))
                    .exec(&self.conn)
                    .await
                    .map_err(|e| format!("更新帧记录失败: {}", e))?;
                Ok(())
            }
        }
    }
    
    /// 查询帧记录
    async fn get_frame_record(&self, frame_id: u32) -> Result<Option<(u64, u64, u32, String)>, String> {
        use frames_entity::Entity;
        
        let model = Entity::find()
            .filter(frames_entity::Column::FrameId.eq(frame_id as i32))
            .one(&self.conn)
            .await
            .map_err(|e| format!("查询帧记录失败: {}", e))?;
        
        Ok(model.map(|m| (
            m.sequence_number as u64,
            m.timestamp as u64,
            m.unit_count as u32,
            m.data_path,
        )))
    }
    
    /// 按时间范围查询帧 ID 列表
    async fn query_frame_ids_by_time_range(&self, start: u64, end: u64) -> Result<Vec<u32>, String> {
        use frames_entity::{Entity, Column};
        
        let models = Entity::find()
            .select_only()
            .column(Column::FrameId)
            .filter(Column::Timestamp.gte(start as i64))
            .filter(Column::Timestamp.lte(end as i64))
            .order_by_asc(Column::Timestamp)
            .all(&self.conn)
            .await
            .map_err(|e| format!("查询时间范围帧ID失败: {}", e))?;
        
        Ok(models.into_iter().map(|m: frames_entity::Model| m.frame_id as u32).collect())
    }
    
    /// 获取所有帧 ID
    async fn get_all_frame_ids(&self) -> Result<Vec<u32>, String> {
        use frames_entity::{Entity, Column};
        
        let models = Entity::find()
            .select_only()
            .column(Column::FrameId)
            .order_by_asc(Column::Timestamp)
            .all(&self.conn)
            .await
            .map_err(|e| format!("查询所有帧ID失败: {}", e))?;
        
        Ok(models.into_iter().map(|m: frames_entity::Model| m.frame_id as u32).collect())
    }
    
    /// 删除帧记录
    async fn delete_frame_record(&self, frame_id: u32) -> Result<String, String> {
        use frames_entity::{Entity, Column};
        
        let data_path = self.get_frame_record(frame_id)
            .await?
            .map(|(_, _, _, path)| path);
        
        Entity::delete_many()
            .filter(Column::FrameId.eq(frame_id as i32))
            .exec(&self.conn)
            .await
            .map_err(|e| format!("删除帧记录失败: {}", e))?;
        
        Ok(data_path.unwrap_or_default())
    }
    
    /// 清空所有帧记录
    async fn clear_all_frame_records(&self) -> Result<Vec<String>, String> {
        use frames_entity::Entity;
        
        let all_ids = self.get_all_frame_ids().await?;
        let mut data_paths = Vec::new();
        
        for frame_id in all_ids {
            if let Ok(Some((_, _, _, path))) = self.get_frame_record(frame_id).await {
                data_paths.push(path);
            }
        }
        
        Entity::delete_many()
            .exec(&self.conn)
            .await
            .map_err(|e| format!("清空帧记录失败: {}", e))?;
        
        Ok(data_paths)
    }
    
    /// 获取统计信息
    async fn get_stats(&self) -> Result<(u64, u64, u64), String> {
        let select_result = self.conn
            .query_one(Statement::from_string(
                self.conn.get_database_backend(),
                r#"
                SELECT 
                    COUNT(*) as total_frames,
                    MIN(timestamp) as first_timestamp,
                    MAX(timestamp) as last_timestamp
                FROM frames
                "#.to_owned(),
            ))
            .await
            .map_err(|e| format!("查询统计信息失败: {}", e))?;
            
        if let Some(row) = select_result {
            let total_frames: i64 = row.try_get("", "total_frames")
                .map_err(|e| format!("获取总数失败: {}", e))?;
            let first_timestamp: i64 = row.try_get("", "first_timestamp")
                .map_err(|e| format!("获取最小时间戳失败: {}", e))?;
            let last_timestamp: i64 = row.try_get("", "last_timestamp")
                .map_err(|e| format!("获取最大时间戳失败: {}", e))?;
            
            Ok((total_frames as u64, first_timestamp as u64, last_timestamp as u64))
        } else {
            Err("查询统计信息返回空结果".to_string())
        }
    }
}

// ============================================================================
// 公共 API - 仅依赖 inpto 和 expto
// ============================================================================

/// 帧存储管理器 - 公共接口
/// 
/// 外部调用者只需使用 inpto::FrameData 和 expto::rdmp::Unit
pub struct FrameStorage {
    db: FrameDatabase,
}

impl FrameStorage {
    /// 创建新的存储实例
    pub async fn new(base_path: &Path) -> Result<Self, String> {
        let db_path = base_path.join("frames.db");
        let data_dir = base_path.join("data");
        
        let db = FrameDatabase::new(&db_path, &data_dir).await?;
        
        Ok(FrameStorage { db })
    }

    /// 使用默认路径创建存储（相对于可执行文件）
    pub async fn new_default() -> Result<Self, String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
            .parent()
            .ok_or("无法确定可执行文件所在目录".to_string())
            .map(|p| p.to_path_buf())?;

        let base_path = exe_path.join("storage");
        Self::new(&base_path).await
    }
    
    // ------------------------------------------------------------------------
    // Unit 数据帧操作
    // ------------------------------------------------------------------------
    
    /// 保存单个 Unit 数据帧
    /// 
    /// # 参数
    /// * `unit` - Unit 数据（来自 expto）
    /// * `frame_id` - 帧唯一标识符
    pub async fn save_unit_frame(&self, unit: &Unit, frame_id: u32) -> Result<(), String> {
        let file_path = self.db.data_dir.join(format!("unit_{}.bin", frame_id));
        
        let serialized_data = bincode::serialize(unit)
            .map_err(|e| format!("序列化 Unit 数据失败: {}", e))?;
        
        fs::write(&file_path, serialized_data)
            .map_err(|e| format!("写入帧数据失败: {}", e))?;
        
        let sequence_number = unit.stamp.as_ref()
            .map(|s| s.sequence_number as u64)
            .unwrap_or(frame_id as u64);
        let timestamp = unit.stamp.as_ref()
            .map(|s| s.timestamp)
            .unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            });
        
        self.db.save_frame_record(
            frame_id,
            sequence_number,
            timestamp,
            1, // 单个 Unit
            &file_path.to_string_lossy(),
        ).await?;
        
        Ok(())
    }
    
    /// 加载 Unit 数据帧
    pub async fn load_unit_frame(&self, frame_id: u32) -> Result<Unit, String> {
        let record = self.db.get_frame_record(frame_id)
            .await?
            .ok_or(format!("帧 {} 未找到", frame_id))?;
        
        let (_, _, _, data_path) = record;
        
        let data = fs::read(&data_path)
            .map_err(|e| format!("读取帧数据失败: {}", e))?;
            
        bincode::deserialize(&data)
            .map_err(|e| format!("反序列化 Unit 数据失败: {}", e))
    }
    
    // ------------------------------------------------------------------------
    // FrameData 操作（基于 inpto）
    // ------------------------------------------------------------------------
    
    /// 保存 FrameData 帧
    /// 
    /// # 参数
    /// * `frame_data` - 帧数据（来自 inpto）
    pub async fn save_frame_data(&self, frame_data: &FrameData) -> Result<(), String> {
        let frame_id = frame_data.id as u32;
        let file_path = self.db.data_dir.join(format!("frame_{}.bin", frame_id));
        
        let serialized_data = bincode::serialize(frame_data)
            .map_err(|e| format!("序列化 FrameData 失败: {}", e))?;
        
        fs::write(&file_path, serialized_data)
            .map_err(|e| format!("写入帧数据失败: {}", e))?;
        
        let sequence_number = frame_data.units.first()
            .and_then(|u| u.stamp.as_ref())
            .map(|s| s.sequence_number as u64)
            .unwrap_or(frame_data.id);
        
        self.db.save_frame_record(
            frame_id,
            sequence_number,
            frame_data.timestamp,
            frame_data.units.len() as u32,
            &file_path.to_string_lossy(),
        ).await?;
        
        Ok(())
    }
    
    /// 批量保存 FrameData 帧
    pub async fn save_frame_data_batch(&self, frames: &[FrameData]) -> Result<(), String> {
        for frame in frames {
            self.save_frame_data(frame).await?;
        }
        Ok(())
    }
    
    /// 加载 FrameData 帧
    pub async fn load_frame_data(&self, frame_id: u32) -> Result<FrameData, String> {
        let record = self.db.get_frame_record(frame_id)
            .await?
            .ok_or(format!("帧 {} 未找到", frame_id))?;
        
        let (_, _, _, data_path) = record;
        
        let data = fs::read(&data_path)
            .map_err(|e| format!("读取帧数据失败: {}", e))?;
            
        bincode::deserialize(&data)
            .map_err(|e| format!("反序列化 FrameData 失败: {}", e))
    }
    
    // ------------------------------------------------------------------------
    // 查询操作
    // ------------------------------------------------------------------------
    
    /// 按时间范围查询帧 ID 列表
    pub async fn query_frame_ids_by_time_range(&self, start: u64, end: u64) -> Result<Vec<u32>, String> {
        self.db.query_frame_ids_by_time_range(start, end).await
    }
    
    /// 按时间范围加载 FrameData 帧
    pub async fn load_frame_data_by_time_range(&self, start: u64, end: u64) -> Result<Vec<FrameData>, String> {
        let frame_ids = self.db.query_frame_ids_by_time_range(start, end).await?;
        
        let mut frames = Vec::new();
        for frame_id in frame_ids {
            if let Ok(frame) = self.load_frame_data(frame_id).await {
                frames.push(frame);
            }
        }
        
        Ok(frames)
    }
    
    /// 获取所有帧 ID
    pub async fn get_all_frame_ids(&self) -> Result<Vec<u32>, String> {
        self.db.get_all_frame_ids().await
    }
    
    /// 加载所有 FrameData 帧
    pub async fn load_all_frame_data(&self) -> Result<Vec<FrameData>, String> {
        let frame_ids = self.db.get_all_frame_ids().await?;
        
        let mut frames = Vec::new();
        for frame_id in frame_ids {
            if let Ok(frame) = self.load_frame_data(frame_id).await {
                frames.push(frame);
            }
        }
        
        Ok(frames)
    }
    
    // ------------------------------------------------------------------------
    // 删除操作
    // ------------------------------------------------------------------------
    
    /// 删除指定帧
    pub async fn delete_frame(&self, frame_id: u32) -> Result<(), String> {
        let data_path = self.db.delete_frame_record(frame_id).await?;
        
        if !data_path.is_empty() {
            let _ = fs::remove_file(&data_path);
        }
        
        Ok(())
    }
    
    /// 清空所有帧数据
    pub async fn clear_all(&self) -> Result<(), String> {
        let data_paths = self.db.clear_all_frame_records().await?;
        
        for data_path in data_paths {
            let _ = fs::remove_file(&data_path);
        }
        
        // 清理数据目录中的所有文件
        if self.db.data_dir.exists() {
            for entry in fs::read_dir(&self.db.data_dir).ok().into_iter().flatten() {
                if let Ok(entry) = entry {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
        
        Ok(())
    }
    
    // ------------------------------------------------------------------------
    // 统计信息
    // ------------------------------------------------------------------------
    
    /// 获取帧统计信息
    /// 
    /// 返回: (总帧数, 最早时间戳, 最晚时间戳)
    pub async fn get_stats(&self) -> Result<(u64, u64, u64), String> {
        self.db.get_stats().await
    }
    
    /// 获取数据目录路径
    pub fn data_dir(&self) -> &Path {
        &self.db.data_dir
    }
}
