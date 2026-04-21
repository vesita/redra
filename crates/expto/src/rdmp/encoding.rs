use prost::Message;
use log;

use crate::rdmp::{ExHeader, Unit};

pub fn encode(message: &Unit) -> Result<Vec<u8>, String> {
    log::debug!("开始编码协议包");

    let unit_data = Unit::encode_to_vec(&message);
    let unit_len = unit_data.len() as u32;
    
    let temp_header = ExHeader {
        me: 1,  // 占位符
        next: unit_len,
    };
    let trailer_size = temp_header.encoded_len() as u32;
    
    let header = ExHeader {
        me: trailer_size,
        next: unit_len,
    };
    
    let mut buf = Vec::new();
    
    if let Err(e) = header.encode_length_delimited(&mut buf) {
        log::error!("协议头编码失败: {}", e);
        return Err(e.to_string());
    }
    buf.extend_from_slice(&unit_data);

    log::debug!("成功编码协议包，header: {:?}, payload size: {}", header, unit_data.len());
    Ok(buf)
}