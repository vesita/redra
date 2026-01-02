use bevy::prelude::*;
use redra::graph::communicate::channels::RDChannel;
use redra::graph::init::material::initialize_materials;
use redra::graph::GraphPlugin;
use redra::module::camera::LookTransformPlugin;
use redra::{
    graph::{setup::rd_setup, update::rd_update},
    module::{camera::fps::*,parser::core::RDPack},
    net::listener::RDListener,
};
use tokio::sync::{broadcast, mpsc};

use log::info;

/// 程序主入口函数
/// 
/// 此函数启动应用程序，初始化网络通信、图形渲染系统和UI组件
/// 使用Tokio异步运行时处理网络任务，并使用Bevy引擎渲染图形界面
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    // 初始化网络通信通道
    // engine_sender 和 net_receiver 用于引擎向网络模块广播消息
    // net_sender 和 engine_receiver 用于网络模块向引擎发送消息
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);

    // 创建通信通道结构体
    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    info!("启动网络任务...");
    
    // 启动网络监听任务
    tokio::spawn(async move {
        let mut net = RDListener::new(net_sender, net_receiver);
        net.run().await;
    });

    // 构建并运行Bevy应用程序
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.9))) // 设置较亮的背景色
        .add_plugins(DefaultPlugins) // 添加默认插件
        .add_plugins(FpsCameraPlugin::default()) // 添加FPS相机插件
        .add_plugins(LookTransformPlugin) // 添加相机变换插件
        .add_plugins(GraphPlugin) // 使用 GraphPlugin 替代 UiModule
        .insert_resource(channel) // 插入通信通道资源
        .add_systems(Startup, rd_setup) // 添加rd_setup系统
        .add_systems(Startup, initialize_materials) // 添加initialize_materials系统
        .add_systems(Update, rd_update) // 添加更新系统
        .run();
    Ok(())
}