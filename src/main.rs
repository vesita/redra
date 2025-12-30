use bevy::prelude::*;
use redra::module::camera::LookTransformPlugin;
use redra::module::resource::channel::RDChannel;
use redra::{
    graph::{setup::rd_setup, update::rd_update},
    module::{resource::RDResource, camera::fps::*,parser::core::RDPack},
    net::listener::RDListener,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};


#[derive(Debug)]
enum AppState {
    Loading,
    Playing,
}

#[tokio::main]
async fn main() -> Result<(), AppState> {
    // // 初始化 shutdown channel
    // let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

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
        net.run().await;
    });

    let resource = RDResource {
        channel: channel,
        materials: HashMap::new(),
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
    // let _ = shutdown_tx.send(());
    Ok(())
}
