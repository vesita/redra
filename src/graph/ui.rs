pub mod clear;
pub mod panel;
pub mod font;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use panel::PanelPlugin;
use crate::graph::clear::ClearAllEvent;  // 导入清除事件
use crate::graph::ui::font::replace_fonts;

pub struct UiModule;

impl Plugin for UiModule {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(PanelPlugin)
            .add_event::<ClearAllEvent>()  // 添加清除事件
            .add_systems(Startup, replace_fonts);
    }
}