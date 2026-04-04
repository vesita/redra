pub mod client;

// 导出重要的公共API
pub use client::*;

// 导出redra_proto中的编码解码功能，使它们可以直接通过redra_client访问
pub use redra_proto::coding::decoding::{decode_command, auto_decode_commands, read_trailer};
pub use redra_proto::coding::encoding::{encode_command, encode_multiple_commands};

// 为了向后兼容，保留旧的函数名
pub use redra_proto::coding::decoding::{decode_command as decode_pack, auto_decode_commands as auto_decode};
pub use redra_proto::coding::encoding::{encode_command as encode_pack, encode_multiple_commands as encode_multiple};

// 导出redra_proto中的protobuf定义，方便客户端直接使用
pub use redra_proto::proto;