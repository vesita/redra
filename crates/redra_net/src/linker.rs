use bevy::prelude::*;
use expto::rdmp::Unit;
use expto::rdmp::decoding::decode_and_next;
use log::{info, error, debug};
use tokio::sync::broadcast;
use tokio::{net::TcpStream, sync::mpsc};
use tokio::io::{AsyncReadExt};


// #[derive(Resource)]
// pub struct LinkerPool {
//     pool: Vec<ThLc<RDLinker>>,
//     id_pool: Vec<usize>,
//     holder: mpsc::Receiver<usize>,
//     release: mpsc::Sender<usize>,
// }


// impl LinkerPool {
//     pub fn new() -> LinkerPool {
//         info!("初始化连接池...");
//         let (release, holder) = mpsc::channel::<usize>(64);
//         LinkerPool {
//             pool: Vec::new(),
//             id_pool: vec![0],
//             holder,
//             release,
//         }
//     }

//     pub fn get_id(&mut self) -> usize {
//         match self.id_pool.pop() {
//             Some(id) => {
//                 if self.id_pool.is_empty() {
//                     self.id_pool.push(id + 1);
//                 }
//                 id
//             },
//             None => {
//                 panic!("没有可用的ID");
//             }
//         }
//     }

//     pub fn release(&mut self, id: usize) {
//         self.id_pool.push(id);
//     }

//     pub async fn start_linker(
//         &mut self,
//         socket: TcpStream,
//         sender: mpsc::Sender<Unit>,
//         receiver: broadcast::Receiver<Unit>,
//     ) {
//         let id = self.get_id();
//         let release_clone = self.release.clone();  // 提前克隆release通道
//         let mut linker = RDLinker::new(id, socket, sender, receiver);
//         tokio::spawn(async move {
//             linker.run(release_clone).await;
//         });
//         self.pool.push(Arc::new(Mutex::new(linker)));
//     }
// }

pub async fn start_linker(
    id: usize,
    release: mpsc::Sender<usize>,
    socket: TcpStream,
    sender: mpsc::Sender<Unit>,
    receiver: broadcast::Receiver<Unit>,
) {
    let mut linker = RDLinker::new(id, socket, sender, receiver);
    tokio::spawn(async move {
        linker.run(release).await;
    });
}

/// 连接处理器，负责处理单个TCP连接的数据读取和转发
/// 
/// 该结构体管理一个TCP连接，持续从连接中读取原始数据，
/// 并将其转发到工作分配器的缓冲区，由forwarder负责解析
pub struct RDLinker {
    /// 连接的唯一标识ID
    pub id: usize,
    /// TCP连接套接字
    pub socket: TcpStream,

    pub sender: mpsc::Sender<Unit>,
    pub receiver: broadcast::Receiver<Unit>,
}


impl RDLinker {
    /// 创建一个新的连接处理器实例
    /// 
    /// # 参数
    /// * `id` - 连接的唯一标识ID
    /// * `socket` - TCP连接套接字
    /// * `wosh` - 工作分配器的共享引用
    /// * `expand_request` - 用于请求扩展转发任务的发送器
    /// 
    /// # 返回值
    /// * `RDLinker` - 新创建的连接处理器实例
    pub fn new(
        id: usize,
        socket: TcpStream,
        sender: mpsc::Sender<Unit>,
        receiver: broadcast::Receiver<Unit>,
    ) -> RDLinker {
        RDLinker {
            id,
            socket,
            sender,
            receiver,
        }
    }
    
    /// 启动连接处理器，开始处理TCP连接数据
    /// 
    /// 该方法进入一个循环，持续从TCP连接中读取原始数据，
    /// 并将其发送到转发任务进行解析
    /// 
    /// # 参数
    /// * `release` - 用于发送连接释放通知的发送器
    pub async fn run(&mut self, release: mpsc::Sender<usize>) {
        info!("启动TCP链接处理器 ID: {}", self.id);
        
        let mut total_bytes_received = 0;
        let mut packets_received = 0;

        // 使用更大的缓冲区来减少系统调用
        let mut buffer = [0; 1024];
        
        // 累积缓冲区，用于处理TCP拆包/粘包
        let mut accum_buffer = Vec::new();

        loop {
            let result = self.socket.read(&mut buffer).await;
            match result {
                Ok(0) => {
                    info!("客户端主动断开连接，退出链接处理器 ID: {}", self.id);
                    break;
                },
                Ok(len) => {
                    total_bytes_received += len;
                    packets_received += 1;
                    
                    debug!("从TCP连接 ID: {} 接收到 {} 字节数据，累计接收: {} 字节，数据包序号: {}", 
                            self.id, len, total_bytes_received, packets_received);
                    
                    // 将新接收的数据追加到累积缓冲区
                    accum_buffer.extend_from_slice(&buffer[..len]);
                    
                    // 处理累积缓冲区中的完整数据包
                    loop {
                        match decode_and_next(&mut accum_buffer) {
                            Ok((unit, remaining)) => {
                                // 发送解析出的协议单元
                                if let Err(e) = self.sender.send(unit).await {
                                    error!("发送解析后的数据包失败: {}", e);
                                    break;
                                }
                                
                                // 更新累积缓冲区为剩余数据
                                accum_buffer = remaining.to_vec();
                            }
                            Err(e) => {
                                debug!("解析数据包时出错或没有完整数据包: {}，等待更多数据", e);
                                // 没有完整的数据包可供解析，跳出内循环等待更多数据
                                break;
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("从TCP连接ID: {} 读取数据失败: {}", self.id, e);
                    break;
                }
            }
        }
        release.send(self.id).await.expect("释放资源失败");
        info!("TCP链接处理器任务结束，ID: {}，总计处理 {} 字节，{} 个数据包", 
              self.id, total_bytes_received, packets_received);
    }
}