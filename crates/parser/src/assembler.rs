use expto::rdmp::Unit;
use inpto::FrameData;

/// 帧组装器：将 Units 组装为 FrameData
pub struct FrameAssembler {
    seq_counter: u64,
}

impl FrameAssembler {
    pub fn new() -> Self {
        Self {
            seq_counter: 0,
        }
    }

    /// 将 Units 组装为 FrameData
    pub fn assemble_frame(&mut self, units: &[Unit], timestamp_ms: u64) -> FrameData {
        self.seq_counter += 1;
        
        let timestamp_ns = timestamp_ms * 1_000_000; // 毫秒转纳秒
        let mut frame_data = FrameData::new(self.seq_counter, timestamp_ns);
        frame_data.add_units(units.to_vec());
        
        frame_data
    }

    /// 重置序列号计数器
    pub fn reset(&mut self) {
        self.seq_counter = 0;
    }
}

impl Default for FrameAssembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_frame() {
        let mut assembler = FrameAssembler::new();
        let units = vec![];
        
        let frame = assembler.assemble_frame(&units, 1000);
        assert_eq!(frame.id, 1);
        assert_eq!(frame.timestamp, 1_000_000_000);
        
        let frame2 = assembler.assemble_frame(&units, 2000);
        assert_eq!(frame2.id, 2);
    }
}
