pub mod proto;
pub mod coding;

// 导出优化后的API
pub use coding::decoding::{
    decode,
    decode_multiple,
    parse_trailer,
    is_macro,
    handle_macro,
    // 为了向后兼容保留旧的函数名
    decode_command,
    decode_multiple_commands,
    read_trailer,
    is_macro_command,
    handle_macro_command,
};
pub use coding::encoding::{
    encode,
    encode_tcmp,
    encode_multiple_tcmp,
    connection_control,
    heartbeat,
    metadata,
    batch,
};

// 为了向后兼容，保留旧的函数名
pub use self::coding::decoding::{decode as decode_pack, decode_multiple as auto_decode};
pub use self::coding::encoding::{encode as encode_pack, encode_multiple_tcmp as encode_multiple};
// 保留旧的函数名，映射到新的函数
pub use self::coding::encoding::{
    encode as encode_command,
    encode_multiple_tcmp as encode_multiple_commands,
    encode_tcmp as encode_command_with_trailer,
    encode_multiple_tcmp as encode_multiple_commands_with_trailer,
    connection_control as create_connection_control_command,
    heartbeat as create_heartbeat_command,
    metadata as create_metadata_command,
};