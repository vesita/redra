use log::{debug, error};
use prost::Message;

use crate::proto::{command::Command, declare};

/// 解码单个Command消息
/// 
/// 将字节数组解码为Command协议对象
/// 
/// # 示例
/// ```
/// use redra_proto::coding::decoding::decode;
/// let data: Vec<u8> = vec![]; // 实际数据
/// if let Ok(command) = decode(&data) {
///     // 处理command
/// }
/// ```
pub fn decode(buffer: &[u8]) -> Result<Command, String> {
    match Command::decode(buffer) {
        Ok(pack) => {
            debug!("Command解码成功，ID: {}", pack.command_id);
            Ok(pack)
        },
        Err(e) => {
            error!("协议数据包解码失败: {}", e);
            Err("decode error".to_string())
        }
    }
}

/// 解码多个Command消息
/// 
/// 从一个字节数组中解码出多个Command对象，通常用于处理批量数据
/// 
/// # 示例
/// ```
/// use redra_proto::coding::decoding::decode_multiple;
/// let data: Vec<u8> = vec![]; // 实际数据
/// if let Ok(commands) = decode_multiple(&data) {
///     // 处理commands
/// }
/// ```
pub fn decode_multiple(buffer: &[u8]) -> Result<Vec<Command>, String> {
    let mut commands = Vec::new();
    let mut offset = 0;
    
    while offset < buffer.len() {
        // 尝试读取trailer以确定下一个command的位置
        if let Some((start, end)) = parse_trailer(buffer, offset) {
            if end <= buffer.len() {
                let command_data = &buffer[start..end];
                match decode(command_data) {
                    Ok(command) => {
                        commands.push(command);
                        offset = end; // 移动到下一个command的位置
                    },
                    Err(e) => {
                        error!("解码Command时出错: {}", e);
                        return Err(e);
                    }
                }
            } else {
                break; // 缓冲区不够，跳出循环
            }
        } else {
            break; // 无法读取trailer，跳出循环
        }
    }
    
    Ok(commands)
}

/// 读取协议预告信息(trailer)
/// 
/// 从指定位置开始读取协议预告信息，确定数据包的长度和位置
/// 符合TCMP (Trailer-Command Messaging Protocol) 规范
/// 
/// # 示例
/// ```
/// use redra_proto::coding::decoding::parse_trailer;
/// let data: Vec<u8> = vec![]; // 实际数据
/// if let Some((start, end)) = parse_trailer(&data, 0) {
///     // 处理数据范围 [start, end)
/// }
/// ```
pub fn parse_trailer(buffer: &[u8], base: usize) -> Option<(usize, usize)> {
    if buffer.len() < base + 4 {
        return None;
    }
    debug!("正在读取协议预告信息，起始位置: {}", base);
    let me = buffer[base + 1] as usize;
    let left = base + me;
    debug!("offset:{}", me);
    let temp: Vec<u8> = buffer[base..left].to_vec();
    if let Ok(trailer) = declare::Trailer::decode(&temp[..]) {
        let right = trailer.next as usize + left;
        debug!("协议预告信息解析成功，数据范围: [{}..{}]", left, right);
        return Some((left, right));
    }
    error!("协议预告信息解析失败，起始位置: {}", base);
    None
}

/// 检查Command是否为宏指令
/// 
/// # 示例
/// ```
/// use redra_proto::coding::decoding::is_macro;
/// let command = Command::default(); // 实际command
/// if is_macro(&command) {
///     // 处理宏指令
/// }
/// ```
pub fn is_macro(command: &Command) -> bool {
    matches!(
        &command.cmd_pack,
        Some(crate::proto::command::command::CmdPack::Macro(_))
    )
}

/// 处理宏指令
/// 
/// 根据宏指令类型执行不同的处理逻辑
/// 
/// # 示例
/// ```
/// use redra_proto::coding::decoding::handle_macro;
/// let command = Command::default(); // 实际command
/// if let Ok(result) = handle_macro(&command) {
///     println!("处理结果: {}", result);
/// }
/// ```
pub fn handle_macro(command: &Command) -> Result<String, String> {
    if !is_macro(command) {
        return Err("不是宏指令".to_string());
    }

    if let Some(crate::proto::command::command::CmdPack::Macro(macro_cmd)) = &command.cmd_pack {
        if let Some(macro_command) = &macro_cmd.macro_command {
            use crate::proto::command::macro_cmd::MacroCommand;
            
            match macro_command {
                MacroCommand::ConnectionControl(conn_ctrl) => {
                    debug!("处理连接控制命令: {} for session {}", conn_ctrl.action, conn_ctrl.session_id);
                    Ok(format!("连接控制: {} for session {}", conn_ctrl.action, conn_ctrl.session_id))
                },
                MacroCommand::Heartbeat(heartbeat) => {
                    debug!("处理心跳命令: {} for session {}", heartbeat.timestamp, heartbeat.session_id);
                    Ok(format!("心跳: {} for session {}", heartbeat.timestamp, heartbeat.session_id))
                },
                MacroCommand::Batch(batch_cmd) => {
                    debug!("处理批量命令，包含 {} 个子命令", batch_cmd.commands.len());
                    Ok(format!("批量命令: {} 个子命令", batch_cmd.commands.len()))
                },
                MacroCommand::Metadata(meta_cmd) => {
                    debug!("处理元数据命令: {}", meta_cmd.key);
                    Ok(format!("元数据: {}", meta_cmd.key))
                },
            }
        } else {
            Err("宏指令内容为空".to_string())
        }
    } else {
        Err("不是宏指令".to_string())
    }
}

// 为了向后兼容，保留旧的函数名
pub use decode as decode_command;
pub use decode_multiple as decode_multiple_commands;
pub use parse_trailer as read_trailer;
pub use is_macro as is_macro_command;
pub use handle_macro as handle_macro_command;