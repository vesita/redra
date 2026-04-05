use bevy::prelude::*;

pub mod axis;

// 专门处理渲染相关的系统和资源
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(axis::AxisRenderingPlugin); // 添加坐标轴渲染插件
    }
}