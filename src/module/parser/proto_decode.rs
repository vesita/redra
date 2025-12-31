use log::{debug, error};
use prost::Message;

use crate::proto::{command::Command, declare};

/// 解析单个Pack消息
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
    match Command::decode(buffer) {
        Ok(pack) => {
            Ok(pack)
        },
        Err(e) => {
            error!("协议数据包解码失败: {}", e);
            Err("decode error".to_string())
        }
    }
}


/// 自动解码函数
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
    // 这个函数现在只解析一个Pack消息，因为linker已经处理了trailer
    match decode_pack(buffer) {
        Ok(pack) => Ok(vec![pack]),
        Err(e) => Err(e),
    }
}

/// 读取协议预告信息(trailer)
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
    if buffer.len() < base + 4 {
        return None;
    }
    debug!("正在读取协议预告信息，起始位置: {}", base);
    let me = buffer[base + 1].clone() as usize;
    let left = base + me;
    debug!("offset:{}", me);
    let temp: Vec<u8> = buffer[base..left].to_vec().clone();
    if let Ok(trailer) = declare::Trailer::decode(&temp[..]) {
        let right = trailer.next as usize + left;
        debug!("协议预告信息解析成功，数据范围: [{}..{}]", left, right);
        return Some((left, right));
    }
    error!("协议预告信息解析失败，起始位置: {}", base);
    None
}