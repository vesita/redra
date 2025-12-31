use bevy::{
    mesh::Mesh,
    transform::components::Transform,
};
use std::sync::Arc;

/// RDPack 枚举定义了网络通信中可能传输的各种数据包类型
/// 包括消息、形状数据和格式数据等
#[derive(Clone)]
pub enum RDPack {
    Message(String),                    // 消息数据包，包含字符串内容
    SpawnShape(Box<RDShapePack>),      // 形状生成数据包，包含形状的网格、变换和材质信息
    SpawnFormat(Box<FormatPack>),      // 格式数据包，用于传输特定格式的数据
}

/// RDShapePack 结构体定义了形状实体的完整描述
/// 包含网格、变换矩阵和材质信息
#[derive(Clone)]
pub struct RDShapePack {
    pub mesh: Arc<Mesh>,              // 形状的网格数据，使用 Arc 共享以提高性能
    pub transform: Transform,         // 形状的变换信息（位置、旋转、缩放）
    pub material: String,             // 形状的材质名称
}

/// FormatPack 结构体定义了格式数据包的内容
/// 目前为空，可根据需要扩展
#[derive(Clone)]
pub struct FormatPack {

}