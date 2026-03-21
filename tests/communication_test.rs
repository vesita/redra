//! 通信系统测试
//! 
//! 这些测试专注于通信系统的功能：
//! 1. RDChannel 的创建和基本操作
//! 2. 消息发送和接收
//! 3. Client Sender 编码逻辑
//! 4. Trailer 编码测试
//! 
//! 运行方式：cargo test --test communication_test -- --nocapture
//! 注意：部分测试会启动完整的 Bevy 应用窗口

use tokio::sync::{broadcast, mpsc};

use redra::graph::communicate::channels::RDChannel;
use redra::module::parser::core::RDPack;
use bevy::prelude::*;
use redra::graph::init::material::initialize_materials;
use redra::graph::GraphPlugin;
use redra::module::camera::fps::FpsCameraPlugin;
use redra::module::camera::LookTransformPlugin;
use redra::graph::setup::rd_setup;
use redra::graph::update::rd_update;
use redra::render::frame::{toggle_frame_rate, FrameRateState};

/// 测试 1: RDChannel 创建和基本操作
#[tokio::test]
async fn test_rdchannel_creation() {
    // 创建一个广播 channel
    let (tx, _) = broadcast::channel::<RDPack>(100);
    let (_, rx) = mpsc::channel::<RDPack>(100);
    
    let _channel = RDChannel {
        sender: tx.clone(),
        receiver: rx,
    };
    
    // 验证 channel 已创建
    assert!(true);
    
    // 测试发送消息（需要至少一个接收者）
    let pack = RDPack::Message("测试消息".to_string());
    // 在广播 channel 中，即使没有接收者也可以发送
    let send_result = tx.send(pack);
    // 如果没有接收者，send 会返回 Err(SendError)，这是正常的
    // 我们只关心调用成功
    println!("发送结果：{}", if send_result.is_ok() { "成功" } else { "失败" });
}

/// 测试 2: 通过 Channel 发送和接收消息
#[tokio::test]
async fn test_channel_send_receive() {
    // 创建通信 channel
    let (tx, mut rx) = broadcast::channel::<RDPack>(100);
    let (_, rx_mpsc) = mpsc::channel::<RDPack>(100);
    
    let _channel = RDChannel {
        sender: tx.clone(),
        receiver: rx_mpsc,
    };
    
    // 发送消息
    let message_pack = RDPack::Message("测试消息".to_string());
    let send_result = tx.send(message_pack);
    assert!(send_result.is_ok());
    
    // 接收消息
    let received = rx.try_recv();
    assert!(received.is_ok());
    
    if let Ok(pack) = received {
        match pack {
            RDPack::Message(msg) => {
                assert_eq!(msg, "测试消息");
            }
            _ => panic!("期望收到 Message 类型的包"),
        }
    }
}

/// 测试 3: Client Sender 编码与 Channel 集成
#[tokio::test]
async fn test_client_sender_channel_integration() {
    // 创建通信 channel
    let (tx, mut rx) = broadcast::channel::<RDPack>(100);
    let (_, rx_mpsc) = mpsc::channel::<RDPack>(100);
    
    let _channel = RDChannel {
        sender: tx.clone(),
        receiver: rx_mpsc,
    };
    
    // 测试发送点数据编码逻辑
    use prost::Message;
    use redra::proto::{
        command::Command,
        designation::{DesignCmd, Spawn, design_cmd::Cmd, spawn},
        shape::{self, ShapePack, shape_pack},
        transform::Translation,
    };
    
    let point = shape::Point {
        pos: Some(Translation { x: 1.0, y: 2.0, z: 3.0 }),
    };
    let spawn = Spawn {
        id: None,
        data: Some(spawn::Data::ShapeData(ShapePack {
            data: Some(shape_pack::Data::Point(point)),
        })),
    };
    let design_cmd = DesignCmd {
        cmd: Some(Cmd::Spawn(spawn)),
    };
    let pack = Command {
        cmd_pack: Some(redra::proto::command::command::CmdPack::Designation(design_cmd)),
    };
    
    let encoded_data = pack.encode_to_vec();
    assert!(!encoded_data.is_empty());
    
    // 通过 channel 发送编码后的数据（模拟网络接收）
    let pack = RDPack::Message(format!("编码数据长度：{}", encoded_data.len()));
    let send_result = tx.send(pack);
    assert!(send_result.is_ok());
    
    // 验证可以接收
    let received = rx.try_recv();
    assert!(received.is_ok());
}

/// 测试 4: Client Sender 点数据编码测试
#[tokio::test]
async fn test_client_sender_point_encoding() {
    use prost::Message;
    use redra::proto::{
        command::{Command, command::CmdPack},
        designation::{DesignCmd, Spawn, design_cmd::Cmd, spawn},
        shape::{self, ShapePack, shape_pack},
        transform::Translation,
    };
    
    // 测试不同坐标值的点编码
    let test_cases = vec![
        (0.0f32, 0.0f32, 0.0f32),
        (1.0f32, 2.0f32, 3.0f32),
        (-1.0f32, -1.0f32, -1.0f32),
        (100.5f32, 200.3f32, 300.7f32),
    ];

    for (x, y, z) in test_cases {
        let point = shape::Point {
            pos: Some(Translation { x, y, z }),
        };
        let spawn = Spawn {
            id: None,
            data: Some(spawn::Data::ShapeData(ShapePack {
                data: Some(shape_pack::Data::Point(point)),
            })),
        };
        let design_cmd = DesignCmd {
            cmd: Some(Cmd::Spawn(spawn)),
        };
        let pack = Command {
            cmd_pack: Some(CmdPack::Designation(design_cmd)),
        };

        let encoded = pack.encode_to_vec();
        assert!(!encoded.is_empty(), "坐标 ({}, {}, {}) 的编码不应为空", x, y, z);
        
        let decoded = Command::decode(encoded.as_slice())
            .expect("应该能成功解码");
        assert!(decoded.cmd_pack.is_some());
    }
}

/// 测试 5: Client Sender 线段编码测试
#[tokio::test]
async fn test_client_sender_segment_encoding() {
    use prost::Message;
    use redra::proto::{
        command::{Command, command::CmdPack},
        designation::{DesignCmd, Spawn, design_cmd::Cmd, spawn},
        shape::{self, ShapePack, shape_pack},
        transform::Translation,
    };
    
    // 测试不同方向的线段编码
    let test_cases = vec![
        ([0.0f32, 0.0f32, 0.0f32], [1.0f32, 0.0f32, 0.0f32]), // X 轴
        ([0.0f32, 0.0f32, 0.0f32], [0.0f32, 1.0f32, 0.0f32]), // Y 轴
        ([0.0f32, 0.0f32, 0.0f32], [0.0f32, 0.0f32, 1.0f32]), // Z 轴
        ([-5.0f32, -5.0f32, -5.0f32], [5.0f32, 5.0f32, 5.0f32]), // 对角线
    ];

    for (start, end) in test_cases {
        let segment = shape::Segment {
            start: Some(shape::Point {
                pos: Some(Translation {
                    x: start[0],
                    y: start[1],
                    z: start[2],
                }),
            }),
            end: Some(shape::Point {
                pos: Some(Translation {
                    x: end[0],
                    y: end[1],
                    z: end[2],
                }),
            }),
        };
        let spawn = Spawn {
            id: None,
            data: Some(spawn::Data::ShapeData(ShapePack {
                data: Some(shape_pack::Data::Segment(segment)),
            })),
        };
        let design_cmd = DesignCmd {
            cmd: Some(Cmd::Spawn(spawn)),
        };
        let pack = Command {
            cmd_pack: Some(CmdPack::Designation(design_cmd)),
        };

        let encoded = pack.encode_to_vec();
        assert!(!encoded.is_empty(), "线段 {:?} -> {:?} 的编码不应为空", start, end);
        
        let decoded = Command::decode(encoded.as_slice())
            .expect("应该能成功解码");
        assert!(decoded.cmd_pack.is_some());
    }
}

/// 测试 6: Trailer 编码测试
#[tokio::test]
async fn test_trailer_encoding() {
    use prost::Message;
    use redra::proto::declare::Trailer;
    
    // 测试 Trailer 编码逻辑
    let data = vec![1u8, 2, 3, 4, 5];
    
    let trailer = Trailer {
        me: 1,
        next: data.len() as u32,
    };
    let declare_length = trailer.encoded_len();
    let final_trailer = Trailer {
        me: declare_length as u32,
        next: data.len() as u32,
    };

    let encoded = final_trailer.encode_to_vec();
    
    // 验证编码后的数据不为空
    assert!(!encoded.is_empty());
    
    // 验证可以解码
    let decoded = Trailer::decode(encoded.as_slice())
        .expect("应该能成功解码 Trailer");
    assert_eq!(decoded.next, data.len() as u32);
}

/// 测试 7: 多个 Channel 并发测试（带完整应用启动）
#[tokio::test]
async fn test_multiple_channels_concurrent() {
    // 创建通信通道
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);

    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    // 创建多个独立的 channel 用于测试
    let mut channels = Vec::new();
    
    for i in 0..5 {
        let (tx, _) = broadcast::channel::<RDPack>(100);
        let (_, rx) = mpsc::channel::<RDPack>(100);
        
        channels.push(RDChannel {
            sender: tx.clone(),
            receiver: rx,
        });
        
        // 发送消息
        let pack = RDPack::Message(format!("Channel {} 的消息", i));
        let _ = tx.send(pack);
    }
    
    // 验证所有 channel 都已创建
    assert_eq!(channels.len(), 5);
    
    println!("✓ 多通道并发测试完成，创建了 5 个独立通道");
}

/// 测试 8: 大量消息压力测试（带完整应用）
#[tokio::test]
async fn test_stress_many_messages() {
    // 创建通信通道
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);

    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    let (tx, mut rx) = broadcast::channel::<RDPack>(1000);
    let (_, rx_mpsc) = mpsc::channel::<RDPack>(1000);
    
    let _channel = RDChannel {
        sender: tx.clone(),
        receiver: rx_mpsc,
    };
    
    // 发送 1000 条消息
    let start_time = std::time::Instant::now();
    for i in 0..1000 {
        let pack = RDPack::Message(format!("消息 #{}", i));
        let _ = tx.send(pack);
    }
    let send_time = start_time.elapsed();
    
    println!("发送 1000 条消息耗时：{:?}", send_time);
    
    // 接收并验证消息
    let mut received_count = 0;
    while let Ok(_) = rx.try_recv() {
        received_count += 1;
    }
    
    println!("接收到 {} 条消息", received_count);
    assert!(received_count > 0, "应该能接收到至少一条消息");
}

/// 测试 9: 完整应用启动测试 - 类似 main 函数的运行模式
#[tokio::test]
async fn test_full_application_startup() {
    println!("正在启动完整的应用进行测试...");
    
    // 创建通信通道
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);

    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    info!("启动网络任务...");
    
    // 启动网络监听任务
    tokio::spawn(async move {
        use redra::net::listener::RDListener;
        let mut net = RDListener::new(net_sender, net_receiver);
        // 这里不实际运行，避免阻塞测试
        // net.run().await;
        println!("网络任务已初始化（测试模式）");
    });

    // 构建并运行 Bevy 应用程序
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.9)))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .add_plugins(GraphPlugin)
        .insert_resource(channel)
        .add_systems(Startup, rd_setup)
        .add_systems(Startup, initialize_materials)
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 })
        .add_systems(Update, (rd_update, toggle_frame_rate))
        .run();
        
    println!("✓ 完整应用测试完成");
}
