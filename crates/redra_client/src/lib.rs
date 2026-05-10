//! Redra 客户端 SDK — 构建并发送 3D 可视化数据到 redra 服务端
//!
//! # 两种 API 风格
//!
//! - **链式构建器** [`ShapeBuilder`] — 推荐，支持 ID、材质、标签、分组点云
//! - **便捷函数** [`send_sphere`] / [`send_cube`] 等 — 单行调用，适合快速原型
//!
//! # 材质体系
//!
//! 通过 [`defaults`] 模块访问预设材质：
//! - `defaults::base::*` — 7 种基础色（`"red"`, `"green"`, ...）
//! - `defaults::cluster::*` — 12 种聚类色板（`"cluster_01"` ~ `"cluster_12"`）
//! - `defaults::semantic::*` — 语义色（`"ground"`, `"alert"`, ...）
//! - `defaults::effects::*` — 效果材质（`"glass"`, `"metal"`, ...）

pub mod client;
pub mod defaults;

// 导出 expto 中的重要类型，方便使用者
pub use expto::prelude::*;

// 导出 send 模块中的所有便捷函数
pub use client::send::*;

// 导出 builder 模块（ShapeBuilder + 便捷函数）
pub use client::builder::*;

// 导出 writer 模块（RdraWriter）
pub use client::writer::*;

// 导出 sql_writer 模块（SqlWriter）
pub use client::sql_writer::SqlWriter;

