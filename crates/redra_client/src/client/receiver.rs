use log::{info, warn, error};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use redra_proto::proto::command::Command;
use redra_proto::coding::decoding::decode_command;

pub struct Receiver {
    stream: TcpStream,
}

impl Receiver {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    /// 接收单个命令
    pub async fn receive_command(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
        // 读取数据包长度（这里假设使用长度前缀协议）
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let data_len = u32::from_le_bytes(len_buf) as usize;

        // 限制最大数据包大小以防止内存耗尽攻击
        if data_len > 10 * 1024 * 1024 {  // 10MB limit
            return Err("数据包过大".into());
        }

        // 读取命令数据
        let mut buf = vec![0u8; data_len];
        self.stream.read_exact(&mut buf).await?;

        // 解码命令
        match decode_command(&buf) {
            Ok(command) => {
                info!("接收到命令: {:?}", command.command_id);
                Ok(command)
            },
            Err(e) => {
                error!("解码命令失败: {}", e);
                Err(e.into())
            }
        }
    }

    /// 持续监听命令
    pub async fn listen_for_commands<F>(&mut self, mut handler: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(Command),
    {
        loop {
            match self.receive_command().await {
                Ok(command) => {
                    handler(command);
                }
                Err(e) => {
                    warn!("接收命令时出错: {}", e);
                    // 继续监听，除非是致命错误
                    continue;
                }
            }
        }
    }
}