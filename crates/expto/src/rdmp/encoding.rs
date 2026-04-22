use prost::Message;
use log;

use crate::rdmp::{ExHeader, Unit};

/// 编码一个完整的 RDMP 协议包
/// 
/// ## RDMP 包的数据序列结构
/// 
/// 一个完整的 RDMP 包由两部分组成：Header + Payload
/// 
/// ```text
/// ┌──────────────────────────────────────────────────────────────┐
/// │                     RDMP Package                             │
/// ├─────────────────────────┬────────────────────────────────────┤
/// │      Header             │       Payload (Unit)               │
/// │  (ExHeader + Varint前缀) │    (Protobuf 编码的 Unit 数据)      │
/// └─────────────────────────┴────────────────────────────────────┘
/// ```
/// 
/// ### Header 详细结构
/// 
/// Header 使用 Protobuf 的 length-delimited 编码方式：
/// 
/// ```text
/// ┌──────────────┬──────────────────────────────────────────┐
/// │ Varint 前缀   │         ExHeader 内容                     │
/// │ (1-5 字节)    │  ┌──────────┬────────────────────────┐  │
/// │              │  │ me       │ next                   │  │
/// │              │  │(varint)  │ (varint)               │  │
/// │              │  └──────────┴────────────────────────┘  │
/// └──────────────┴──────────────────────────────────────────┘
/// ```
/// 
/// - **Varint 前缀**: 表示后面 ExHeader 内容的字节数
/// - **me 字段**: 存储 Header 的**完整编码长度**（包括 Varint 前缀 + ExHeader 内容）
/// - **next 字段**: 表示后面 Payload 的字节数
/// 
/// ### 示例
/// 
/// 假设 Payload 长度为 100 字节，ExHeader 内容编码后为 3 字节：
/// 
/// ```text
/// 位置:  0    1    2    3    4    5    6    ...  103
///       ┌────┬────┬────┬────┬────┬────┬────┬─────────┐
///       │ 03 │ 08 │ 03 │ 10 │ 64 │ UU │ UU │ ... UU  │
///       └────┴────┴────┴────┴────┴────┴────┴─────────┘
///        ↑         ↑              ↑
///        │         │              └─ Payload (100字节, header.next=100)
///        │         └─ ExHeader 内容 (3字节)
///        │            - 08 03: me=3 (tag=1, value=3)
///        │            - 10 64: next=100 (tag=2, value=100)
///        └─ Varint 前缀 (1字节): 表示后面有3字节
/// 
/// 总长度 = 1 (varint前缀) + 3 (ExHeader内容) + 100 (Payload) = 104 字节
/// header.me = 4 (完整Header长度 = 1 + 3)
/// header.next = 100 (Payload长度)
/// ```
/// 
/// ## 参数
/// * `message` - 要编码的 Unit 消息
/// 
/// ## 返回
/// * `Ok(Vec<u8>)` - 编码后的完整 RDMP 包
/// * `Err(String)` - 编码失败时的错误信息
pub fn encode(message: &Unit) -> Result<Vec<u8>, String> {
    log::debug!("开始编码协议包");

    // 1. 编码 Payload (Unit)
    let unit_data = Unit::encode_to_vec(&message);
    let unit_len = unit_data.len() as u32;
    
    // 2. 构建 Header
    let header = encode_header(unit_len);
    
    // 3. 组装完整的 RDMP 包
    let mut buf = Vec::new();
    
    // 3.1 编码 Header（使用 length-delimited 格式：varint前缀 + ExHeader内容）
    if let Err(e) = header.encode_length_delimited(&mut buf) {
        log::error!("协议头编码失败: {}", e);
        return Err(e.to_string());
    }
    
    // 3.2 追加 Payload
    buf.extend_from_slice(&unit_data);

    log::debug!("成功编码协议包，header: {:?}, payload size: {}", header, unit_data.len());
    Ok(buf)
}

/// 构建 ExHeader，计算并设置正确的 me 字段值
/// 
/// ## me 字段的计算逻辑
/// 
/// me 字段需要存储 Header 的完整编码长度，但由于 varint 编码的特性，
/// me 字段本身的值会影响其编码后的字节数，因此需要迭代计算：
/// 
/// 1. 先用占位符创建临时 header
/// 2. 计算 ExHeader 内容的编码长度
/// 3. 计算完整长度 = varint前缀长度 + ExHeader内容长度
/// 4. 用真实长度创建最终 header
/// 
/// 通常只需一次迭代即可收敛，因为大多数情况下占位符和真实值的 varint 编码长度相同。
/// 
/// ## 参数
/// * `next` - Payload 的长度（字节数）
/// 
/// ## 返回
/// * `ExHeader` - 构建好的协议头，其中 me 字段已设置为完整的编码长度
pub fn encode_header(next: u32) -> ExHeader {
    // 第一步：创建临时 header，me 使用占位符
    let temp_header = ExHeader {
        me: 1,  // 占位符，用于计算 ExHeader 内容的编码长度
        next,
    };
    
    // 第二步：计算 ExHeader 内容的编码长度（不包括 varint 前缀）
    let content_len = temp_header.encoded_len();
    
    // 第三步：计算完整的 Header 编码长度（包括 varint 前缀）
    let total_len = prost::length_delimiter_len(content_len) + content_len;
    
    // 第四步：创建最终的 header，me 设置为完整长度
    let header = ExHeader {
        me: total_len as u32,
        next,
    };
    header
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{prelude::decode, rdmp::{ExObject, ExTransform, decoding::decode_and_next, ex_object::UObject}};

    #[test]
    fn test_header_max() {
        let header = encode_header(u32::MAX);
        // me 应该等于 header 的完整编码长度（包括 length-delimited 前缀）
        let content_len = header.encoded_len();
        let total_len = prost::length_delimiter_len(content_len) + content_len;
        assert_eq!(header.me as usize, total_len);
    }

    #[test]
    fn test_encode_header_min() {
        let header = encode_header(0);
        // me 应该等于 header 的完整编码长度（包括 length-delimited 前缀）
        let content_len = header.encoded_len();
        let total_len = prost::length_delimiter_len(content_len) + content_len;
        assert_eq!(header.me as usize, total_len);
    }

    #[test]
    fn test_header_varint_boundaries() {
        // 测试 varint 从 1 字节变 2 字节的边界 (127 -> 128)
        let header_127 = encode_header(127);
        let content_len_127 = header_127.encoded_len();
        let total_len_127 = prost::length_delimiter_len(content_len_127) + content_len_127;
        assert_eq!(header_127.me as usize, total_len_127);
        
        let header_128 = encode_header(128);
        let content_len_128 = header_128.encoded_len();
        let total_len_128 = prost::length_delimiter_len(content_len_128) + content_len_128;
        assert_eq!(header_128.me as usize, total_len_128);
        
        // 测试 varint 从 2 字节变 3 字节的边界 (16383 -> 16384)
        let header_16383 = encode_header(16383);
        let content_len_16383 = header_16383.encoded_len();
        let total_len_16383 = prost::length_delimiter_len(content_len_16383) + content_len_16383;
        assert_eq!(header_16383.me as usize, total_len_16383);
        
        let header_16384 = encode_header(16384);
        let content_len_16384 = header_16384.encoded_len();
        let total_len_16384 = prost::length_delimiter_len(content_len_16384) + content_len_16384;
        assert_eq!(header_16384.me as usize, total_len_16384);
        
        // 测试 varint 从 3 字节变 4 字节的边界 (2097151 -> 2097152)
        let header_2097151 = encode_header(2097151);
        let content_len_2097151 = header_2097151.encoded_len();
        let total_len_2097151 = prost::length_delimiter_len(content_len_2097151) + content_len_2097151;
        assert_eq!(header_2097151.me as usize, total_len_2097151);
        
        let header_2097152 = encode_header(2097152);
        let content_len_2097152 = header_2097152.encoded_len();
        let total_len_2097152 = prost::length_delimiter_len(content_len_2097152) + content_len_2097152;
        assert_eq!(header_2097152.me as usize, total_len_2097152);
    }

    #[test]
    fn test_header_encode_decode_roundtrip() {
        use crate::rdmp::decoding::decode_header;
        
        // 测试关键边界值和典型值
        for next in [0, 100, 127, 128, 1000, 16383, 16384, 100000, u32::MAX] {
            let header = encode_header(next);
            
            // 使用 encode_length_delimited 编码
            let mut encoded = Vec::new();
            header.encode_length_delimited(&mut encoded).expect("编码失败");
            
            // 验证 me 字段等于完整的编码长度（包含 length-delimited 前缀）
            let content_len = header.encoded_len();
            let expected_total_len = prost::length_delimiter_len(content_len) + content_len;
            assert_eq!(header.me as usize, expected_total_len, 
                "next={} 时，me字段({})应等于完整编码长度({})", next, header.me, expected_total_len);
            
            // 验证整个编码数据的长度与 me 字段一致
            assert_eq!(encoded.len(), header.me as usize,
                "next={} 时，编码后的总长度应与 me 字段一致", next);
            
            // 解码验证
            let decoded = decode_header(&encoded).expect("解码失败");
            assert_eq!(decoded.me, header.me, "next={} 时，me字段不匹配", next);
            assert_eq!(decoded.next, header.next, "next={} 时，next字段不匹配", next);
        }
    }

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