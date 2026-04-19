use expto::rdmp::{Mesh, Transform, Unit};
use serde::{Deserialize, Serialize};

/// 帧单元结构，用于存储单帧中的网格和变换数据
#[derive(Serialize, Deserialize)]
pub struct FrameUnit {
    pub mesh: Mesh,
    pub transform: Transform,
}

/// 基础的 Memo 结构，用于存储关键帧和单元数据
#[derive(Serialize, Deserialize)]
pub struct Memo {
    pub key_frame: FrameUnit,
    pub units: Vec<Unit>,
}

/// 用于表示时间戳，支持纳秒级精度
pub type Timestamp = u64;

/// 内存中的帧数据结构
#[derive(Clone, Serialize, Deserialize)]
pub struct FrameData {
    /// 帧唯一标识符
    pub id: u64,
    /// 时间戳（纳秒）
    pub timestamp: Timestamp,
    /// 原始单元数据列表
    pub units: Vec<Unit>,
    /// 用户自定义元数据
    pub metadata: std::collections::HashMap<String, String>,
}

impl FrameData {
    /// 创建新的帧数据实例
    pub fn new(id: u64, timestamp: Timestamp) -> Self {
        Self {
            id,
            timestamp,
            units: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 添加一个单元数据
    pub fn add_unit(&mut self, unit: Unit) {
        self.units.push(unit);
    }

    /// 批量添加单元数据
    pub fn add_units(&mut self, mut units: Vec<Unit>) {
        self.units.append(&mut units);
    }

    /// 获取帧的摘要信息
    pub fn summary(&self) -> String {
        format!(
            "Frame {}: {} units",
            self.id,
            self.units.len()
        )
    }
}

/// 扩展的帧数据结构，包含更多高级功能
#[derive(Serialize, Deserialize)]
pub struct ExtendedFrameData {
    /// 基础帧数据
    pub base: FrameData,
    /// 累积变换信息
    pub transforms: std::collections::HashMap<String, Transform>,
    /// 标记的对象
    pub marked_objects: std::collections::HashSet<String>,
    /// 压缩状态
    pub is_compressed: bool,
}

impl ExtendedFrameData {
    /// 从基础帧数据创建扩展帧数据
    pub fn from_base(base: FrameData) -> Self {
        Self {
            base,
            transforms: std::collections::HashMap::new(),
            marked_objects: std::collections::HashSet::new(),
            is_compressed: false,
        }
    }

    /// 添加变换信息
    pub fn add_transform(&mut self, object_id: String, transform: Transform) {
        self.transforms.insert(object_id, transform);
    }

    /// 标记一个对象
    pub fn mark_object(&mut self, object_id: String) {
        self.marked_objects.insert(object_id);
    }

    /// 检查对象是否被标记
    pub fn is_object_marked(&self, object_id: &str) -> bool {
        self.marked_objects.contains(object_id)
    }
}

/// 帧数据管理器，用于高效管理大量帧数据
#[derive(Serialize, Deserialize)]
pub struct FrameDataManager {
    /// 帧数据容器
    pub frames: std::collections::VecDeque<ExtendedFrameData>,
    /// 按时间戳排序的索引
    pub timestamp_index: std::collections::BTreeMap<Timestamp, usize>,
    /// 按ID索引的映射
    pub id_index: std::collections::HashMap<u64, usize>,
    /// 当前播放位置
    pub current_index: Option<usize>,
}

impl FrameDataManager {
    /// 创建新的管理器实例
    pub fn new() -> Self {
        Self {
            frames: std::collections::VecDeque::new(),
            timestamp_index: std::collections::BTreeMap::new(),
            id_index: std::collections::HashMap::new(),
            current_index: None,
        }
    }

    /// 添加帧数据
    pub fn add_frame(&mut self, frame: ExtendedFrameData) {
        let frame_id = frame.base.id;
        let frame_timestamp = frame.base.timestamp;
        let index = self.frames.len();
        
        self.frames.push_back(frame);
        self.timestamp_index.insert(frame_timestamp, index);
        self.id_index.insert(frame_id, index);
        
        // 如果这是第一个帧，设置为当前位置
        if self.current_index.is_none() {
            self.current_index = Some(0);
        }
    }

    /// 获取当前帧
    pub fn current_frame(&self) -> Option<&ExtendedFrameData> {
        if let Some(index) = self.current_index {
            self.frames.get(index)
        } else {
            None
        }
    }

    /// 获取当前帧的可变引用
    pub fn current_frame_mut(&mut self) -> Option<&mut ExtendedFrameData> {
        if let Some(index) = self.current_index {
            self.frames.get_mut(index)
        } else {
            None
        }
    }

    /// 跳转到下一帧
    pub fn next_frame(&mut self) -> Option<&ExtendedFrameData> {
        if let Some(index) = self.current_index {
            if index + 1 < self.frames.len() {
                self.current_index = Some(index + 1);
                self.frames.get(index + 1)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 跳转到上一帧
    pub fn prev_frame(&mut self) -> Option<&ExtendedFrameData> {
        if let Some(index) = self.current_index {
            if index > 0 {
                self.current_index = Some(index - 1);
                self.frames.get(index - 1)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 按ID跳转到指定帧
    pub fn jump_to_frame_by_id(&mut self, id: u64) -> Option<&ExtendedFrameData> {
        if let Some(&index) = self.id_index.get(&id) {
            self.current_index = Some(index);
            self.frames.get(index)
        } else {
            None
        }
    }

    /// 按时间戳跳转到最接近的帧
    pub fn jump_to_nearest_frame_by_timestamp(&mut self, timestamp: Timestamp) -> Option<&ExtendedFrameData> {
        if self.frames.is_empty() {
            return None;
        }

        // 使用BTreeMap的范围查询找到最接近的时间戳
        let closest_entry = self.timestamp_index
            .range(..=timestamp)
            .next_back()
            .or_else(|| self.timestamp_index.range(timestamp..).next());

        if let Some((&_ts, &index)) = closest_entry {
            self.current_index = Some(index);
            self.frames.get(index)
        } else {
            None
        }
    }

    /// 获取帧数量
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// 清空所有数据
    pub fn clear(&mut self) {
        self.frames.clear();
        self.timestamp_index.clear();
        self.id_index.clear();
        self.current_index = None;
    }
}