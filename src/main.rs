use bevy::prelude::*;
use redra::graph::communicate::channels::RDChannel;
use redra::graph::init::material::initialize_materials;
use redra::graph::GraphPlugin;
use redra::manager::Manager;
use redra::{
    graph::{setup::rd_setup, update::rd_update},
    module::parser::core::RDPack,
    net::listener::RDListener,
};
use smooth_bevy_cameras::LookTransformPlugin;
use smooth_bevy_cameras::controllers::fps::FpsCameraPlugin;
use tokio::sync::{broadcast, mpsc};

use log::info;

use redra::render::frame::{toggle_frame_rate, FrameRateState};

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
        .add_plugins(Manager::default())
        .add_plugins(FpsCameraPlugin::default()) // 添加FPS相机插件
        .add_plugins(LookTransformPlugin) // 添加相机变换插件
        .add_plugins(GraphPlugin) // 使用 GraphPlugin 替代 UiModule
        .insert_resource(channel) // 插入通信通道资源
        .add_systems(Startup, rd_setup) // 添加rd_setup系统
        .add_systems(Startup, initialize_materials) // 添加initialize_materials系统
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 }) // 添加帧率状态资源
        .add_systems(Update, (rd_update, toggle_frame_rate)) // 添加更新系统和帧率切换系统
        .run();
    Ok(())
}


// # Redra 项目启动指南
// 
// ## 重要提示：阅读项目规则
// 
// **AI Agent 请注意：** 在修改此项目代码前，请务必阅读 `prompt/` 目录下的规则文件！
// 
// ### 规则文件位置
// - **全局规则**: `prompt/rule.yaml` - 包含项目整体规范
// - **子规则目录**: `prompt/rules/` - 包含各类专项规则
//   - `git_commit.yaml` - Git 提交规范
//   - `normal_language.yaml` - 代码语言使用规范
//   - `preference.yaml` - 编码偏好设置
//   - `tool_chain.yaml` - 工具链使用规范
// 
// ### 为什么要阅读规则？
// 这些规则定义了项目的代码风格、架构设计、命名约定等重要规范.
// 遵循这些规则可以确保代码一致性和项目质量.
// 
// ---
// 