//! SQLite 存储实现（基于 sea-orm）

use sea_orm::entity::prelude::*;
use sea_orm::{Database, DatabaseConnection, Set, QuerySelect, QueryOrder, Schema};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use chrono::Utc;

#[cfg(feature = "graph")]
use bevy::prelude::*;

use crate::data::frame::{KeyFrame, Inpto};

// ============================================================================
// 实体定义
// ============================================================================

mod frames_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "frames")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = true)]
        pub frame_id: i32,
        pub timestamp: i64,
        pub created_at: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod entities_table {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "entities")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = true)]
        pub id: i64,
        pub entity_id: i64,
        pub frame_id: i32,
        pub material: String,
        pub mesh_data: Vec<u8>,
        pub tx: f32,
        pub ty: f32,
        pub tz: f32,
        pub rx: f32,
        pub ry: f32,
        pub rz: f32,
        pub rw: f32,
        pub sx: f32,
        pub sy: f32,
        pub sz: f32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod entity_tags_table {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "entity_tags")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = true)]
        pub id: i64,
        pub entity_id: i64,
        pub frame_id: i32,
        pub tag_index: i32,
        pub tag_text: String,
        pub tag_data: Vec<u8>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

// ============================================================================
// FrameStorage
// ============================================================================

/// SQLite 存储管理器。
///
/// 每个实体存储为 SQL 行，支持按帧、材质、标签查询。
/// 内部使用 sea-orm 进行 ORM 操作，通过 Tokio Runtime 桥接同步/异步。
#[cfg_attr(feature = "graph", derive(Resource))]
pub struct FrameStorage {
    conn: DatabaseConnection,
    pub(crate) db_path: PathBuf,
    rt: tokio::runtime::Runtime,
}

impl FrameStorage {
    /// 打开或创建 SQLite 数据库文件。
    pub fn new(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建数据库目录 {} 失败: {}", parent.display(), e))?;
            }
        }

        if !db_path.exists() {
            match std::fs::File::create(db_path) {
                Ok(f) => { drop(f); }
                Err(e) => {
                    return Err(format!(
                        "无法在 {} 创建数据库文件 (OS 错误: {}), \
                         请检查该目录是否有写入权限或是否是网络驱动器/SMB 挂载点", db_path.display(), e
                    ));
                }
            }
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|e| format!("创建运行时失败: {}", e))?;

        let path_str = db_path.to_string_lossy().replace('\\', "/");
        let formats = [
            format!("sqlite:{}?mode=rwc", path_str),
            format!("sqlite:///{}?mode=rwc", path_str),
            format!("sqlite:{}", path_str),
            format!("sqlite:///{}", path_str),
        ];

        let mut last_err = String::new();
        let mut conn = None;
        for conn_str in &formats {
            match rt.block_on(async { Database::connect(conn_str).await }) {
                Ok(c) => { conn = Some(c); break; }
                Err(e) => {
                    last_err = format!("{}: {}", conn_str, e);
                    log::warn!("SQLite 连接格式 {} 失败: {}", conn_str, e);
                }
            }
        }
        let conn = conn.ok_or_else(|| format!("所有 SQLite 连接格式均失败: {}", last_err))?;

        let storage = Self {
            conn,
            db_path: db_path.to_path_buf(),
            rt,
        };
        storage.init_tables()?;
        Ok(storage)
    }

    pub fn new_default() -> Result<Self, String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
            .parent()
            .ok_or("无法确定可执行文件所在目录".to_string())?
            .to_path_buf();
        let db_path = exe_path.join("storage.db");
        Self::new(&db_path)
    }

    // ── 表初始化 ──

    fn init_tables(&self) -> Result<(), String> {
        self.rt.block_on(async {
            let backend = self.conn.get_database_backend();
            let schema = Schema::new(backend);

            let stmt = backend.build(
                schema.create_table_from_entity(frames_entity::Entity).if_not_exists(),
            );
            self.conn.execute(stmt).await.map_err(|e| format!("创建表 frames 失败: {}", e))?;

            let stmt = backend.build(
                schema.create_table_from_entity(entities_table::Entity).if_not_exists(),
            );
            self.conn.execute(stmt).await.map_err(|e| format!("创建表 entities 失败: {}", e))?;

            let stmt = backend.build(
                schema.create_table_from_entity(entity_tags_table::Entity).if_not_exists(),
            );
            self.conn.execute(stmt).await.map_err(|e| format!("创建表 entity_tags 失败: {}", e))?;

            let indexes = [
                "CREATE INDEX IF NOT EXISTS idx_entities_frame ON entities(frame_id)",
                "CREATE INDEX IF NOT EXISTS idx_entities_entity ON entities(entity_id)",
                "CREATE INDEX IF NOT EXISTS idx_entities_material ON entities(material)",
                "CREATE INDEX IF NOT EXISTS idx_tags_text ON entity_tags(tag_text)",
            ];
            for sql in indexes {
                let stmt = sea_orm::Statement::from_string(backend, sql.to_owned());
                self.conn
                    .execute(stmt)
                    .await
                    .map_err(|e| format!("创建索引失败: {}", e))?;
            }

            Ok(())
        })
    }

    // ── 流式写入 ──

    /// 追加一帧到数据库，返回分配的 frame_id（由 AUTOINCREMENT 自动生成）。
    pub fn append_frame(&self, keyframe: &KeyFrame) -> Result<i32, String> {
        self.rt.block_on(async {
            let now = Utc::now().timestamp() as i64;

            // 插入帧记录，由 AUTOINCREMENT 自动分配 frame_id
            let insert_result = frames_entity::ActiveModel {
                frame_id: Default::default(),
                timestamp: Set(keyframe.timestamp as i64),
                created_at: Set(now),
            }
            .insert(&self.conn)
            .await
            .map_err(|e| format!("插入帧记录失败: {}", e))?;

            let next_id = insert_result.frame_id;

            for (entity_id, inpto) in keyframe.iter_entities() {
                let mesh_data = bincode::serialize(&inpto.mesh)
                    .map_err(|e| format!("序列化 mesh 失败: {}", e))?;

                let t = &inpto.transform;
                entities_table::ActiveModel {
                    id: Default::default(),
                    entity_id: Set(entity_id as i64),
                    frame_id: Set(next_id),
                    material: Set(inpto.material.clone()),
                    mesh_data: Set(mesh_data),
                    tx: Set(t.tx),
                    ty: Set(t.ty),
                    tz: Set(t.tz),
                    rx: Set(t.rx),
                    ry: Set(t.ry),
                    rz: Set(t.rz),
                    rw: Set(t.rw),
                    sx: Set(t.sx),
                    sy: Set(t.sy),
                    sz: Set(t.sz),
                }
                .insert(&self.conn)
                .await
                .map_err(|e| format!("插入实体失败: {}", e))?;

                for (ti, tag) in inpto.tags.iter().enumerate() {
                    let tag_data = bincode::serialize(tag)
                        .map_err(|e| format!("序列化 tag 失败: {}", e))?;
                    entity_tags_table::ActiveModel {
                        id: Default::default(),
                        entity_id: Set(entity_id as i64),
                        frame_id: Set(next_id),
                        tag_index: Set(ti as i32),
                        tag_text: Set(tag.text.clone()),
                        tag_data: Set(tag_data),
                    }
                    .insert(&self.conn)
                    .await
                    .map_err(|e| format!("插入标签失败: {}", e))?;
                }
            }

            log::info!(
                "帧 {} 已写入数据库 ({} 个实体)",
                next_id,
                keyframe.entity_count()
            );
            Ok(next_id)
        })
    }

    /// 批量追加多帧。
    pub fn append_frames(&self, keyframes: &[KeyFrame]) -> Result<(), String> {
        for kf in keyframes {
            self.append_frame(kf)?;
        }
        Ok(())
    }

    // ── 读取 ──

    /// 加载指定帧。
    pub fn load_frame(&self, frame_id: i32) -> Result<KeyFrame, String> {
        self.rt.block_on(async {
            let frame = frames_entity::Entity::find_by_id(frame_id)
                .one(&self.conn)
                .await
                .map_err(|e| format!("查询帧失败: {}", e))?
                .ok_or_else(|| format!("帧 {} 不存在", frame_id))?;

            let entity_rows = entities_table::Entity::find()
                .filter(entities_table::Column::FrameId.eq(frame_id))
                .all(&self.conn)
                .await
                .map_err(|e| format!("查询实体失败: {}", e))?;

            let tag_rows = entity_tags_table::Entity::find()
                .filter(entity_tags_table::Column::FrameId.eq(frame_id))
                .all(&self.conn)
                .await
                .map_err(|e| format!("查询标签失败: {}", e))?;

            let mut tag_map: HashMap<(i64, i32), Vec<(usize, expto::rdmp::Tag)>> =
                HashMap::new();
            for tr in &tag_rows {
                let tag: expto::rdmp::Tag = bincode::deserialize(&tr.tag_data)
                    .map_err(|e| format!("反序列化标签失败: {}", e))?;
                tag_map
                    .entry((tr.entity_id, tr.frame_id))
                    .or_default()
                    .push((tr.tag_index as usize, tag));
            }

            let mut keyframe = KeyFrame::new(frame.timestamp as u64);

            for er in &entity_rows {
                let mesh: expto::rdmp::ExMesh = bincode::deserialize(&er.mesh_data)
                    .map_err(|e| format!("反序列化 mesh 失败: {}", e))?;

                let transform = crate::data::frame::inpto::InptoTransform {
                    tx: er.tx, ty: er.ty, tz: er.tz,
                    rx: er.rx, ry: er.ry, rz: er.rz, rw: er.rw,
                    sx: er.sx, sy: er.sy, sz: er.sz,
                };

                let mut tags = Vec::new();
                if let Some(entry) = tag_map.get(&(er.entity_id, er.frame_id)) {
                    let mut sorted = entry.clone();
                    sorted.sort_by_key(|(idx, _)| *idx);
                    for (_, tag) in sorted {
                        tags.push(tag);
                    }
                }

                let inpto = Inpto {
                    mesh,
                    material: er.material.clone(),
                    transform,
                    tags,
                };
                keyframe.ids.insert(er.entity_id as u64, keyframe.packs.len());
                keyframe.packs.push(inpto);
            }

            Ok(keyframe)
        })
    }

    /// 加载所有帧。
    pub fn load_all_frames(&self) -> Result<Vec<KeyFrame>, String> {
        let ids = self.get_all_frame_ids()?;
        let mut frames = Vec::with_capacity(ids.len());
        for id in ids {
            frames.push(self.load_frame(id)?);
        }
        Ok(frames)
    }

    // ── 查询 ──

    pub fn query_by_material(&self, material: &str) -> Result<Vec<(i32, u64)>, String> {
        self.rt.block_on(async {
            let rows = entities_table::Entity::find()
                .filter(entities_table::Column::Material.eq(material))
                .select_only()
                .column(entities_table::Column::FrameId)
                .column(entities_table::Column::EntityId)
                .into_tuple::<(i32, i64)>()
                .all(&self.conn)
                .await
                .map_err(|e| format!("按材质查询失败: {}", e))?;
            Ok(rows.into_iter().map(|(fid, eid)| (fid, eid as u64)).collect())
        })
    }

    pub fn query_by_tag(&self, tag_text: &str) -> Result<Vec<(i32, u64)>, String> {
        self.rt.block_on(async {
            let rows = entity_tags_table::Entity::find()
                .filter(entity_tags_table::Column::TagText.eq(tag_text))
                .select_only()
                .column(entity_tags_table::Column::FrameId)
                .column(entity_tags_table::Column::EntityId)
                .into_tuple::<(i32, i64)>()
                .all(&self.conn)
                .await
                .map_err(|e| format!("按标签查询失败: {}", e))?;
            Ok(rows.into_iter().map(|(fid, eid)| (fid, eid as u64)).collect())
        })
    }

    pub fn frame_count(&self) -> Result<u64, String> {
        self.rt.block_on(async {
            frames_entity::Entity::find()
                .count(&self.conn)
                .await
                .map_err(|e| format!("统计帧数失败: {}", e))
        })
    }

    pub fn entity_count(&self) -> Result<u64, String> {
        self.rt.block_on(async {
            entities_table::Entity::find()
                .count(&self.conn)
                .await
                .map_err(|e| format!("统计实体数失败: {}", e))
        })
    }

    pub fn get_all_frame_ids(&self) -> Result<Vec<i32>, String> {
        self.rt.block_on(async {
            use frames_entity::Column;
            let rows = frames_entity::Entity::find()
                .select_only()
                .column(Column::FrameId)
                .order_by_asc(Column::FrameId)
                .into_tuple::<(i32,)>()
                .all(&self.conn)
                .await
                .map_err(|e| format!("查询帧 ID 列表失败: {}", e))?;
            Ok(rows.into_iter().map(|(id,)| id).collect())
        })
    }

    // ── 备份/导出 ──

    /// 将当前数据库导出到指定路径（文件复制 + VACUUM）。
    pub fn export_db(&self, dest: &Path) -> Result<(), String> {
        self.vacuum()?;
        std::fs::copy(&self.db_path, dest)
            .map_err(|e| format!("导出数据库失败: {}", e))?;
        log::info!("数据库已导出到: {}", dest.display());
        Ok(())
    }

    // ── 维护 ──

    /// 清空所有数据。
    pub fn clear_all(&self) -> Result<(), String> {
        self.rt.block_on(async {
            entity_tags_table::Entity::delete_many()
                .exec(&self.conn)
                .await
                .map_err(|e| format!("删除标签失败: {}", e))?;
            entities_table::Entity::delete_many()
                .exec(&self.conn)
                .await
                .map_err(|e| format!("删除实体失败: {}", e))?;
            frames_entity::Entity::delete_many()
                .exec(&self.conn)
                .await
                .map_err(|e| format!("删除帧记录失败: {}", e))?;
            Ok(())
        })
    }

    /// VACUUM（压缩数据库文件）。
    pub fn vacuum(&self) -> Result<(), String> {
        self.rt.block_on(async {
            let backend = self.conn.get_database_backend();
            let stmt = sea_orm::Statement::from_string(backend, "VACUUM".to_owned());
            self.conn
                .execute(stmt)
                .await
                .map_err(|e| format!("VACUUM 失败: {}", e))?;
            Ok(())
        })
    }
}
