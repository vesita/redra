use expto::rdmp::{CommandType, Unit};

use crate::manager::data::frame::{FrameManager, KeyFrame, UnitPack};


impl FrameManager {
    pub fn new() -> Self {
        Self {
            current_frame: 0,
            timestamp: 0,
            keyframes: Vec::new(),
            frames: Vec::new(),
            temp_units: Vec::new(),
            temp_keyframe: None,
            first_temp_unit_timestamp: None,
        }
    }

    pub fn add_keyframe(&mut self, keyframe: KeyFrame) {
        self.keyframes.push(keyframe);
    }

    pub fn add_frame(&mut self, frame: UnitPack) {
        self.frames.push(frame);
    }

    pub fn submit(&mut self, unit: &Unit) {
        match unit.command {
            Some(cmd) => {
                if let Some(command_type) = CommandType::try_from(cmd.u_command).ok() {
                    match command_type {
                        CommandType::Frameend => {
                            // 创建当前帧并添加
                            self.add_frame(UnitPack::new_frame(self.last_keyframe_idx(), &self.temp_units));
                            
                            // 同时生成 KeyFrame（供渲染器使用）
                            if !self.temp_units.is_empty() {
                                let mut keyframe = KeyFrame::new(self.timestamp);
                                for temp_unit in &self.temp_units {
                                    keyframe.update(temp_unit);
                                }
                                self.add_keyframe(keyframe);
                                log::info!("帧管理器：完成一帧，包含 {} 个 Unit，生成 KeyFrame", self.temp_units.len());
                            } else {
                                log::warn!("帧管理器：收到 Frameend 但 temp_units 为空");
                            }
                            
                            // 清空临时单元，为下一帧做准备
                            self.temp_units.clear();
                            // 重置时间戳跟踪
                            self.first_temp_unit_timestamp = None;
                        },
                        _ => {
                            // 记录第一个 temp_unit 的时间戳
                            if self.temp_units.is_empty() {
                                self.first_temp_unit_timestamp = unit.stamp.as_ref().map(|s| s.timestamp);
                            }
                            self.temp_units.push(unit.clone());
                        }
                    }
                }
            },
            None => {
                // 没有命令的单元也加入临时列表
                if self.temp_units.is_empty() {
                    self.first_temp_unit_timestamp = unit.stamp.as_ref().map(|s| s.timestamp);
                }
                self.temp_units.push(unit.clone());
            },
        }
    }

    pub fn submit_units(&mut self, units: &[Unit]) {
        for unit in units {
            self.submit(unit);
        }
    }

    pub fn last_keyframe_idx(&self) -> Option<usize> {
        self.keyframes.len().checked_sub(1)
    }

    pub fn generate_keyframe(&mut self) { 
        if self.should_generate_keyframe() {
            let mut temp_keyframe = self.temp_keyframe.take().unwrap_or_else(|| {
                KeyFrame::new(self.timestamp)
            });
            for unit in self.temp_units.drain(..) { 
                temp_keyframe.update(&unit);
            }
            self.add_keyframe(temp_keyframe);
        }
    }

    pub fn should_generate_keyframe(&self) -> bool { 
        if self.temp_units.is_empty() {
            return false;
        }

        // 条件1: 数量达到阈值
        if self.temp_units.len() >= 100 {
            return true;
        }

        // 条件2: 超时判断（5秒 = 5000毫秒）
        if let Some(first_timestamp) = self.first_temp_unit_timestamp {
            if let Some(last_unit_stamp) = self.temp_units.last().and_then(|u| u.stamp.as_ref()) {
                let current_timestamp = last_unit_stamp.timestamp;
                if current_timestamp.saturating_sub(first_timestamp) >= 5000 {
                    return true;
                }
            }
        }

        false
    }

    pub fn current_frame(&self) -> &KeyFrame { 
        self.keyframes.get(self.current_frame).unwrap()
    }

    // ==================== 数据访问接口（供 FrameRenderer 使用）====================

    /// 获取当前帧索引
    pub fn current_frame_index(&self) -> usize {
        self.current_frame
    }

    /// 获取当前关键帧的可变引用
    pub fn get_current_keyframe(&self) -> Option<&KeyFrame> {
        self.keyframes.get(self.current_frame)
    }

    /// 获取指定索引的关键帧
    pub fn get_keyframe(&self, index: usize) -> Option<&KeyFrame> {
        self.keyframes.get(index)
    }

    /// 获取总帧数
    pub fn total_frames(&self) -> usize {
        self.keyframes.len()
    }

    /// 切换到下一帧
    pub fn next_frame(&mut self) -> bool {
        if self.current_frame + 1 < self.keyframes.len() {
            self.current_frame += 1;
            true
        } else {
            false
        }
    }

    /// 切换到上一帧
    pub fn prev_frame(&mut self) -> bool {
        if self.current_frame > 0 {
            self.current_frame -= 1;
            true
        } else {
            false
        }
    }

    /// 跳转到指定帧
    pub fn seek_to_frame(&mut self, frame_index: usize) -> bool {
        if frame_index < self.keyframes.len() {
            self.current_frame = frame_index;
            true
        } else {
            false
        }
    }

    /// 获取所有关键帧的引用（用于序列化保存）
    pub fn get_all_keyframes(&self) -> &[KeyFrame] {
        &self.keyframes
    }

    /// 清空所有帧数据（用于加载新文件前）
    pub fn clear(&mut self) {
        self.current_frame = 0;
        self.timestamp = 0;
        self.keyframes.clear();
        self.frames.clear();
        self.temp_units.clear();
        self.temp_keyframe = None;
        self.first_temp_unit_timestamp = None;
        log::info!("帧管理器已清空");
    }

    /// 检查是否有帧数据
    pub fn has_frames(&self) -> bool {
        !self.keyframes.is_empty()
    }
}
