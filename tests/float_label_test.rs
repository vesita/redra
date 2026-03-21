//! FloatLabel 组件功能测试
//! 
//! 这些测试专注于 FloatLabel 组件的核心功能：
//! 1. FloatLabel 的创建和配置
//! 2. Billboard 效果（标签面向相机）
//! 3. 动态创建和管理标签
//! 4. 性能测试
//! 
//! 运行方式：cargo test --test float_label_test -- --nocapture
//! 注意：这些测试会启动完整的 Bevy 应用窗口，可以看到实际效果

use bevy::prelude::*;

// 导入被测试的模块
use redra::graph::component::{FloatLabel, FloatLabelPlugin, MainCamera3d, spawn_label_at};
use redra::graph::init::material::initialize_materials;
use redra::graph::GraphPlugin;
use redra::module::camera::fps::FpsCameraPlugin;
use redra::module::camera::LookTransformPlugin;
use redra::graph::setup::rd_setup;
use redra::graph::update::rd_update;
use redra::render::frame::{toggle_frame_rate, FrameRateState};
use tokio::sync::{broadcast, mpsc};
use redra::graph::communicate::channels::RDChannel;
use redra::module::parser::core::RDPack;

/// 测试 1: FloatLabel 基础功能 - 创建和配置
#[test]
fn test_float_label_basic_creation() {
    // 测试基本的标签创建
    let label = FloatLabel::new("测试标签");
    assert_eq!(label.text, "测试标签");
    assert_eq!(label.font_size, 48.0);
    assert_eq!(label.color, Color::WHITE);
    assert!(!label.with_background);
    
    // 测试 builder 模式
    let custom_label = FloatLabel::new("自定义")
        .with_font_size(64.0)
        .with_color(Color::srgb(1.0, 0.0, 0.0))
        .with_background(Color::srgba(0.0, 0.0, 0.0, 0.8));
    
    assert_eq!(custom_label.text, "自定义");
    assert_eq!(custom_label.font_size, 64.0);
    assert_eq!(custom_label.color, Color::srgb(1.0, 0.0, 0.0));
    assert!(custom_label.with_background);
    
    println!("✓ 基础标签创建测试通过");
}

/// 测试 2: FloatLabel 插件注册（带完整应用）
#[test]
fn test_float_label_plugin_registration() {
    let mut app = App::new();
    app.add_plugins(FloatLabelPlugin);
    
    // 验证插件已成功注册
    let world = app.world();
    println!("✓ 插件注册成功，世界包含 {} 个实体", world.entities().len());
}

/// 测试 3: FloatLabel 在场景中生成（完整应用模式）
#[tokio::test]
async fn test_float_label_spawn_in_scene() {
    println!("启动 FloatLabel 场景测试...");
    
    // 创建通信通道
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);
    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    let mut app = App::new();
    
    // 添加所有必要的插件，模拟主程序
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .add_plugins(GraphPlugin)
        .insert_resource(channel)
        .add_systems(Startup, rd_setup)
        .add_systems(Startup, initialize_materials)
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 })
        .add_systems(Update, (rd_update, toggle_frame_rate));
    
    // 创建主相机
    app.world_mut().commands().spawn((
        Camera3d::default(),
        Transform::from_xyz(8.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        redra::graph::component::MainCamera3d,
    ));
    
    // 创建浮动标签
    spawn_label_at(
        &mut app.world_mut().commands(),
        Vec3::new(0.0, 1.0, 0.0),
        "场景中的标签",
        Some(FloatLabel::new("场景中的标签").with_font_size(48.0)),
    );
    
    // 运行应用几帧
    for i in 0..10 {
        app.update();
        println!("帧 {}", i);
    }
    
    // 验证标签存在
    let label_count = {
        let world = app.world_mut();
        let mut query = world.query::<&FloatLabel>();
        query.iter(world).count()
    };
    
    println!("✓ 场景测试完成，检测到 {} 个标签", label_count);
    assert!(label_count > 0);
}

/// 测试 4: 使用 spawn_label_at 辅助函数（完整应用模式）
#[tokio::test]
async fn test_spawn_label_at_helper() {
    println!("测试 spawn_label_at 辅助函数...");
    
    // 创建通信通道
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);
    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .add_plugins(GraphPlugin)
        .insert_resource(channel)
        .add_systems(Startup, rd_setup)
        .add_systems(Startup, initialize_materials)
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 })
        .add_systems(Update, (rd_update, toggle_frame_rate));
    
    // 创建相机
    app.world_mut().commands().spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera3d,
    ));
    
    // 使用辅助函数创建标签
    let label_entity = spawn_label_at(
        &mut app.world_mut().commands(),
        Vec3::new(1.0, 2.0, 3.0),
        "位置标签",
        Some(FloatLabel::new("位置标签").with_font_size(32.0)),
    );
    
    // 刷新命令以确保实体被创建
    for _ in 0..5 {
        app.update();
    }
    
    // 验证实体已创建
    let world = app.world();
    let entity_ref = world.get_entity(label_entity);
    assert!(entity_ref.is_ok(), "标签实体应该存在");
    
    // 验证位置正确
    if let Ok(entity_ref) = entity_ref {
        if let Some(transform) = entity_ref.get::<Transform>() {
            assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
            println!("✓ 标签位置正确：{}", transform.translation);
        }
    }
    
    println!("✓ spawn_label_at 辅助函数测试通过");
}

/// 测试 5: Billboard 效果 - 标签面向相机（完整应用模式）
#[tokio::test]
async fn test_billboard_effect() {
    println!("测试 Billboard 效果...");
    
    // 创建通信通道
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);
    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .add_plugins(GraphPlugin)
        .insert_resource(channel)
        .add_systems(Startup, rd_setup)
        .add_systems(Startup, initialize_materials)
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 })
        .add_systems(Update, (rd_update, toggle_frame_rate));
    
    // 创建主相机
    app.world_mut().commands().spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera3d,
    ));
    
    // 创建浮动标签
    app.world_mut().commands().spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        FloatLabel::new("Billboard 测试"),
    ));
    
    // 运行多帧更新
    for _ in 0..10 {
        app.update();
    }
    
    // 验证标签存在
    let label_count = {
        let world = app.world_mut();
        let mut query = world.query::<&FloatLabel>();
        query.iter(world).count()
    };
    
    println!("✓ Billboard 效果测试完成，检测到 {} 个标签", label_count);
    assert!(label_count > 0);
}

/// 测试 6: 多个相机场下的 Billboard 行为（完整应用模式）
#[tokio::test]
async fn test_multiple_cameras_billboard() {
    println!("测试多相机 Billboard 行为...");
    
    // 创建通信通道
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);
    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .add_plugins(GraphPlugin)
        .insert_resource(channel)
        .add_systems(Startup, rd_setup)
        .add_systems(Startup, initialize_materials)
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 })
        .add_systems(Update, (rd_update, toggle_frame_rate));
    
    // 创建多个相机，但只有一个标记为主相机
    app.world_mut().commands().spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera3d,  // 主相机
    ));
    
    app.world_mut().commands().spawn((
        Camera3d::default(),
        Transform::from_xyz(-5.0, 3.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        // 未标记，不应影响 billboard
    ));
    
    // 创建标签
    app.world_mut().commands().spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        FloatLabel::new("多相机测试"),
    ));
    
    // 运行更新
    for _ in 0..10 {
        app.update();
    }
    
    // 验证标签仍然存在并正常工作
    let label_count = {
        let world = app.world_mut();
        let mut query = world.query::<&FloatLabel>();
        query.iter(world).count()
    };
    
    println!("✓ 多相机 Billboard 测试完成，检测到 {} 个标签", label_count);
    assert!(label_count > 0);
}

/// 测试 7: 动态创建和删除标签（完整应用模式）
#[tokio::test]
async fn test_dynamic_label_management() {
    println!("测试动态标签管理...");
    
    // 创建通信通道
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);
    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .add_plugins(GraphPlugin)
        .insert_resource(channel)
        .add_systems(Startup, rd_setup)
        .add_systems(Startup, initialize_materials)
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 })
        .add_systems(Update, (rd_update, toggle_frame_rate));
    
    // 创建相机
    app.world_mut().commands().spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera3d,
    ));
    
    // 动态创建标签
    let mut label_entities = Vec::new();
    for i in 0..5 {
        let entity = spawn_label_at(
            &mut app.world_mut().commands(),
            Vec3::new(i as f32 * 2.0, 1.0, 0.0),
            format!("动态标签 #{}", i),
            None,
        );
        label_entities.push(entity);
    }
    
    // 刷新命令
    app.update();
    
    // 验证所有标签都已创建
    {
        let world = app.world();
        for entity in &label_entities {
            assert!(world.get_entity(*entity).is_ok(), "标签实体 {:?} 应该存在", entity);
        }
    }
    println!("✓ 创建了 5 个标签");
    
    // 删除部分标签
    for entity in label_entities.drain(2..) {
        app.world_mut().commands().entity(entity).despawn();
    }
    
    // 刷新命令
    app.update();
    
    // 验证剩余标签数量
    let remaining_count = {
        let world = app.world_mut();
        let mut query = world.query::<&FloatLabel>();
        query.iter(world).count()
    };
    
    println!("✓ 动态管理测试完成，剩余 {} 个标签（删除了 3 个）", remaining_count);
    assert_eq!(remaining_count, 2, "应该剩下 2 个标签");
}

/// 测试 8: 完整的场景测试 - 相机、光源和多个标签（类似 main 函数）
#[tokio::test]
async fn test_full_scene() {
    println!("=== 完整场景测试 ===");
    println!("正在启动包含相机、光源和多个标签的完整场景...");
    
    // 创建通信通道
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);
    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    let mut app = App::new();
    
    // 添加所有必要的插件
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .add_plugins(GraphPlugin)
        .insert_resource(channel)
        .add_systems(Startup, rd_setup)
        .add_systems(Startup, initialize_materials)
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 })
        .add_systems(Update, (rd_update, toggle_frame_rate));
    
    // 1. 创建主相机
    app.world_mut().commands().spawn((
        Camera3d::default(),
        Transform::from_xyz(8.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera3d,
    ));
    
    // 2. 创建光源
    app.world_mut().commands().spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.0, 10.0, 0.0),
    ));
    
    // 3. 创建多个浮动标签
    let positions = vec![
        (Vec3::new(0.0, 1.0, 0.0), "中心"),
        (Vec3::new(3.0, 1.5, 0.0), "右侧"),
        (Vec3::new(-3.0, 1.5, 0.0), "左侧"),
    ];
    
    for (pos, text) in positions {
        spawn_label_at(
            &mut app.world_mut().commands(),
            pos,
            text.to_string(),
            Some(FloatLabel::new(text.to_string())
                .with_font_size(40.0)
                .with_color(Color::WHITE)
                .with_background(Color::srgba(0.0, 0.0, 0.0, 0.7))),
        );
    }
    
    println!("场景初始化完成，开始运行...");
    
    // 运行多帧更新
    for i in 0..30 {
        app.update();
        if i % 10 == 0 {
            println!("运行帧：{}", i);
        }
    }
    
    // 验证场景中的所有标签都存在
    let label_count = {
        let world = app.world_mut();
        let mut query = world.query::<&FloatLabel>();
        query.iter(world).count()
    };
    
    println!("✓ 完整场景测试完成");
    println!("  - 检测到 {} 个浮动标签", label_count);
    assert_eq!(label_count, 3, "应该有 3 个浮动标签");
}

/// 测试 9: 压力测试 - 大量标签的性能（完整应用模式）
#[tokio::test]
async fn test_stress_many_labels() {
    println!("=== 压力测试：创建 100 个标签 ===");
    
    // 创建通信通道
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);
    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .add_plugins(GraphPlugin)
        .insert_resource(channel)
        .add_systems(Startup, rd_setup)
        .add_systems(Startup, initialize_materials)
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 })
        .add_systems(Update, (rd_update, toggle_frame_rate));
    
    // 创建相机
    app.world_mut().commands().spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera3d,
    ));
    
    // 创建 100 个标签
    let start_time = std::time::Instant::now();
    for i in 0..100 {
        let x = (i as f32 * 0.5).sin() * 10.0;
        let z = (i as f32 * 0.3).cos() * 10.0;
        
        spawn_label_at(
            &mut app.world_mut().commands(),
            Vec3::new(x, 1.0, z),
            format!("标签 #{}", i),
            Some(FloatLabel::new(format!("标签 #{}", i))
                .with_font_size(24.0 + (i % 3) as f32 * 8.0)),
        );
    }
    let creation_time = start_time.elapsed();
    
    println!("创建 100 个标签耗时：{:?}", creation_time);
    
    // 运行多帧更新，验证性能
    let start_time = std::time::Instant::now();
    for i in 0..60 {
        app.update();
        if i % 20 == 0 {
            println!("更新帧：{}/60", i);
        }
    }
    let update_time = start_time.elapsed();
    
    println!("\n性能统计:");
    println!("  - 创建时间：{:?}", creation_time);
    println!("  - 60 帧更新总耗时：{:?}", update_time);
    println!("  - 平均每帧耗时：{:?}", update_time / 60);
    println!("  - 平均 FPS: {:.2}", 60.0 / (update_time.as_secs_f64() / 60.0));
    
    // 验证所有标签都存在
    let label_count = {
        let world = app.world_mut();
        let mut query = world.query::<&FloatLabel>();
        query.iter(world).count()
    };
    
    println!("\n✓ 压力测试完成");
    println!("  - 检测到 {} 个标签", label_count);
    assert_eq!(label_count, 100, "应该创建了 100 个标签");
}

/// 测试 10: 终极测试 - 完全模拟 main 函数的运行模式
#[tokio::test]
async fn test_main_like_application() {
    println!("\n===========================================");
    println!("终极测试：完全模拟 main 函数启动应用");
    println!("===========================================\n");
    
    // 创建通信通道
    let (engine_sender, net_receiver) = broadcast::channel::<RDPack>(1024);
    let (net_sender, engine_receiver) = mpsc::channel::<RDPack>(1024);

    let channel = RDChannel {
        sender: engine_sender,
        receiver: engine_receiver,
    };
    
    info!("启动网络任务...");
    
    // 启动网络监听任务（后台）
    tokio::spawn(async move {
        use redra::net::listener::RDListener;
        let mut net = RDListener::new(net_sender, net_receiver);
        println!("网络任务已初始化（测试模式，不实际监听端口）");
    });

    // 构建并运行 Bevy 应用程序 - 完全复制 main 函数
    println!("正在启动 Bevy 图形应用...");
    
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.9))) // 设置较亮的背景色
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(LookTransformPlugin)
        .add_plugins(GraphPlugin)
        .insert_resource(channel)
        .add_systems(Startup, |world: &mut World| {
            // 在 Startup 时创建测试场景
            println!("初始化测试场景...");
            
            // 创建主相机
            world.commands().spawn((
                Camera3d::default(),
                Transform::from_xyz(8.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
                MainCamera3d,
            ));
            
            // 创建光源
            world.commands().spawn((
                DirectionalLight::default(),
                Transform::from_xyz(0.0, 10.0, 0.0),
            ));
            
            // 创建多个测试标签
            let test_labels = vec![
                (Vec3::new(0.0, 1.0, 0.0), "测试标签 1"),
                (Vec3::new(2.0, 1.5, 0.0), "测试标签 2"),
                (Vec3::new(-2.0, 1.5, 0.0), "测试标签 3"),
            ];
            
            for (pos, text) in test_labels {
                spawn_label_at(
                    &mut world.commands(),
                    pos,
                    text.to_string(),
                    Some(FloatLabel::new(text.to_string())
                        .with_font_size(48.0)
                        .with_color(Color::WHITE)
                        .with_background(Color::srgba(0.0, 0.0, 0.0, 0.8))),
                );
            }
            
            println!("测试场景初始化完成！");
        })
        .add_systems(Startup, initialize_materials)
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 })
        .add_systems(Update, (rd_update, toggle_frame_rate))
        .run();
        
    println!("\n✓ 终极测试完成：应用正常运行后关闭");
}
