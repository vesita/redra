/// Parser crate - 协议解析核心逻辑库
/// 
/// 本 crate 提供纯粹的协议解析功能，不包含 Bevy 集成。
/// Bevy 集成应在主应用中实现。

pub mod parser;
pub mod assembler;
pub mod defaults;

// 导出公共 API
pub use parser::{ProtocolParser, ParsedCommand};
pub use assembler::FrameAssembler;
pub use defaults::DefaultMaterialConfig;