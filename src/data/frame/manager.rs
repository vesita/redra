use std::time::Instant;

use bevy::prelude::*;
use expto::rdmp::{CommandType, Unit};

use crate::data::frame::{KeyFrame, UnitPack};

/// 帧管理器 — 核心数据管理资源
#[derive(Resource)]
pub struct FrameManager {
    pub current_frame: usize,
    timestamp: u64,
    keyframes: Vec<KeyFrame>,
    frames: Vec<UnitPack>,
    temp_units: Vec<Unit>,
    temp_keyframe: Option<KeyFrame>,
    first_temp_unit_timestamp: Option<u64>,
    first_temp_unit_at: Option<Instant>,
}

impl Default for FrameManager {
    fn default() -> Self {
        Self {
            current_frame: 0,
            timestamp: 0,
            keyframes: Vec::new(),
            frames: Vec::new(),
            temp_units: Vec::new(),
            temp_keyframe: None,
            first_temp_unit_timestamp: None,
            first_temp_unit_at: None,
        }
    }
}

impl FrameManager {
    pub fn new() -> Self {
        Self::default()
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

                            self.temp_units.clear();
                            self.first_temp_unit_timestamp = None;
                            self.first_temp_unit_at = None;
                        }
                        _ => {
                            if self.temp_units.is_empty() {
                                self.first_temp_unit_timestamp = unit.stamp.as_ref().map(|s| s.timestamp);
                                self.first_temp_unit_at = Some(Instant::now());
                            }
                            self.temp_units.push(unit.clone());
                        }
                    }
                }
            }
            None => {
                if self.temp_units.is_empty() {
                    self.first_temp_unit_timestamp = unit.stamp.as_ref().map(|s| s.timestamp);
                    self.first_temp_unit_at = Some(Instant::now());
                }
                self.temp_units.push(unit.clone());
            }
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
        if self.temp_units.len() >= 100 {
            return true;
        }
        // 检查 protobuf 时间戳差（多 Unit 场景）
        if let Some(first_timestamp) = self.first_temp_unit_timestamp {
            if let Some(last_unit_stamp) = self.temp_units.last().and_then(|u| u.stamp.as_ref()) {
                let current_timestamp = last_unit_stamp.timestamp;
                if current_timestamp.saturating_sub(first_timestamp) >= 5000 {
                    return true;
                }
            }
        }
        // 检查实时时间差（单 Unit 含大量对象的场景，如点云）
        if let Some(first_time) = self.first_temp_unit_at {
            if first_time.elapsed().as_millis() >= 5000 {
                return true;
            }
        }
        false
    }

    pub fn current_frame(&self) -> &KeyFrame {
        self.keyframes.get(self.current_frame).unwrap()
    }

    // ==================== 数据访问接口 ====================

    pub fn current_frame_index(&self) -> usize {
        self.current_frame
    }

    pub fn get_current_keyframe(&self) -> Option<&KeyFrame> {
        self.keyframes.get(self.current_frame)
    }

    pub fn get_keyframe(&self, index: usize) -> Option<&KeyFrame> {
        self.keyframes.get(index)
    }

    pub fn total_frames(&self) -> usize {
        self.keyframes.len()
    }

    pub fn next_frame(&mut self) -> bool {
        if self.current_frame + 1 < self.keyframes.len() {
            self.current_frame += 1;
            true
        } else {
            false
        }
    }

    pub fn prev_frame(&mut self) -> bool {
        if self.current_frame > 0 {
            self.current_frame -= 1;
            true
        } else {
            false
        }
    }

    pub fn seek_to_frame(&mut self, frame_index: usize) -> bool {
        if frame_index < self.keyframes.len() {
            self.current_frame = frame_index;
            true
        } else {
            false
        }
    }

    pub fn get_all_keyframes(&self) -> &[KeyFrame] {
        &self.keyframes
    }

    pub fn clear(&mut self) {
        self.current_frame = 0;
        self.timestamp = 0;
        self.keyframes.clear();
        self.frames.clear();
        self.temp_units.clear();
        self.temp_keyframe = None;
        self.first_temp_unit_timestamp = None;
        self.first_temp_unit_at = None;
        log::info!("帧管理器已清空");
    }

    pub fn has_frames(&self) -> bool {
        !self.keyframes.is_empty()
    }

    pub fn delete_entities(&mut self, entity_ids: &[u64]) -> usize {
        let mut deleted_count = 0;

        for keyframe in &mut self.keyframes {
            for &entity_id in entity_ids {
                if keyframe.ids.contains_key(&entity_id) {
                    if let Some(idx) = keyframe.ids.get(&entity_id).copied() {
                        keyframe.packs.remove(idx);
                        keyframe.ids.remove(&entity_id);
                        for (_, idx_ref) in keyframe.ids.iter_mut() {
                            if *idx_ref > idx {
                                *idx_ref -= 1;
                            }
                        }
                        deleted_count += 1;
                    }
                }
            }
        }

        if deleted_count > 0 {
            log::info!("从帧数据中删除了 {} 个实体", deleted_count);
        }

        deleted_count
    }
}
