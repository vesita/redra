pub mod proto;
pub mod coding;

// 重新导出关键功能，便于外部使用
pub use coding::decoding::{decode_command, auto_decode_commands, read_trailer};
pub use coding::encoding::{encode_command, encode_multiple_commands};

// 为了向后兼容，保留旧的函数名
pub use self::coding::decoding::{decode_command as decode_pack, auto_decode_commands as auto_decode};
pub use self::coding::encoding::{encode_command as encode_pack, encode_multiple_commands as encode_multiple};