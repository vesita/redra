use expto::rdmp::Unit;

use crate::manager::data::frame::UnitPack;


impl UnitPack {
    pub fn new() -> Self {
        Self {
            last_keyframe: None,
            pack: Vec::new(),
        }
    }

    pub fn new_frame(current_keyframe: Option<usize>, units: &[Unit]) -> Self {
        Self {
            last_keyframe: current_keyframe,
            pack: units.to_vec(),
        }
    }
}
