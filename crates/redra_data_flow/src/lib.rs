use bevy::prelude::*;

use redra_parser::core::RDPack;
use tokio::sync::mpsc;

// 数据流插件，负责管理数据流动和处理
pub struct DataFlowPlugin;

impl Plugin for DataFlowPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DataFlowState>()
            .add_systems(Update, handle_data_flow);
    }
}

// 数据流状态资源
#[derive(Resource, Default)]
pub struct DataFlowState {
    // 可以在这里添加数据流处理的状态
}

// 处理数据流的主要系统
fn handle_data_flow(
    mut commands: Commands,
    mut rd_channel: ResMut<redra_net::RDChannel>,
) {
    // 这个系统将在主循环中运行，处理来自网络的数据
    // 监听RDChannel并处理接收到的RDPack
    // 暂时只是预留接口，实际的数据处理在spawn系统中完成
}

// 处理模式枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingMode {
    Render,   // 实时渲染
    Record,   // 录制
    Replay,  // 回放
}

// 数据分发器资源
#[derive(Resource)]
pub struct DataDispatcher {
    pub current_mode: ProcessingMode,
    pub render_sender: Option<mpsc::UnboundedSender<RDPack>>,
    pub record_sender: Option<mpsc::UnboundedSender<RDPack>>,
    pub replay_sender: Option<mpsc::UnboundedSender<RDPack>>,
}

impl Default for DataDispatcher {
    fn default() -> Self {
        Self {
            current_mode: ProcessingMode::Render,
            render_sender: None,
            record_sender: None,
            replay_sender: None,
        }
    }
}

impl DataDispatcher {
    pub fn new(mode: ProcessingMode) -> Self {
        Self {
            current_mode: mode,
            ..Default::default()
        }
    }

    // 分发数据包到相应的目的地
    pub fn dispatch(&self, pack: RDPack) {
        match self.current_mode {
            ProcessingMode::Render => {
                if let Some(ref sender) = self.render_sender {
                    let _ = sender.send(pack);
                }
            }
            ProcessingMode::Record => {
                if let Some(ref sender) = self.record_sender {
                    let _ = sender.send(pack);
                }
            }
            ProcessingMode::Replay => {
                if let Some(ref sender) = self.replay_sender {
                    let _ = sender.send(pack);
                }
            }
        }
    }

    // 切换处理模式
    pub fn set_mode(&mut self, mode: ProcessingMode) {
        self.current_mode = mode;
    }
}
