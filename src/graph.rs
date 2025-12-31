pub mod setup;
pub mod update;
pub mod spawn;
pub mod axis;
pub mod init;
pub mod communicate;
pub mod ui;

// 导入材质模块
pub mod material;
pub use material::MaterialManager;

use bevy::prelude::*;
use setup::*;
use ui::*;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(UiModule)
            .add_systems(Startup, rd_setup)
            .add_systems(Update, crate::graph::update::rd_update);
    }
}