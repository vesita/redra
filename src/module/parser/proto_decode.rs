use log::{debug, error};
use prost::Message;

use crate::proto::{command::Command, declare};

// 解析单个Pack消息
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


pub fn auto_decode(buffer: &[u8]) -> Result<Vec<Command>, String> { 
    // 这个函数现在只解析一个Pack消息，因为linker已经处理了trailer
    match decode_pack(buffer) {
        Ok(pack) => Ok(vec![pack]),
        Err(e) => Err(e),
    }
}

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