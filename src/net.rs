use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use tokio::sync::{broadcast, mpsc};
use log::info;
use tokio::runtime::Runtime;

use crate::{
    module::parser::core::RDPack,
    graph::communicate::channels::RDChannel,
};

mod listener;
mod forwarder;
mod linker;
mod work_share;

use listener::RDListener;

// 网络插件资源，用于存储网络通信通道
#[derive(Resource)]
pub struct NetworkHandles {
    pub handle: Task<()>,
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
            .init_resource::<NetworkStatus>()
            .add_systems(Startup, setup_network);
    }
}

// 启动网络服务
fn setup_network(
    mut commands: Commands,
) {    

    // 创建网络通信通道
    // engine_sender 和 net_receiver 用于引擎向网络模块广播消息
    // net_sender 和 engine_receiver 用于网络模块向引擎发送消息
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);

    // 插入通道资源
    commands.insert_resource(RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    });

    info!("启动网络任务...");

    // 创建独立的Tokio运行时并在线程中运行网络监听器
    // 修复策略：由于Bevy的IoTaskPool/AsyncComputeTaskPool基于async-std或类似的非Tokio运行时，
    // 直接在其中使用tokio::net::TcpListener::bind会导致"there is no reactor running"错误。
    // 因此，我们通过std::thread::spawn创建一个原生线程，并在其中构建独立的Tokio运行时(Runtime)，
    // 从而为网络监听逻辑提供完整的Tokio上下文支持。
    let task = AsyncComputeTaskPool::get()
        .spawn(async move {
            std::thread::spawn(move || {
                // 创建独立的Tokio运行时
                let rt = Runtime::new().expect("Failed to create Tokio runtime");
                rt.block_on(async move {
                    let mut net = RDListener::new(net_sender, net_receiver);
                    net.run().await;
                });
            });
        });

    // 存储网络句柄作为资源
    commands.insert_resource(NetworkHandles { handle: task });
}

// // 检查网络状态
// fn check_network_status(
//     net_handles: Res<NetworkHandles>,
//     mut network_status: ResMut<NetworkStatus>,
// ) {
//     // 检查网络任务是否仍在运行
//     if net_handles.handle.is_finished() && !network_status.task_finished {
//         info!("网络任务已结束");
//         network_status.task_finished = true;
//     }
// }