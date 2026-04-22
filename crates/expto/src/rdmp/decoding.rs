use prost::Message;
use log;

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
    // encode_length_delimited 格式: varint(header长度) + header内容 + payload
    // header.me 是 header 内容的长度（不包括 varint）
    // 需要找到 varint 的实际长度来计算 payload 的起始位置
    
    // 简单方法：重新编码 header 来获取实际占用的字节数
    let mut header_buf = Vec::new();
    if let Err(e) = header.encode_length_delimited(&mut header_buf) {
        return Err(format!("header 重新编码失败: {}", e));
    }
    let header_total_len = header_buf.len();
    
    let payload_data = &data[header_total_len..];
    
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

pub fn decode_and_next(data: &[u8]) -> Result<(Unit, &[u8]), String> {
    if data.is_empty() {
        return Err("empty data".to_string());
    }
    
    let header = decode_header(data)?;
    
    // 重新编码 header 来获取实际占用的字节数（包括 varint）
    let mut header_buf = Vec::new();
    if let Err(e) = header.encode_length_delimited(&mut header_buf) {
        return Err(format!("header 重新编码失败: {}", e));
    }
    let header_total_len = header_buf.len();
    
    // 检查是否有足够的 payload 数据
    if data.len() < header_total_len + header.next as usize {
        return Err(format!(
            "数据不足：需要 {} 字节，实际 {} 字节",
            header_total_len + header.next as usize,
            data.len()
        ));
    }
    
    let payload_data = &data[header_total_len..header_total_len + header.next as usize];

    // 解析消息内容
    let message = match Unit::decode(payload_data) {
        Ok(unit) => unit,
        Err(e) => {
            log::error!("消息体解码失败: {}", e);
            return Err("payload decode error".to_string());
        }
    };

    // 返回剩余数据（从当前消息结束之后开始）
    let remaining = &data[header_total_len + header.next as usize..];
    Ok((message, remaining))
}
