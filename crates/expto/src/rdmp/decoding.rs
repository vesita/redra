use prost::Message;
use log;

use crate::rdmp::{ExHeader, Unit};


/// 从数据中解码 ExHeader
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
    
    // me 字段直接表示 Header 的完整编码长度，无需额外计算
    let header_length = header.me as usize;
    let payload_data = &data[header_length..];
    
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

/// 解码一个 Unit 并返回剩余数据
pub fn decode_and_next(data: &[u8]) -> Result<(Unit, &[u8]), String> {
    if data.is_empty() {
        return Err("empty data".to_string());
    }
    
    // 解码头部
    let header = decode_header(data)?;
    
    // me 字段直接表示 Header 的完整编码长度
    let header_length = header.me as usize;
    
    // 检查是否有足够的 payload 数据
    if data.len() < header_length + header.next as usize {
        return Err(format!(
            "数据不足：需要 {} 字节，实际 {} 字节",
            header_length + header.next as usize,
            data.len()
        ));
    }
    
    let payload_data = &data[header_length..header_length + header.next as usize];

    // 解析消息内容
    let message = match Unit::decode(payload_data) {
        Ok(unit) => unit,
        Err(e) => {
            log::error!("消息体解码失败: {}", e);
            return Err("payload decode error".to_string());
        }
    };

    // 返回剩余数据（从当前消息结束之后开始）
    let remaining = &data[header_length + header.next as usize..];
    Ok((message, remaining))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rdmp::{ExObject, ex_object::UObject};
    use crate::rdmp::encoding::encode;

    #[test]
    fn test_decode_header_basic() {
        // 创建一个简单的 Unit
        let unit = Unit {
            stamp: None,
            command: None,
            objects: vec![ExObject {
                u_object: Some(UObject::Id(42)),
            }],
        };
        
        // 编码
        let encoded = encode(&unit).expect("编码失败");
        
        // 解码 header
        let header = decode_header(&encoded).expect("解码 header 失败");
        
        // 验证 header 字段
        assert!(header.me > 0, "me 应该大于 0");
        assert!(header.next > 0, "next 应该大于 0");
        
        // 验证 me 与编码后的实际长度一致
        let mut re_encoded = Vec::new();
        header.encode_length_delimited(&mut re_encoded).expect("重新编码失败");
        assert_eq!(header.me as usize, re_encoded.len(),
            "me 应该等于重新编码后的总长度（包括 varint 前缀）");
    }

    #[test]
    fn test_header_me_is_total_length() {
        // 验证 me 字段确实等于编码后的总长度（包括 varint 前缀）
        for next in [0, 100, 127, 128, 1000, 16383, 16384] {
            let temp_header = ExHeader { me: 1, next };
            let content_len = temp_header.encoded_len();
            let total_len = prost::length_delimiter_len(content_len) + content_len;
            
            // 构造 header，me 设置为完整长度
            let header = ExHeader { 
                me: total_len as u32, 
                next 
            };
            
            // 编码这个 header
            let mut encoded = Vec::new();
            header.encode_length_delimited(&mut encoded).expect("编码失败");
            
            // 验证 me 字段是否等于编码后的总字节数
            assert_eq!(header.me as usize, encoded.len(),
                "next={} 时，me 应该等于编码后的总长度", next);
        }
    }

    #[test]
    fn test_decode_empty_data() {
        // 测试空数据
        let result = decode(&[]);
        assert!(result.is_err(), "空数据应该解码失败");
        
        let result = decode_header(&[]);
        assert!(result.is_err(), "空数据应该无法解码 header");
        
        let result = decode_and_next(&[]);
        assert!(result.is_err(), "空数据应该解码失败");
    }

    #[test]
    fn test_decode_partial_header() {
        // 测试不完整的 header 数据
        let unit = Unit {
            stamp: None,
            command: None,
            objects: vec![ExObject {
                u_object: Some(UObject::Id(1)),
            }],
        };
        
        let encoded = encode(&unit).expect("编码失败");
        
        // 只取前几个字节（不完整）
        let partial = &encoded[..2];
        
        let result = decode_header(partial);
        assert!(result.is_err(), "不完整的 header 应该解码失败");
    }

    #[test]
    fn test_decode_complete_roundtrip() {
        // 测试完整的编解码往返
        let original_unit = Unit {
            stamp: None,
            command: None,
            objects: vec![
                ExObject { u_object: Some(UObject::Id(1)) },
                ExObject { u_object: Some(UObject::Id(2)) },
                ExObject { u_object: Some(UObject::Id(3)) },
            ],
        };
        
        // 编码
        let encoded = encode(&original_unit).expect("编码失败");
        
        // 解码
        let decoded_unit = decode(&encoded).expect("解码失败");
        
        // 验证
        assert_eq!(decoded_unit.objects.len(), original_unit.objects.len());
        for (orig, dec) in original_unit.objects.iter().zip(decoded_unit.objects.iter()) {
            assert_eq!(orig.u_object, dec.u_object);
        }
    }

    #[test]
    fn test_decode_and_next_single() {
        // 测试单个 Unit 的 decode_and_next
        let unit = Unit {
            stamp: None,
            command: None,
            objects: vec![ExObject {
                u_object: Some(UObject::Id(99)),
            }],
        };
        
        let encoded = encode(&unit).expect("编码失败");
        
        let (decoded, remaining) = decode_and_next(&encoded).expect("decode_and_next 失败");
        
        // 验证解码的数据
        assert_eq!(decoded.objects.len(), 1);
        match &decoded.objects[0].u_object {
            Some(UObject::Id(id)) => assert_eq!(*id, 99),
            _ => panic!("期望 ID 对象"),
        }
        
        // 验证剩余数据为空
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_decode_and_next_multiple() {
        // 测试多个连续 Unit 的解码
        let units = vec![
            Unit {
                stamp: None,
                command: None,
                objects: vec![ExObject { u_object: Some(UObject::Id(1)) }],
            },
            Unit {
                stamp: None,
                command: None,
                objects: vec![ExObject { u_object: Some(UObject::Id(2)) }],
            },
            Unit {
                stamp: None,
                command: None,
                objects: vec![ExObject { u_object: Some(UObject::Id(3)) }],
            },
        ];
        
        // 编码并拼接所有 Unit
        let mut combined = Vec::new();
        for unit in &units {
            let encoded = encode(unit).expect("编码失败");
            combined.extend_from_slice(&encoded);
        }
        
        // 依次解码
        let mut remaining = combined.as_slice();
        for (i, expected_id) in [1, 2, 3].iter().enumerate() {
            let (decoded, next_remaining) = decode_and_next(remaining)
                .expect(&format!("第 {} 个 Unit 解码失败", i + 1));
            
            match &decoded.objects[0].u_object {
                Some(UObject::Id(id)) => assert_eq!(*id, *expected_id),
                _ => panic!("第 {} 个 Unit 的对象类型错误", i + 1),
            }
            
            remaining = next_remaining;
        }
        
        // 验证所有数据都已消费
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_decode_insufficient_payload() {
        // 测试 payload 数据不足的情况
        let unit = Unit {
            stamp: None,
            command: None,
            objects: vec![ExObject {
                u_object: Some(UObject::Id(123)),
            }],
        };
        
        let encoded = encode(&unit).expect("编码失败");
        
        // 截断数据，使其不足以包含完整的 payload
        let truncated = &encoded[..encoded.len() - 5];
        
        let result = decode(truncated);
        assert!(result.is_err(), "数据不足应该解码失败");
    }

    #[test]
    fn test_decode_and_next_insufficient_data() {
        // 测试 decode_and_next 在数据不足时的行为
        let unit = Unit {
            stamp: None,
            command: None,
            objects: vec![ExObject {
                u_object: Some(UObject::Id(456)),
            }],
        };
        
        let encoded = encode(&unit).expect("编码失败");
        
        // 截断数据
        let truncated = &encoded[..encoded.len() / 2];
        
        let result = decode_and_next(truncated);
        assert!(result.is_err(), "数据不足应该解码失败");
    }

    #[test]
    fn test_header_varint_boundary_decoding() {
        // 测试 varint 边界值的 header 解码
        for next in [127, 128, 16383, 16384, 2097151, 2097152] {
            let temp_header = ExHeader { me: 1, next };
            let content_len = temp_header.encoded_len();
            let total_len = prost::length_delimiter_len(content_len) + content_len;
            
            let header = ExHeader { 
                me: total_len as u32, 
                next 
            };
            
            // 编码 header
            let mut encoded = Vec::new();
            header.encode_length_delimited(&mut encoded).expect("编码失败");
            
            // 解码 header
            let decoded = decode_header(&encoded).expect(&format!(
                "next={} 时解码失败", next
            ));
            
            assert_eq!(decoded.me, header.me);
            assert_eq!(decoded.next, header.next);
            
            // 验证 me 字段即为总长度
            assert_eq!(decoded.me as usize, encoded.len(),
                "next={} 时 me 值与编码长度不匹配", next);
        }
    }

    #[test]
    fn test_decode_with_complex_unit() {
        // 测试包含多种对象类型的复杂 Unit
        use crate::rdmp::{ExTransform};
        
        let unit = Unit {
            stamp: None,
            command: None,
            objects: vec![
                ExObject { u_object: Some(UObject::Id(100)) },
                ExObject { 
                    u_object: Some(UObject::Transform(ExTransform {
                        x: 1.0, y: 2.0, z: 3.0,
                        rx: 0.0, ry: 0.0, rz: 0.0,
                        sx: 1.0, sy: 1.0, sz: 1.0,
                    }))
                },
                ExObject { u_object: Some(UObject::Id(200)) },
            ],
        };
        
        let encoded = encode(&unit).expect("编码失败");
        let decoded = decode(&encoded).expect("解码失败");
        
        assert_eq!(decoded.objects.len(), 3);
        
        // 验证第一个对象
        match &decoded.objects[0].u_object {
            Some(UObject::Id(id)) => assert_eq!(*id, 100),
            _ => panic!("第一个对象应该是 ID"),
        }
        
        // 验证第二个对象
        match &decoded.objects[1].u_object {
            Some(UObject::Transform(t)) => {
                assert!((t.x - 1.0).abs() < f32::EPSILON);
                assert!((t.y - 2.0).abs() < f32::EPSILON);
                assert!((t.z - 3.0).abs() < f32::EPSILON);
            },
            _ => panic!("第二个对象应该是 Transform"),
        }
        
        // 验证第三个对象
        match &decoded.objects[2].u_object {
            Some(UObject::Id(id)) => assert_eq!(*id, 200),
            _ => panic!("第三个对象应该是 ID"),
        }
    }
}