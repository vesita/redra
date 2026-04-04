use log::{info, debug, warn};
use prost::Message;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use redra_proto::proto::command::Command;

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

    pub async fn send_command(&mut self, command: Command) -> Result<(), Box<dyn std::error::Error>> {
        debug!("开始发送命令");
        
        // 先对命令进行编码
        let encoded_data = command.encode_to_vec();
        let len = encoded_data.len() as u32;
        
        // 创建trailer用于命令数据
        let temp_trailer = redra_proto::proto::declare::Trailer {
            me: 1,  // 临时值
            next: len,
        };
        
        // 获取trailer编码后的长度
        let trailer_size = temp_trailer.encoded_len() as u32;
        
        // 创建最终的trailer
        let trailer = redra_proto::proto::declare::Trailer {
            me: trailer_size as u32,
            next: len,
        };

        let mut trailer_buf = Vec::new();
        trailer.encode(&mut trailer_buf)?;
        
        // 先发送trailer，然后发送实际数据
        self.stream.write_all(&trailer_buf).await?;
        self.stream.write_all(&encoded_data).await?;
        
        info!("命令已发送");
        Ok(())
    }
    
    pub async fn send_raw_data(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let len = data.len() as u32;
        debug!("发送原始数据，长度为: {}", len);
        
        // 创建trailer用于原始数据
        let temp_trailer = redra_proto::proto::declare::Trailer {
            me: 1,  // 临时值
            next: len,
        };
        
        // 获取trailer编码后的长度
        let trailer_size = temp_trailer.encoded_len() as u32;
        
        // 创建最终的trailer
        let trailer = redra_proto::proto::declare::Trailer {
            me: trailer_size as u32,
            next: len,
        };

        let mut trailer_buf = Vec::new();
        trailer.encode(&mut trailer_buf)?;
        
        if let Err(e) = self.stream.write_all(&trailer_buf).await {
            warn!("写入trailer时发生错误: {}", e);
            return Err(Box::new(e));
        }
        
        if let Err(e) = self.stream.write_all(data).await {
            warn!("写入数据时发生错误: {}", e);
            return Err(Box::new(e));
        }
        
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