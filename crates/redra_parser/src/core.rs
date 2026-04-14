use nalgebra::Vector3;
use std::sync::Arc;
#[cfg(feature = "graph")]
use bevy::prelude::{Mesh, Component};  // 仅在启用graph功能时导入
#[cfg(not(feature = "graph"))]
use bevy::prelude::Mesh;  // 在其他情况下只需导入Mesh

// 定义数据包类型枚举
#[derive(Clone, Debug)]
pub enum RDPack {
    Message(String),
    SpawnShape(Box<InternalShapePack>),
    SpawnFormat(Box<InternalFormatPack>),
    PointCloud(InternalPointCloudPack),
    // 可能还有其他类型...
}

// 定义格式数据包结构
#[derive(Clone, Debug)]
pub struct InternalFormatPack {
    pub data: InternalFormatData,
}

#[derive(Clone, Debug)]
pub enum InternalFormatData {
    Image(InternalImageData),
    Text(InternalTextData),
    Model(InternalModelData),
    Audio(InternalAudioData),
    Video(InternalVideoData),
}

// 定义图像数据
#[derive(Clone, Debug)]
pub struct InternalImageData {
    pub data: Vec<u8>,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
}

// 定义文本数据
#[derive(Clone, Debug)]
pub struct InternalTextData {
    pub content: String,
    pub language: String,
    pub encoding: String,
}

// 定义模型数据
#[derive(Clone, Debug)]
pub struct InternalModelData {
    pub data: Vec<u8>,
    pub format: String,
    pub textures: Vec<String>,
}

// 定义音频数据
#[derive(Clone, Debug)]
pub struct InternalAudioData {
    pub data: Vec<u8>,
    pub format: String,
    pub sample_rate: u32,
    pub channels: u32,
}

// 定义视频数据
#[derive(Clone, Debug)]
pub struct InternalVideoData {
    pub data: Vec<u8>,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub duration: f32,
}

// 定义形状数据包结构
#[derive(Clone, Debug)]
pub struct InternalShapePack {
    pub mesh: Arc<Mesh>,
    pub transform: bevy::prelude::Transform,
    pub material: String,
    pub source: Option<String>,  // 新增来源信息
}

// 定义点云数据包结构
#[derive(Clone, Debug)]
#[cfg_attr(feature = "graph", derive(Component))]  // 仅在启用graph功能时添加Component派生
pub struct InternalPointCloudPack {
    pub frame_id: u32,
    pub timestamp: f64,
    pub points: Vec<(f32, f32, f32)>,
}

// 定义几何数据类型
#[derive(Clone, Debug)]
pub enum InternalShapeGeometry {
    Point { position: Vector3<f32> },
    Segment { start: Vector3<f32>, end: Vector3<f32> },
    Sphere { center: Vector3<f32>, radius: f32 },
    Cube { center: Vector3<f32>, size: Vector3<f32> },
    // ... 其他形状
}

// 定义姿态数据
#[derive(Clone, Debug)]
pub struct InternalPoseData {
    pub translation: Vector3<f32>,
    pub rotation: nalgebra::Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Default for InternalPoseData {
    fn default() -> Self {
        Self {
            translation: Vector3::new(0.0, 0.0, 0.0),
            rotation: nalgebra::Quaternion::from_real(1.0),  // 修复这里
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}