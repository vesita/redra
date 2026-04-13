use prost::Message;

use crate::proto::{command::Command, declare::Trailer};

/// 编码单个Command消息（不包含Trailer）
/// 
/// 该函数接收一个Command对象并将其编码为字节数组
/// 
/// # 参数
/// * `command` - 要编码的Command对象
/// 
/// # 返回值
/// * `Ok(Vec<u8>)` - 编码成功的字节数组（仅Pack部分）
/// * `Err(String)` - 编码失败时返回错误信息
pub fn encode_command(command: &Command) -> Result<Vec<u8>, String> {
    let encoded = Command::encode_to_vec(command);
    Ok(encoded)
}

/// 编码Command并添加Trailer（完整的数据包）
/// 
/// 根据项目规范，生成 "Trailer + Pack" 格式的完整数据包
/// 
/// # 参数
/// * `command` - 要编码的Command对象
/// 
/// # 返回值
/// * `Ok(Vec<u8>)` - 包含Trailer和Pack的完整数据包
/// * `Err(String)` - 编码失败时返回错误信息
/// 
/// # 示例
/// ```no_run
/// use redra_proto::coding::encoding::encode_command_with_trailer;
/// use redra_proto::proto::command::Command;
/// 
/// let command = Command::default();
/// let packet = encode_command_with_trailer(&command).unwrap();
/// // packet 格式: [Trailer][Pack]
/// ```
pub fn encode_command_with_trailer(command: &Command) -> Result<Vec<u8>, String> {
    // 1. 编码Command（Pack部分）
    let pack_data = Command::encode_to_vec(command);
    let pack_len = pack_data.len() as u32;
    
    // 2. 创建Trailer
    // 先创建一个临时Trailer来计算其大小
    let temp_trailer = Trailer {
        me: 1,  // 占位符
        next: pack_len,
    };
    let trailer_size = temp_trailer.encoded_len() as u32;
    
    // 3. 创建最终的Trailer
    let trailer = Trailer {
        me: trailer_size,
        next: pack_len,
    };
    
    // 4. 编码Trailer
    let mut trailer_buf = Vec::new();
    trailer.encode(&mut trailer_buf)
        .map_err(|e| format!("Trailer编码失败: {}", e))?;
    
    // 5. 组合成完整数据包: [Trailer][Pack]
    let mut packet = trailer_buf;
    packet.extend_from_slice(&pack_data);
    
    Ok(packet)
}

/// 批量编码多个命令（每个命令都带Trailer）
/// 
/// 该函数接收一组Command对象，将每个命令编码为独立的 "Trailer + Pack" 数据包
/// 
/// # 参数
/// * `commands` - 要编码的Command对象向量
/// 
/// # 返回值
/// * `Ok(Vec<u8>)` - 所有数据包的串联结果
/// * `Err(String)` - 编码失败时返回错误信息
pub fn encode_multiple_commands(commands: &[Command]) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    
    for command in commands {
        match encode_command(command) {
            Ok(mut encoded) => {
                result.append(&mut encoded);
            },
            Err(e) => {
                return Err(format!("批量编码失败，遇到错误: {}", e));
            }
        }
    }
    
    Ok(result)
}

/// 批量编码多个命令并添加Trailer（推荐用于网络传输）
/// 
/// 每个命令都被编码为独立的 "Trailer + Pack" 数据包
/// 
/// # 参数
/// * `commands` - 要编码的Command对象向量
/// 
/// # 返回值
/// * `Ok(Vec<u8>)` - 所有完整数据包的串联结果
/// * `Err(String)` - 编码失败时返回错误信息
pub fn encode_multiple_commands_with_trailer(commands: &[Command]) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    
    for command in commands {
        match encode_command_with_trailer(command) {
            Ok(mut packet) => {
                result.append(&mut packet);
            },
            Err(e) => {
                return Err(format!("批量编码失败，遇到错误: {}", e));
            }
        }
    }
    
    Ok(result)
}
