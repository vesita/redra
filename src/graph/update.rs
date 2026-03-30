use bevy::prelude::*;

// TODO: 重新设计数据管理系统
// 当前问题：recv_and_spawn 与 record_data_frames 竞争消费 channel receiver，导致数据丢失
// 新设计目标：
// 1. 按帧管理点云数据（Frame-based）
// 2. 支持录制和回放完整的帧序列
// 3. 每帧包含：帧 ID、时间戳、点云数据列表、元数据
// 4. 实时接收的数据应该先累积到当前帧缓冲区
// 5. 当帧完成时（达到预期点数或超时），保存到录制器
// 6. 回放时按帧顺序重现整个场景状态
//
// 实现计划：
// - 移除 recv_and_spawn 的实时生成逻辑
// - 在 record.rs 中实现 FrameBuilder 和 DataFrameManager
// - 通过 UI 控制帧的切换和回放
// - 可选：实现帧的保存/加载功能

pub fn rd_update (
    _commands: Commands,
    // TODO: 暂时禁用实时生成，等待新的数据管理系统实现
    // meshes: ResMut<Assets<Mesh>>,
    // materials: ResMut<Assets<StandardMaterial>>,
    // material_manager: ResMut<MaterialManager>,
    // channel: ResMut<channels::RDChannel>,
) {
    // 此函数暂时留空，等待新的数据管理系统实现
    // 目前所有数据将通过 record_data_frames 系统记录到 DataRecorder
}