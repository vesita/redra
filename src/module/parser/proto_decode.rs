use redra_proto::proto::command::Command;
use redra_proto::coding::decoding::{decode_command as proto_decode_command, auto_decode_commands, read_trailer as proto_read_trailer};

/// 解析单个Pack消息 (为了向后兼容保留)
/// 
/// 该函数接收一个字节数组并尝试将其解码为Command协议对象
/// 
/// # 参数
/// * `buffer` - 包含协议数据的字节数组切片
/// 
/// # 返回值
/// * `Ok(Command)` - 解码成功的Command对象
/// * `Err(String)` - 解码失败时返回错误信息
pub fn decode_pack(buffer: &[u8]) -> Result<Command, String> {
    proto_decode_command(buffer)
}


/// 自动解码函数 (为了向后兼容保留)
/// 
/// 该函数调用decode_pack来解析协议数据，目前只解析一个Pack消息
/// 因为linker组件已经处理了trailer部分
/// 
/// # 参数
/// * `buffer` - 包含协议数据的字节数组切片
/// 
/// # 返回值
/// * `Ok(Vec<Command>)` - 解码成功的Command对象向量
/// * `Err(String)` - 解码失败时返回错误信息
pub fn auto_decode(buffer: &[u8]) -> Result<Vec<Command>, String> { 
    auto_decode_commands(buffer)
}

/// 读取协议预告信息(trailer) (为了向后兼容保留)
/// 
/// 该函数从指定位置开始读取协议预告信息，确定数据包的长度和位置
/// 
/// # 参数
/// * `buffer` - 包含协议数据的字节数组切片
/// * `base` - 开始读取的位置索引
/// 
/// # 返回值
/// * `Some((usize, usize))` - 解析成功时返回数据范围 (起始位置, 结束位置)
/// * `None` - 解析失败时返回None
pub fn read_trailer(buffer: &[u8], base: usize) -> Option<(usize, usize)> {
    proto_read_trailer(buffer, base)
}

// 新的更明确的函数名版本
pub fn decode_command_new(buffer: &[u8]) -> Result<Command, String> {
    proto_decode_command(buffer)
}

pub fn auto_decode_command_new(buffer: &[u8]) -> Result<Vec<Command>, String> {
    auto_decode_commands(buffer)
}