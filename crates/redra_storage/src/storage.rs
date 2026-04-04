//! 基于 SQLite 的数据持久化存储模块

use rusqlite::{Connection, params, Result as SqliteResult};
use std::path::{Path, PathBuf};
use std::fs;
use log::warn;
use std::env;

/// 帧元数据（存储在SQLite中）
#[derive(Debug, Clone)]
pub struct FrameMetadata {
    pub frame_id: u32,
    pub sequence_number: u64,
    pub timestamp: u64,
    pub point_count: u32,
    pub frame_type: String,
    pub data_path: String,  // 二进制数据文件路径
}

/// 帧类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum FrameType {
    PCD,           // PCD文件数据
    REALTIME,      // 实时传感器数据
    MANUAL,        // 手动标记的帧
}

impl ToString for FrameType {
    fn to_string(&self) -> String {
        match self {
            FrameType::PCD => "pcd".to_string(),
            FrameType::REALTIME => "realtime".to_string(),
            FrameType::MANUAL => "manual".to_string(),
        }
    }
}

impl From<&str> for FrameType {
    fn from(s: &str) -> Self {
        match s {
            "pcd" => FrameType::PCD,
            "realtime" => FrameType::REALTIME,
            "manual" => FrameType::MANUAL,
            _ => FrameType::REALTIME,
        }
    }
}

/// SQLite数据库管理器
pub struct FrameDatabase {
    conn: Connection,
    data_dir: PathBuf,
}

impl FrameDatabase {
    /// 创建或打开数据库
    pub fn new(db_path: &Path, data_dir: &Path) -> Result<Self, String> {
        // 确保目录存在
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建数据库目录失败: {}", e))?;
        }
        fs::create_dir_all(data_dir)
            .map_err(|e| format!("创建数据目录失败: {}", e))?;
        
        // 打开数据库连接
        let conn = Connection::open(db_path)
            .map_err(|e| format!("打开数据库失败: {}", e))?;
        
        // 启用WAL模式以获得更好的并发性能（使用query而不是execute）
        if let Err(e) = conn.pragma_update(None, "journal_mode", "WAL") {
            warn!("启用WAL模式失败: {}，使用默认模式", e);
        }
        
        // 优化写入性能
        conn.execute("PRAGMA synchronous=NORMAL", [])
            .map_err(|e| format!("设置同步方式失败: {}", e))?;
        
        // 增加缓存大小（单位：页，每页4KB）
        conn.execute("PRAGMA cache_size=-20000", [])  // 大约80MB缓存
            .map_err(|e| format!("设置缓存大小失败: {}", e))?;
        
        let mut db = FrameDatabase {
            conn,
            data_dir: data_dir.to_path_buf(),
        };
        
        // 初始化表结构
        db.init_tables()?;
        
        Ok(db)
    }
    
    /// 初始化数据库表
    fn init_tables(&mut self) -> Result<(), String> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS frames (
                frame_id INTEGER PRIMARY KEY,
                sequence_number INTEGER UNIQUE NOT NULL,
                timestamp INTEGER NOT NULL,
                point_count INTEGER NOT NULL,
                frame_type TEXT NOT NULL,
                data_path TEXT NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        ).map_err(|e| format!("创建frames表失败: {}", e))?;
        
        // 创建索引以加速查询
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_frames_timestamp ON frames(timestamp)",
            [],
        ).map_err(|e| format!("创建时间戳索引失败: {}", e))?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_frames_sequence ON frames(sequence_number)",
            [],
        ).map_err(|e| format!("创建序列号索引失败: {}", e))?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_frames_type ON frames(frame_type)",
            [],
        ).map_err(|e| format!("创建类型索引失败: {}", e))?;
        
        Ok(())
    }
    
    /// 插入单个帧
    pub fn insert_frame(&self, frame: &FrameMetadata) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO frames 
             (frame_id, sequence_number, timestamp, point_count, frame_type, data_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                frame.frame_id as i64,
                frame.sequence_number as i64,
                frame.timestamp as i64,
                frame.point_count as i64,
                &frame.frame_type,
                &frame.data_path
            ],
        )?;
        Ok(())
    }
    
    /// 批量插入帧（使用事务，性能更好）
    pub fn insert_frames_batch(&mut self, frames: &[FrameMetadata]) -> SqliteResult<()> {
        let tx = self.conn.transaction()?;
        
        for frame in frames {
            tx.execute(
                "INSERT OR REPLACE INTO frames 
                 (frame_id, sequence_number, timestamp, point_count, frame_type, data_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    frame.frame_id as i64,
                    frame.sequence_number as i64,
                    frame.timestamp as i64,
                    frame.point_count as i64,
                    &frame.frame_type,
                    &frame.data_path
                ],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }
    
    /// 按时间范围查询帧
    pub fn query_by_time_range(&self, start: u64, end: u64) -> SqliteResult<Vec<FrameMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT frame_id, sequence_number, timestamp, point_count, frame_type, data_path
             FROM frames
             WHERE timestamp BETWEEN ?1 AND ?2
             ORDER BY timestamp"
        )?;
        
        let frames = stmt.query_map(params![start as i64, end as i64], |row| {
            Ok(FrameMetadata {
                frame_id: row.get::<_, i64>(0)? as u32,
                sequence_number: row.get::<_, i64>(1)? as u64,
                timestamp: row.get::<_, i64>(2)? as u64,
                point_count: row.get::<_, i64>(3)? as u32,
                frame_type: row.get::<_, String>(4)?.as_str().into(),
                data_path: row.get::<_, String>(5)?,
            })
        })?;
        
        Ok(frames.filter_map(|f| f.ok()).collect())
    }
    
    /// 按序列号查询单个帧
    pub fn get_frame_by_sequence(&self, seq: u64) -> SqliteResult<Option<FrameMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT frame_id, sequence_number, timestamp, point_count, frame_type, data_path
             FROM frames
             WHERE sequence_number = ?1"
        )?;
        
        let mut frames = stmt.query_map(params![seq as i64], |row| {
            Ok(FrameMetadata {
                frame_id: row.get::<_, i64>(0)? as u32,
                sequence_number: row.get::<_, i64>(1)? as u64,
                timestamp: row.get::<_, i64>(2)? as u64,
                point_count: row.get::<_, i64>(3)? as u32,
                frame_type: row.get::<_, String>(4)?.as_str().into(),
                data_path: row.get::<_, String>(5)?,
            })
        })?;
        
        Ok(frames.next().and_then(|f| f.ok()))
    }
    
    /// 获取所有帧
    pub fn get_all_frames(&self) -> SqliteResult<Vec<FrameMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT frame_id, sequence_number, timestamp, point_count, frame_type, data_path
             FROM frames
             ORDER BY timestamp"
        )?;
        
        let frames = stmt.query_map(params![], |row| {
            Ok(FrameMetadata {
                frame_id: row.get::<_, i64>(0)? as u32,
                sequence_number: row.get::<_, i64>(1)? as u64,
                timestamp: row.get::<_, i64>(2)? as u64,
                point_count: row.get::<_, i64>(3)? as u32,
                frame_type: row.get::<_, String>(4)?.as_str().into(),
                data_path: row.get::<_, String>(5)?,
            })
        })?;
        
        Ok(frames.filter_map(|f| f.ok()).collect())
    }
    
    /// 按条件过滤帧
    pub fn filter_frames<F>(&self, predicate: F) -> SqliteResult<Vec<FrameMetadata>>
    where
        F: Fn(&FrameMetadata) -> bool,
    {
        let all_frames = self.get_all_frames()?;
        Ok(all_frames.into_iter().filter(predicate).collect())
    }
    
    /// 删除指定帧
    pub fn delete_frame(&self, frame_id: u32) -> SqliteResult<usize> {
        // 首先获取data_path以删除文件
        if let Ok(Some(frame)) = self.get_frame_by_id(frame_id) {
            let _ = fs::remove_file(&frame.data_path);
        }
        
        self.conn.execute(
            "DELETE FROM frames WHERE frame_id = ?1",
            params![frame_id as i64],
        )
    }
    
    /// 根据ID获取帧
    pub fn get_frame_by_id(&self, frame_id: u32) -> SqliteResult<Option<FrameMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT frame_id, sequence_number, timestamp, point_count, frame_type, data_path
             FROM frames
             WHERE frame_id = ?1"
        )?;
        
        let mut frames = stmt.query_map(params![frame_id as i64], |row| {
            Ok(FrameMetadata {
                frame_id: row.get::<_, i64>(0)? as u32,
                sequence_number: row.get::<_, i64>(1)? as u64,
                timestamp: row.get::<_, i64>(2)? as u64,
                point_count: row.get::<_, i64>(3)? as u32,
                frame_type: row.get::<_, String>(4)?.as_str().into(),
                data_path: row.get::<_, String>(5)?,
            })
        })?;
        
        Ok(frames.next().and_then(|f| f.ok()))
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> SqliteResult<FrameStats> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                COUNT(*) as total_frames,
                MIN(timestamp) as first_timestamp,
                MAX(timestamp) as last_timestamp,
                SUM(point_count) as total_points
             FROM frames"
        )?;
        
        let stats = stmt.query_row(params![], |row| {
            Ok(FrameStats {
                total_frames: row.get::<_, i64>(0)? as u64,
                first_timestamp: row.get::<_, i64>(1)? as u64,
                last_timestamp: row.get::<_, i64>(2)? as u64,
                total_points: row.get::<_, i64>(3)? as u64,
            })
        })?;
        
        Ok(stats)
    }
    
    /// 清除所有数据
    pub fn clear_all(&self) -> SqliteResult<()> {
        // 删除所有数据文件
        if self.data_dir.exists() {
            for entry in fs::read_dir(&self.data_dir).ok().into_iter().flatten() {
                if let Ok(entry) = entry {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
        
        self.conn.execute("DELETE FROM frames", [])?;
        Ok(())
    }
    
    /// 获取数据目录路径
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}

/// 帧统计信息
#[derive(Debug)]
pub struct FrameStats {
    pub total_frames: u64,
    pub first_timestamp: u64,
    pub last_timestamp: u64,
    pub total_points: u64,
}

/// 帧数据存储（管理二进制文件和元数据）
pub struct FrameStorage {
    db: FrameDatabase,
}

impl FrameStorage {
    /// 创建新的存储
    pub fn new(base_path: &Path) -> Result<Self, String> {
        let db_path = base_path.join("frames.db");
        let data_dir = base_path.join("data");
        
        let db = FrameDatabase::new(&db_path, &data_dir)?;
        
        Ok(FrameStorage { db })
    }

    /// 使用默认路径（相对于可执行文件）创建新的存储
    pub fn new_default() -> Result<Self, String> {
        // 获取当前可执行文件的路径
        let exe_path = env::current_exe()
            .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
            .parent()
            .ok_or("无法确定可执行文件所在目录".to_string())
            .map(|p| p.to_path_buf())?;

        let base_path = exe_path.join("storage");

        Self::new(&base_path)
    }
    
    /// 保存帧数据
    pub fn save_frame(&self, frame_data: &[u8], metadata: FrameMetadata) -> Result<(), String> {
        // 生成唯一文件名
        let file_path = self.db.data_dir()
            .join(format!("frame_{}.bin", metadata.frame_id));
        
        // 写入二进制数据
        fs::write(&file_path, frame_data)
            .map_err(|e| format!("写入帧数据失败: {}", e))?;
        
        // 更新元数据中的路径
        let mut meta = metadata;
        meta.data_path = file_path.to_string_lossy().to_string();
        
        // 保存到数据库
        self.db.insert_frame(&meta)
            .map_err(|e| format!("保存帧元数据失败: {}", e))?;
        
        Ok(())
    }
    
    /// 加载帧数据
    pub fn load_frame(&self, frame_id: u32) -> Result<Vec<u8>, String> {
        let metadata = self.db.get_frame_by_id(frame_id)
            .map_err(|e| format!("获取帧元数据失败: {}", e))?;
        
        if let Some(meta) = metadata {
            fs::read(&meta.data_path)
                .map_err(|e| format!("读取帧数据失败: {}", e))
        } else {
            Err(format!("帧 {} 未找到", frame_id))
        }
    }
    
    /// 获取数据库引用
    pub fn database(&self) -> &FrameDatabase {
        &self.db
    }
}