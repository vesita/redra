use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use expto::ip::get_addr;

#[derive(Clone)]
pub struct Link {
    stream: Arc<tokio::sync::Mutex<TcpStream>>,
}

impl Link {
    pub async fn connect() -> Result<Self, String> {
        let addr = get_addr();
        let addr_str = addr.clone(); // 克隆地址用于错误消息
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                // 启用Nagle算法的禁用以获得更低的延迟
                stream.set_nodelay(true).map_err(|e| e.to_string())?;
                Ok(Link {
                    stream: Arc::new(tokio::sync::Mutex::new(stream)),
                })
            }
            Err(e) => Err(format!("Failed to connect to {}: {}", addr_str, e)),
        }
    }

    pub async fn send(&self, data: &[u8]) -> Result<(), String> {
        let mut stream = self.stream.lock().await;
        match stream.write_all(data).await {
            Ok(()) => {
                // 确保数据发送完成
                match stream.flush().await {
                    Ok(()) => Ok(()),
                    Err(e) => Err(format!("Failed to flush stream: {}", e)),
                }
            }
            Err(e) => Err(format!("Failed to write to stream: {}", e)),
        }
    }

    pub async fn send_with_timeout(&self, data: &[u8], timeout_seconds: u64) -> Result<(), String> {
        match tokio::time::timeout(std::time::Duration::from_secs(timeout_seconds), self.send(data)).await {
            Ok(result) => result,
            Err(_) => Err("Send operation timed out".to_string()),
        }
    }

    pub fn get_inner_stream(&self) -> &Arc<tokio::sync::Mutex<TcpStream>> {
        &self.stream
    }
}

use tokio::sync::OnceCell;

static GLOBAL_CONNECTION: OnceCell<Arc<Link>> = OnceCell::const_new();

pub async fn get_link() -> Arc<Link> {
    GLOBAL_CONNECTION
        .get_or_init(|| async {
            Arc::new(
                Link::connect()
                    .await
                    .expect("Failed to initialize global connection"),
            )
        })
        .await
        .clone()
}
