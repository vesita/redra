use log::{info, debug, warn};
use tokio::{io::AsyncWriteExt, net::TcpStream};

use redra_proto::proto::command::Command;
use redra_proto::coding::encoding::encode_command_with_trailer;

pub struct Sender {
    stream: TcpStream,
}

impl Sender {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        info!("正在连接到服务器: {}", addr);
        let stream = TcpStream::connect(addr).await?;
        info!("成功连接到服务器: {}", addr);
        Ok(Self::new(stream))
    }

    /// 发送Command（自动添加Trailer）
    /// 
    /// 根据项目规范，使用 "Trailer + Pack" 格式发送数据
    pub async fn send_command(&mut self, command: Command) -> Result<(), Box<dyn std::error::Error>> {
        debug!("开始发送命令");
        
        // 使用统一的编码函数生成完整数据包（Trailer + Pack）
        let packet = encode_command_with_trailer(&command)
            .map_err(|e| format!("编码失败: {}", e))?;
        
        // 发送完整数据包
        self.stream.write_all(&packet).await?;
        self.stream.flush().await?;
        
        debug!("命令已发送，数据包大小: {} 字节", packet.len());
        Ok(())
    }
    
    /// 发送原始数据（不添加Trailer，用于特殊场景）
    pub async fn send_raw_data(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let len = data.len();
        debug!("发送原始数据，长度为: {}", len);
        
        self.stream.write_all(data).await?;
        self.stream.flush().await?;
        
        info!("成功发送原始数据，长度: {}", len);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[tokio::test]
    async fn test_command_encoding() {
        // 测试命令编码逻辑（不涉及网络发送）
        use redra_proto::proto::{
            command::command::CmdPack,
            designation::{DesignCmd, Spawn, design_cmd::Cmd, spawn},
            shape::{self, ShapePack, shape_pack, Pose, Color},
            transform::{Translation, Scale},
        };

        let point = shape::Point {
            pos: Some(Translation { x: 1.0, y: 2.0, z: 3.0 }),
            size: 1.0,  // 添加缺失的 size 字段
        };
        
        let spawn = Spawn {
            id: None,
            name: "test_point".to_string(),
            tags: vec!["test".to_string()],
            initial_pose: Some(Pose {
                translation: Some(Translation { x: 1.0, y: 2.0, z: 3.0 }),
                rotation: None,
                scale: Some(Scale { sx: 1.0, sy: 1.0, sz: 1.0 }),
            }),
            data: Some(spawn::Data::ShapeData(ShapePack {
                name: "point_shape".to_string(),
                tags: vec!["geometry".to_string()],
                color: Some(Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }),
                data: Some(shape_pack::Data::Point(point)),
            })),
        };
        let design_cmd = DesignCmd {
            cmd: Some(Cmd::Spawn(spawn)),
        };
        let command = Command {
            timestamp: 0,  // 使用简单的默认值
            command_id: "test_cmd_001".to_string(),
            cmd_pack: Some(CmdPack::Designation(design_cmd)),
        };

        let encoded_data = command.encode_to_vec();
        
        // 验证编码后的数据不为空
        assert!(!encoded_data.is_empty());
        
        // 验证可以解码回原始数据
        let decoded = Command::decode(encoded_data.as_slice()).expect("应该能成功解码");
        assert!(decoded.cmd_pack.is_some());
    }

    #[tokio::test]
    async fn test_raw_data_length_prefix() {
        // 测试原始数据的长度前缀编码
        let test_data = vec![1u8, 2, 3, 4, 5];
        let expected_length = test_data.len() as u32;
        let mut length_bytes = Vec::new();
        length_bytes.extend_from_slice(&expected_length.to_le_bytes());
        
        assert_eq!(length_bytes.len(), 4);
        assert_eq!(u32::from_le_bytes([length_bytes[0], length_bytes[1], length_bytes[2], length_bytes[3]]), expected_length);
    }
}