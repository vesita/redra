pub mod client;

// 导出常用类型和函数，方便外部使用
pub use client::{
    init_global_client, 
    next_set,
    send_point, 
    send_segment, 
    send_cube, 
    send_sphere,
    send_points,
    send_point_with_config,
    send_segment_with_config,
    send_cube_with_config,
    send_sphere_with_config,
    send_points_with_config,
    ShapeConfig,
};

// 导出 redra_proto 的核心类型和函数
pub use redra_proto::coding::decoding::{decode, parse_trailer, decode_command, decode_multiple as decode_multiple_commands};
pub use redra_proto::coding::encoding::{encode, encode_tcmp, encode as encode_command, encode_multiple_tcmp as encode_multiple_commands};

// 为了向后兼容保留旧的函数名
pub use redra_proto::coding::decoding::{decode as decode_pack, parse_trailer as read_trailer};
pub use redra_proto::coding::encoding::{encode as encode_pack};