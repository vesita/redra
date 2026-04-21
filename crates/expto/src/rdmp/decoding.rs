use prost::Message;
use log;
use serde::de;

use crate::rdmp::{ExHeader, Unit};

pub fn decode_header(data: &[u8]) -> Result<ExHeader, String> {
    let mut cursor = &data[..];
    match ExHeader::decode_length_delimited(&mut cursor) {
        Ok(h) => Ok(h),
        Err(e) => {
            log::error!("协议头解码失败: {}", e);
            Err("header decode error".to_string())
        }
    }
}

/// 解码一个完整的协议包，包括header和实际消息内容
pub fn decode(data: &[u8]) -> Result<Unit, String> {    
    // 解码头部
    let header = decode_header(data)?;
    
    // 获取header之后的数据部分
    let payload_data = &data[(header.me as usize + 1)..];
    
    // 检查数据长度是否符合header中描述的下一部分长度
    if payload_data.len() < header.next as usize {
        let msg = format!(
            "数据长度不足，header指示下一消息长度为{}，实际剩余{}", 
            header.next, 
            payload_data.len()
        );
        log::warn!("{}", msg);
        return Err(msg);
    }
    
    // 解析消息内容
    let message = match Unit::decode(payload_data) {
        Ok(unit) => unit,
        Err(e) => {
            log::error!("消息体解码失败: {}", e);
            return Err("payload decode error".to_string());
        }
    };

    log::debug!("成功解码协议包，header: {:?}, payload size: {}", header, payload_data.len());
    Ok(message)
}

pub fn decode_and_next(data: &mut [u8]) -> Result<(Unit, &[u8]), String> {
    let header = decode_header(data)?;
    let payload_data = &data[(header.me as usize + 1)..];

        // 解析消息内容
    let message = match Unit::decode(payload_data) {
        Ok(unit) => unit,
        Err(e) => {
            log::error!("消息体解码失败: {}", e);
            return Err("payload decode error".to_string());
        }
    };

    let consumed_len = header.me as usize + header.next as usize;
    if consumed_len <= data.len() {
        let (_, remaining) = data.split_at_mut(consumed_len);
        Ok((message, remaining))
    } else {
        Err("consumed length exceeds data length".to_string())
    }
}