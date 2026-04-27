use expto::rdmp::Unit;

/// 原始帧数据包（保留协议原始数据）
pub struct UnitPack {
    #[allow(dead_code)]
    last_keyframe: Option<usize>,
    #[allow(dead_code)]
    pack: Vec<Unit>,
}

impl UnitPack {
    pub fn new() -> Self {
        Self { last_keyframe: None, pack: Vec::new() }
    }

    pub fn new_frame(current_keyframe: Option<usize>, units: &[Unit]) -> Self {
        Self {
            last_keyframe: current_keyframe,
            pack: units.to_vec(),
        }
    }
}
