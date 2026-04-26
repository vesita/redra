use bevy::prelude::*;
use redra::RedraPlugin;
use smooth_bevy_cameras::LookTransformPlugin;

use redra::renderer::frame_rate::FrameRateState;

/// 程序主入口函数
/// 
/// 此函数启动应用程序,初始化图形渲染系统、UI组件和网络通信
/// 使用Bevy引擎渲染图形界面,并通过NetworkPlugin处理网络逻辑
fn main() {
    // 构建并运行Bevy应用程序
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(RedraPlugin) // 使用 RedraPlugin 替代单独的插件（内部已包含拾取功能）
        .add_plugins(LookTransformPlugin)
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.9))) // 设置较亮的背景色
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 }) // 添加帧率状态资源
        .run();
}
