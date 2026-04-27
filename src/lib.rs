// 架构分层（自上而下依赖）：
// - control:   编排层，协调各模块的工作流
// - data:      数据层，纯帧数据管理与协议转换
// - assets:    资源层，材质/字体等资产的加载与管理
// - render:    渲染层，Bevy 场景渲染、交互、UI
// - ui:        UI 层，用户界面（基于 egui）

pub mod control;
pub mod data;
pub mod assets;
pub mod render;
pub mod ui;

// 导出子 crate
pub extern crate expto;

// 导出主要插件
pub use control::ControlPlugin as RedraPlugin;
