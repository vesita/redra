use bevy::prelude::*;
use redra::{channel::core::RDPack, module::resource::RDResource, net::listener::RDListener, setup::rd_setup, update::rd_update};
use tokio::sync::{broadcast, mpsc};
use std::sync::{Arc, Mutex};
use redra::module::resource::{channel::RDChannel, handle::RDHandle};
use smooth_bevy_cameras::{
    LookTransformPlugin,
    controllers::fps::{FpsCameraPlugin},
};

#[derive(Debug)]
enum AppState {
    Loading,
    Playing,
}

fn main() -> Result<(), AppState> {
    // 加载资源
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(64);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(64);
    
    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    let mut listener = RDListener::new(net_sender, net_receiver);
    listener.listen("0.0.0.0:8080");
    
    let handle = RDHandle {
        listener: Arc::new(Mutex::new(listener)),
        servers: std::collections::HashMap::new(),
    };

    let resource = RDResource {
        channel: Arc::new(Mutex::new(channel)),
        handle: Arc::new(Mutex::new(handle)),
        materials: std::collections::HashMap::new(),
    };
    
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .insert_resource(resource)
        .add_systems(Startup, rd_setup)
        .add_systems(Update, rd_update)
        .run();
    Ok(())
}