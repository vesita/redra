use bevy::prelude::*;
use redra::{module::resource::RDResource, net::listener::RDListener, parser::core::RDPack, setup::rd_setup, update::rd_update};
use tokio::sync::{broadcast, mpsc};
use std::sync::{Arc, Mutex, OnceLock};
use redra::module::resource::channel::RDChannel;
use smooth_bevy_cameras::{
    LookTransformPlugin,
    controllers::fps::{FpsCameraPlugin},
};

#[derive(Debug)]
enum AppState {
    Loading,
    Playing,
}


#[tokio::main]
async fn main() -> Result<(), AppState> {
    // 初始化 shutdown channel
    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);
    
    // 加载资源
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);
    
    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    println!("启动网络任务...");
    tokio::spawn(async move {
        let mut net = RDListener::new(net_sender, net_receiver);
        net.run(shutdown_rx).await;
    });

    let resource = RDResource {
        channel: Arc::new(Mutex::new(channel)),
        materials: std::collections::HashMap::new(),
    };
    
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.9))) // 设置较亮的背景色
        .add_plugins(DefaultPlugins)
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .insert_resource(resource)
        .add_systems(Startup, rd_setup)
        .add_systems(Update, rd_update)
        .run();
    let _ = shutdown_tx.send(());
    Ok(())
}