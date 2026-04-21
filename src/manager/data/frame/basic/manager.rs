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
                            self.add_frame(UnitPack::new_frame(self.last_keyframe_idx(), &self.temp_units));
                        },
                        _ => {
                            self.temp_units.push(unit.clone());
                        }
                    }
                }
            },
            None => {
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
        self.temp_units.len() >= 50 ||
        self.temp_keyframe.is_none()
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
}

