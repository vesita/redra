use log::{error, info};
use nalgebra::base;
use prost::Message;

use crate::proto::{declare, rd::{self, Pack}};

pub fn auto_decode(buffer: &[u8]) -> Result<Vec<Pack>, String> { 
    let mut base = 0;
    let mut result = vec![];
    while let Some((left, right)) = read_trailer(&buffer, base) {
        if let Ok(pack) = rd::Pack::decode(&buffer[left..right]) {
            info!("协议数据包解码成功，类型: {:?}", pack.data_type);
            result.push(pack);
            base = right;
        } else {
            error!("协议数据包解码失败，位置: {} 到 {}", left, right);
            return Err("decode error".to_string());
        }
    }
    Ok(result)
}

pub fn read_trailer(buffer: &[u8], base: usize) -> Option<(usize, usize)> {
    if buffer.len() < base + 4 {
        return None;
    }
    info!("正在读取协议预告信息，起始位置: {}", base);
    let me = buffer[base + 1].clone() as usize;
    let left = base + me;
    info!("offset:{}", me);
    let temp: Vec<u8> = buffer[base..left].to_vec().clone();
    if let Ok(trailer) = declare::Trailer::decode(&temp[..]) {
        let right = trailer.next as usize + left;
        info!("协议预告信息解析成功，数据范围: [{}..{}]", left, right);
        return Some((left, right));
    }
    error!("协议预告信息解析失败，起始位置: {}", base);
    None
}