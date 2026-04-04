use bevy::prelude::*;

pub mod axis;
pub mod material;

// 专门处理渲染相关的系统和资源
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        // 目前只添加材质相关的系统，坐标轴由rd_setup函数创建
        // 将来可以在这里添加其他渲染相关的系统
    }
}