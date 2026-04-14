use prost::Message;
use std::collections::HashMap;

use crate::proto::{command::Command, declare::Trailer};

/// 编码单个Command消息
/// 
/// 将Command对象编码为字节数组
/// 
/// # 示例
/// ```
/// use redra_proto::coding::encoding::encode;
/// let command = Command::default();
/// let data = encode(&command).unwrap();
/// ```
pub fn encode(command: &Command) -> Result<Vec<u8>, String> {
    // encode_to_vec is infallible in prost, but we wrap it in Ok to maintain API consistency
    Ok(Command::encode_to_vec(command))
}

/// 编码Command并添加Trailer（TCMP协议格式）
/// 
/// 根据TCMP (Trailer-Command Messaging Protocol) 规范，生成 "Trailer + Pack" 格式的完整数据包
/// 
/// # 示例
/// ```
/// use redra_proto::coding::encoding::encode_tcmp;
/// let command = Command::default();
/// let packet = encode_tcmp(&command).unwrap();
/// ```
pub fn encode_tcmp(command: &Command) -> Result<Vec<u8>, String> {
    let pack_data = encode(command)?;
    let pack_len = pack_data.len() as u32;
    
    // 计算trailer大小
    let temp_trailer = Trailer {
        me: 1,  // 占位符
        next: pack_len,
    };
    let trailer_size = temp_trailer.encoded_len() as u32;
    
    // 创建最终的Trailer
    let trailer = Trailer {
        me: trailer_size,
        next: pack_len,
    };
    
    let mut trailer_buf = Vec::new();
    trailer.encode(&mut trailer_buf)
        .map_err(|e| format!("Trailer编码失败: {}", e))?;
    
    // 组合为完整数据包: [Trailer][Pack]
    let mut packet = trailer_buf;
    packet.extend_from_slice(&pack_data);
    
    Ok(packet)
}

/// 批量编码多个命令并添加TCMP格式（推荐用于网络传输）
/// 
/// 每个命令都被编码为独立的 "Trailer + Pack" 数据包，符合TCMP协议
/// 
/// # 示例
/// ```
/// use redra_proto::coding::encoding::encode_multiple_tcmp;
/// let commands = vec![Command::default(), Command::default()];
/// let packet = encode_multiple_tcmp(&commands).unwrap();
/// ```
pub fn encode_multiple_tcmp(commands: &[Command]) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    
    for command in commands {
        let packet = encode_tcmp(command)?;
        result.extend_from_slice(&packet);
    }
    
    Ok(result)
}

/// 创建连接控制命令
/// 
/// 用于建立、维持或终止连接会话
/// 
/// # 示例
/// ```
/// use redra_proto::coding::encoding::connection_control;
/// let cmd = connection_control("connect", "session_123");
/// ```
pub fn connection_control(action: impl Into<String>, session_id: impl Into<String>) -> Command {
    use crate::proto::command::{Command as ProtoCommand, command::CmdPack, MacroCmd, ConnectionControl, macro_cmd::MacroCommand};
    
    let action_str = action.into();
    let session_id_str = session_id.into();

    let connection_control = ConnectionControl {
        action: action_str.clone(),
        session_id: session_id_str.clone(),
    };
    
    let macro_cmd = MacroCmd {
        macro_command: Some(MacroCommand::ConnectionControl(connection_control)),
    };
    
    ProtoCommand {
        cmd_pack: Some(CmdPack::Macro(macro_cmd)),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        command_id: format!("conn_ctrl_{}_{}", action_str, session_id_str),
    }
}

/// 创建心跳命令
/// 
/// 用于维持连接活跃状态
/// 
/// # 示例
/// ```
/// use redra_proto::coding::encoding::heartbeat;
/// let cmd = heartbeat("session_123");
/// ```
pub fn heartbeat(session_id: impl Into<String>) -> Command {
    use crate::proto::command::{Command as ProtoCommand, command::CmdPack, MacroCmd, Heartbeat, macro_cmd::MacroCommand};
    
    let session_id_str = session_id.into();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    let heartbeat = Heartbeat {
        timestamp,
        session_id: session_id_str.clone(),
    };
    
    let macro_cmd = MacroCmd {
        macro_command: Some(MacroCommand::Heartbeat(heartbeat)),
    };
    
    ProtoCommand {
        cmd_pack: Some(CmdPack::Macro(macro_cmd)),
        timestamp,
        command_id: format!("heartbeat_{}", session_id_str),
    }
}

/// 创建元数据命令
/// 
/// 用于传输配置、属性等元数据
/// 
/// # 示例
/// ```
/// use redra_proto::coding::encoding::metadata;
/// use std::collections::HashMap;
/// let mut props = HashMap::new();
/// props.insert("key".to_string(), "value".to_string());
/// let cmd = metadata("config", "value", props);
/// ```
pub fn metadata(
    key: impl Into<String>, 
    value: impl Into<String>, 
    properties: HashMap<String, String>
) -> Command {
    use crate::proto::command::{Command as ProtoCommand, command::CmdPack, MacroCmd, MetaCmd, macro_cmd::MacroCommand};
    
    let key_str = key.into();
    let value_str = value.into();

    let meta_cmd = MetaCmd {
        key: key_str.clone(),
        value: value_str,
        properties,
    };
    
    let macro_cmd = MacroCmd {
        macro_command: Some(MacroCommand::Metadata(meta_cmd)),
    };
    
    ProtoCommand {
        cmd_pack: Some(CmdPack::Macro(macro_cmd)),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        command_id: format!("meta_{}", key_str),
    }
}

/// 创建批量命令
///
/// 用于一次性发送多个命令
///
/// # 示例
/// ```
/// use redra_proto::coding::encoding::batch;
/// let commands = vec![Command::default(), Command::default()];
/// let cmd = batch(commands);
/// ```
pub fn batch(commands: Vec<Command>) -> Command {
    use crate::proto::command::{Command as ProtoCommand, command::CmdPack, MacroCmd, BatchCmd, macro_cmd::MacroCommand};
    
    let batch_cmd = BatchCmd {
        commands,
        batch_id: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32,  // 使用当前时间作为批次ID
    };
    
    let macro_cmd = MacroCmd {
        macro_command: Some(MacroCommand::Batch(batch_cmd)),
    };
    
    ProtoCommand {
        cmd_pack: Some(CmdPack::Macro(macro_cmd)),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        command_id: format!("batch_{}", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()  // 使用纳秒时间戳作为唯一ID
        ),
    }
}