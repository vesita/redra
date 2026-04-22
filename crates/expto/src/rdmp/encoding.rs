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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{prelude::decode, rdmp::{ExObject, ExTransform, decoding::decode_and_next, ex_object::UObject}};

    #[test]
    fn test_encode_decode_roundtrip() {
        // 创建一个简单的 Unit
        let mut unit = Unit {
            stamp: None,
            command: None,
            objects: vec![],
        };
        
        // 添加一个 ID 对象
        unit.objects.push(ExObject {
            u_object: Some(UObject::Id(1)),
        });
        
        // 编码
        let encoded = encode(&unit).expect("编码失败");
        
        // 解码
        let decoded = decode(&encoded).expect("解码失败");
        
        // 验证
        assert_eq!(decoded.objects.len(), 1);
        match &decoded.objects[0].u_object {
            Some(UObject::Id(id)) => assert_eq!(*id, 1),
            _ => panic!("期望ID对象"),
        }
    }

    #[test]
    fn test_encode_decode_with_transform() {
        // 创建一个包含 Transform 的 Unit
        let mut unit = Unit {
            stamp: None,
            command: None,
            objects: vec![],
        };
        
        unit.objects.push(ExObject {
            u_object: Some(UObject::Id(42)),
        });
        
        unit.objects.push(ExObject {
            u_object: Some(UObject::Transform(ExTransform {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                rx: 0.0,
                ry: 0.0,
                rz: 0.0,
                sx: 1.0,
                sy: 1.0,
                sz: 1.0,
            })),
        });
        
        // 编码
        let encoded = encode(&unit).expect("编码失败");
        
        // 解码
        let decoded = decode(&encoded).expect("解码失败");
        
        // 验证
        assert_eq!(decoded.objects.len(), 2);
        
        match &decoded.objects[0].u_object {
            Some(UObject::Id(id)) => assert_eq!(*id, 42),
            _ => panic!("第一个对象应该是ID"),
        }
        
        match &decoded.objects[1].u_object {
            Some(UObject::Transform(t)) => {
                assert!((t.x - 1.0).abs() < f32::EPSILON);
                assert!((t.y - 2.0).abs() < f32::EPSILON);
                assert!((t.z - 3.0).abs() < f32::EPSILON);
            },
            _ => panic!("第二个对象应该是Transform"),
        }
    }

    #[test]
    fn test_decode_partial_data() {
        // 测试不完整数据的处理
        let mut unit = Unit {
            stamp: None,
            command: None,
            objects: vec![],
        };
        
        unit.objects.push(ExObject {
            u_object: Some(UObject::Id(99)),
        });
        
        let encoded = encode(&unit).expect("编码失败");
        
        // 只取一半数据
        let partial = &encoded[..encoded.len() / 2];
        
        // 应该失败
        let result = decode(partial);
        assert!(result.is_err(), "不完整数据应该解码失败");
    }

    #[test]
    fn test_decode_and_next_single_unit() {
        // 测试 decode_and_next 函数
        let unit = Unit {
            stamp: None,
            command: None,
            objects: vec![ExObject {
                u_object: Some(UObject::Id(123)),
            }],
        };
        
        let encoded = encode(&unit).expect("编码失败");
        
        // 解码
        let (_decoded_unit, remaining) = decode_and_next(&encoded).expect("decode_and_next 失败");
        
        // 验证剩余数据为空
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_decode_and_next_multiple_units() {
        // 测试多个 Unit 连续解码
        let unit1 = Unit {
            stamp: None,
            command: None,
            objects: vec![ExObject {
                u_object: Some(UObject::Id(1)),
            }],
        };
        
        let unit2 = Unit {
            stamp: None,
            command: None,
            objects: vec![ExObject {
                u_object: Some(UObject::Id(2)),
            }],
        };
        
        let encoded1 = encode(&unit1).expect("编码unit1失败");
        let encoded2 = encode(&unit2).expect("编码unit2失败");
        
        // 拼接两个编码后的数据
        let mut combined = encoded1.clone();
        combined.extend_from_slice(&encoded2);
        
        // 解码第一个 Unit
        let (decoded1, remaining1) = decode_and_next(&combined).expect("解码第一个unit失败");
        match &decoded1.objects[0].u_object {
            Some(UObject::Id(id)) => assert_eq!(*id, 1),
            _ => panic!("第一个unit的ID错误"),
        }
        
        // 验证剩余数据应该是第二个 unit
        assert_eq!(remaining1.len(), encoded2.len(), "剩余字节应该等于第二个unit的长度");
        
        // 解码第二个 Unit
        let (decoded2, remaining2) = decode_and_next(remaining1).expect("解码第二个unit失败");
        match &decoded2.objects[0].u_object {
            Some(UObject::Id(id)) => assert_eq!(*id, 2),
            _ => panic!("第二个unit的ID错误"),
        }
        
        assert_eq!(remaining2.len(), 0);
    }

    #[test]
    fn test_header_format() {
        // 测试 header 的编码格式
        let unit = Unit {
            stamp: None,
            command: None,
            objects: vec![ExObject {
                u_object: Some(UObject::Id(1)),
            }],
        };
        
        let unit_data = Unit::encode_to_vec(&unit);
        let unit_len = unit_data.len() as u32;
        
        let temp_header = ExHeader {
            me: 1,
            next: unit_len,
        };
        let trailer_size = temp_header.encoded_len() as u32;
        
        let header = ExHeader {
            me: trailer_size,
            next: unit_len,
        };
        
        let mut buf = Vec::new();
        header.encode_length_delimited(&mut buf).expect("header编码失败");
        
        // 验证第一个字节是 varint 编码的 header 长度
        assert_eq!(buf[0] as u32, trailer_size, "第一个字节应该是header长度的varint编码");
    }

}