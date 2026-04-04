use prost::Message;

use crate::proto::command::Command;

/// 编码单个Command消息
/// 
/// 该函数接收一个Command对象并将其编码为字节数组
/// 
/// # 参数
/// * `command` - 要编码的Command对象
/// 
/// # 返回值
/// * `Ok(Vec<u8>)` - 编码成功的字节数组
/// * `Err(String)` - 编码失败时返回错误信息
pub fn encode_command(command: &Command) -> Result<Vec<u8>, String> {
    let encoded = Command::encode_to_vec(command);
    Ok(encoded)
}

/// 批量编码多个命令
/// 
/// 该函数接收一组Command对象并将它们编码为字节数组
/// 
/// # 参数
/// * `commands` - 要编码的Command对象向量
/// 
/// # 返回值
/// * `Ok(Vec<u8>)` - 编码成功的字节数组
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