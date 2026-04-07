pub mod client;

// 导出重要的公共API
pub use client::*;

// 导出 geometry 模块中的核心类型
pub use client::geometry::{
    // 形状配置构建器
    ShapeConfig,
};

// 导出redra_proto中的编码解码功能，使它们可以直接通过redra_client访问
pub use redra_proto::coding::decoding::{decode_command, auto_decode_commands, read_trailer};
pub use redra_proto::coding::encoding::{encode_command, encode_multiple_commands};

// 导出redra_proto中的protobuf定义，方便客户端直接使用
pub use redra_proto::proto;

// 重新导出常用的类型，方便用户直接使用
pub use redra_proto::proto::shape::Color;
pub use redra_proto::proto::target::TargetId;
