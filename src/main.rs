use bevy::prelude::*;
use redra::graph::communicate::channels::RDChannel;
use redra::graph::init::material::initialize_materials;
use redra::module::camera::LookTransformPlugin;
use redra::{
    graph::{setup::rd_setup, update::rd_update},
    module::{camera::fps::*,parser::core::RDPack},
    net::listener::RDListener,
};
use tokio::sync::{broadcast, mpsc};


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

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

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.9))) // 设置较亮的背景色
        .add_plugins(DefaultPlugins)
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .insert_resource(channel)
        .add_systems(Startup, (rd_setup, initialize_materials))
        .add_systems(Update, rd_update)
        .run();
    Ok(())
}
