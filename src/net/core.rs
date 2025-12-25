use std::{collections::VecDeque, time::Duration};

use bevy::log::tracing::instrument::WithSubscriber;
use socket2::Socket;
use tokio::{sync::{broadcast, mpsc}, time::sleep};

use crate::{net::{forwarder::{self, RDForwarder}, linker::RDLinker, listener::RDListener}, parser::core::RDPack};

pub struct RDNet {
    pub linkers: Vec<tokio::task::JoinHandle<()>>,
    pub forwarders: Vec<tokio::task::JoinHandle<()>>,
    pub released: VecDeque<usize>,
    pub to_engine: mpsc::Sender<RDPack>,
    pub engine_broadcast: broadcast::Receiver<RDPack>,
}

impl RDNet {
    pub fn new(
        to_engine: mpsc::Sender<RDPack>,
        engine_broadcast: broadcast::Receiver<RDPack>,
    ) -> Self {
        RDNet {
            linkers: vec![],
            forwarders: vec![],
            released: VecDeque::new(),
            to_engine,
            engine_broadcast,
        }
    }

    pub async fn run(&mut self) {
        let (listen_tx, mut rx) = mpsc::channel::<Socket>(16);

        let listener = RDListener::new(listen_tx, "0.0.0.0:8080".to_string());
        tokio::spawn(async move {
            listener.run().await;
        });
        println!("来自RDNet");
        let mut connection_id: usize = 0;
        loop {
            match rx.recv().await {
                Some(socket) => {
                    connection_id += 1;
                    let (link_tx, link_rx) = mpsc::channel::<Vec<u8>>(1024);
                    let (forward_tx, forward_rx) = mpsc::channel::<Vec<u8>>(1024);
                    
                    let linker = RDLinker::new(link_tx, link_rx, socket);
                    let forwarder = RDForwarder::new(
                        forward_tx,
                        forward_rx,
                        self.to_engine.clone(),
                        self.engine_broadcast.resubscribe()
                    );
                    println!("链接: {}", connection_id);
                    let linker_task = tokio::spawn(async move {
                        println!("启动链接任务 {}", connection_id);
                        linker.run().await;
                        println!("链接任务 {} 结束", connection_id);
                    });
                    
                    let forwarder_task = tokio::spawn(async move {
                        println!("启动转发任务 {}", connection_id);
                        forwarder.run().await;
                        println!("转发任务 {} 结束", connection_id);
                    });
                    
                    self.linkers.push(linker_task);
                    self.forwarders.push(forwarder_task);
                },
                None => {
                    // 添加一个小延迟以避免过度占用CPU
                    sleep(Duration::from_millis(10)).await;
                    continue;
                }
            }
        }
    }
}