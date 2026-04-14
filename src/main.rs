use bevy::prelude::*;
use redra::graph::GraphPlugin;
use redra::manager::Manager;
use redra_data_flow::DataFlowPlugin;
use redra_net::NetworkPlugin;
use smooth_bevy_cameras::LookTransformPlugin;

use redra::graph::frame_rate::FrameRateState;

/// 程序主入口函数
/// 
/// 此函数启动应用程序，初始化图形渲染系统、UI组件和网络通信
/// 使用Bevy引擎渲染图形界面，并通过NetworkPlugin处理网络逻辑
fn main() {
    // 构建并运行Bevy应用程序
    App::new()
        .add_plugins(DefaultPlugins) // 添加默认插件
        .add_plugins(GraphPlugin)
        .add_plugins(Manager::default())
        .add_plugins(DataFlowPlugin) // 添加数据流插件
        .add_plugins(LookTransformPlugin)
        .add_plugins(NetworkPlugin) // 添加网络插件
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.9))) // 设置较亮的背景色
        .insert_resource(FrameRateState { change: true, frame_rate: 60.0 }) // 添加帧率状态资源
        .run();
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
// ```