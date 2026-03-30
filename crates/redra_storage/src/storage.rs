//! 基于 SQLite 的数据持久化存储模块

use rusqlite::{Connection, params, Result as SqliteResult};
use std::path::{Path, PathBuf};
use std::fs;
use log::warn;

/// Frame metadata (stored in SQLite)
#[derive(Debug, Clone)]
pub struct FrameMetadata {
    pub frame_id: u32,
    pub sequence_number: u64,
    pub timestamp: u64,
    pub point_count: u32,
    pub frame_type: String,
    pub data_path: String,  // Binary data file path
}

/// Frame type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum FrameType {
    PCD,           // PCD file data
    REALTIME,      // Real-time sensor data
    MANUAL,        // Manually marked frames
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

/// SQLite database manager
pub struct FrameDatabase {
    conn: Connection,
    data_dir: PathBuf,
}

impl FrameDatabase {
    /// Create or open database
    pub fn new(db_path: &Path, data_dir: &Path) -> Result<Self, String> {
        // Ensure directories exist
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database directory: {}", e))?;
        }
        fs::create_dir_all(data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
        
        // Open database connection
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        // Enable WAL mode for better concurrent performance (use query instead of execute)
        if let Err(e) = conn.execute("PRAGMA journal_mode=WAL", []) {
            warn!("Failed to enable WAL mode: {}, using default mode", e);
        }
        
        // Optimize write performance
        conn.execute("PRAGMA synchronous=NORMAL", [])
            .map_err(|e| format!("Failed to set synchronous: {}", e))?;
        
        // Increase cache size (unit: pages, 4KB per page)
        conn.execute("PRAGMA cache_size=-20000", [])  // About 80MB cache
            .map_err(|e| format!("Failed to set cache size: {}", e))?;
        
        let mut db = FrameDatabase {
            conn,
            data_dir: data_dir.to_path_buf(),
        };
        
        // Initialize table structure
        db.init_tables()?;
        
        Ok(db)
    }
    
    /// Initialize database tables
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
        ).map_err(|e| format!("Failed to create frames table: {}", e))?;
        
        // Create indexes to accelerate queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_frames_timestamp ON frames(timestamp)",
            [],
        ).map_err(|e| format!("Failed to create timestamp index: {}", e))?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_frames_sequence ON frames(sequence_number)",
            [],
        ).map_err(|e| format!("Failed to create sequence index: {}", e))?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_frames_type ON frames(frame_type)",
            [],
        ).map_err(|e| format!("Failed to create type index: {}", e))?;
        
        Ok(())
    }
    
    /// Insert a single frame
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
    
    /// Batch insert frames (using transactions, better performance)
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
    
    /// Query frames by time range
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
    
    /// Query single frame by sequence number
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
    
    /// Get all frames
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
    
    /// Filter frames by condition
    pub fn filter_frames<F>(&self, predicate: F) -> SqliteResult<Vec<FrameMetadata>>
    where
        F: Fn(&FrameMetadata) -> bool,
    {
        let all_frames = self.get_all_frames()?;
        Ok(all_frames.into_iter().filter(predicate).collect())
    }
    
    /// Delete specified frame
    pub fn delete_frame(&self, frame_id: u32) -> SqliteResult<usize> {
        // First get data_path to delete the file
        if let Ok(Some(frame)) = self.get_frame_by_id(frame_id) {
            let _ = fs::remove_file(&frame.data_path);
        }
        
        self.conn.execute(
            "DELETE FROM frames WHERE frame_id = ?1",
            params![frame_id as i64],
        )
    }
    
    /// Get frame by ID
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
    
    /// Get statistics
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
    
    /// Clear all data
    pub fn clear_all(&self) -> SqliteResult<()> {
        // Delete all data files
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
    
    /// Get data directory path
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}

/// Frame statistics
#[derive(Debug)]
pub struct FrameStats {
    pub total_frames: u64,
    pub first_timestamp: u64,
    pub last_timestamp: u64,
    pub total_points: u64,
}

/// Frame data storage (manages binary files and metadata)
pub struct FrameStorage {
    db: FrameDatabase,
}

impl FrameStorage {
    /// Create new storage
    pub fn new(base_path: &Path) -> Result<Self, String> {
        let db_path = base_path.join("frames.db");
        let data_dir = base_path.join("data");
        
        let db = FrameDatabase::new(&db_path, &data_dir)?;
        
        Ok(FrameStorage { db })
    }
    
    /// Save frame data
    pub fn save_frame(&self, frame_data: &[u8], metadata: FrameMetadata) -> Result<(), String> {
        // Generate unique filename
        let file_path = self.db.data_dir()
            .join(format!("frame_{}.bin", metadata.frame_id));
        
        // Write binary data
        fs::write(&file_path, frame_data)
            .map_err(|e| format!("Failed to write frame data: {}", e))?;
        
        // Update path in metadata
        let mut meta = metadata;
        meta.data_path = file_path.to_string_lossy().to_string();
        
        // Save to database
        self.db.insert_frame(&meta)
            .map_err(|e| format!("Failed to save frame metadata: {}", e))?;
        
        Ok(())
    }
    
    /// Load frame data
    pub fn load_frame(&self, frame_id: u32) -> Result<Vec<u8>, String> {
        let metadata = self.db.get_frame_by_id(frame_id)
            .map_err(|e| format!("Failed to get frame metadata: {}", e))?;
        
        if let Some(meta) = metadata {
            fs::read(&meta.data_path)
                .map_err(|e| format!("Failed to read frame data: {}", e))
        } else {
            Err(format!("Frame {} not found", frame_id))
        }
    }
    
    /// Get database reference
    pub fn database(&self) -> &FrameDatabase {
        &self.db
    }
}
