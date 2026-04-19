use bevy::prelude::*;
use expto::rdmp::Unit;
use tokio::sync::{broadcast, mpsc};

use crate::listener::setup_listener;

pub mod linker;
pub mod listener;

// 定义通信通道资源
#[derive(Resource)]
pub struct RDChannel {
    pub redra_sender: broadcast::Sender<Unit>,
    pub redra_recver: mpsc::Receiver<Unit>,
}


// 跟踪网络状态的资源
#[derive(Resource, Default)]
pub struct NetworkStatus {
    pub task_finished: bool,
}

// 网络插件，负责初始化和管理网络服务
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_listener);
    }
}

// // 启动网络服务
// fn setup_network() {    
//     info!("启动网络任务...");
//     // 创建独立的Tokio运行时并在线程中运行网络监听器
//     // 修复策略：由于Bevy的IoTaskPool/AsyncComputeTaskPool基于async-std或类似的非Tokio运行时，
//     // 直接在其中使用tokio::net::TcpListener::bind会导致"there is no reactor running"错误。
//     // 因此，我们通过std::thread::spawn创建一个原生线程，并在其中构建独立的Tokio运行时(Runtime)，
//     // 从而为网络监听逻辑提供完整的Tokio上下文支持。
//     let _ = AsyncComputeTaskPool::get()
//         .spawn(async move {
//             std::thread::spawn(move || {
//                 // 创建独立的Tokio运行时
//                 let rt = Runtime::new().expect("Failed to create Tokio runtime");
//                 rt.block_on(async move {
//                     let mut net = RDListener::new();
//                     net.run().await;
//                 });
//             });
//         });
// }
