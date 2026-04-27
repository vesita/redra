//! 统一字体管理模块
//! 负责加载和管理所有字体资源（Bevy 原生 + egui）

use std::sync::Arc;
use bevy::prelude::*;
use bevy_egui::EguiContexts;

#[derive(Resource)]
pub struct FontAssets {
    pub bevy_font: Handle<bevy::text::Font>,
}

impl Default for FontAssets {
    fn default() -> Self { Self { bevy_font: Handle::default() } }
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub enum FontLoadStatus {
    Loading,
    Loaded,
}

impl Default for FontLoadStatus {
    fn default() -> Self { FontLoadStatus::Loading }
}

/// 字体插件
pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FontAssets>()
            .init_resource::<FontLoadStatus>()
            .add_systems(Startup, load_all_fonts)
            .add_systems(Update, (setup_egui_fonts, update_font_load_status));
    }
}

fn load_all_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("正在加载字体资源...");
    let bevy_font = asset_server.load("fonts/serif/SourceHanSerifCN-VF.otf");
    commands.insert_resource(FontAssets { bevy_font });
    info!("字体资源加载完成");
}

pub fn setup_egui_fonts(mut contexts: EguiContexts, font_assets: Option<Res<FontAssets>>) {
    if font_assets.is_some() {
        let mut fonts = bevy_egui::egui::FontDefinitions::default();
        fonts.font_data.insert(
            "main_font".to_owned(),
            Arc::new(bevy_egui::egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/serif/SourceHanSerifCN-VF.otf"
            ))),
        );
        fonts.families.entry(bevy_egui::egui::FontFamily::Proportional).or_default().insert(0, "main_font".to_owned());
        fonts.families.entry(bevy_egui::egui::FontFamily::Monospace).or_default().push("main_font".to_owned());
        if let Ok(ctx) = contexts.ctx_mut() {
            ctx.set_fonts(fonts);
        }
    }
}

fn update_font_load_status(
    font_assets: Option<Res<FontAssets>>,
    mut font_status: ResMut<FontLoadStatus>,
    asset_server: Res<AssetServer>,
) {
    if let Some(assets) = font_assets {
        match asset_server.get_load_state(&assets.bevy_font) {
            Some(bevy::asset::LoadState::Loaded) => {
                if *font_status != FontLoadStatus::Loaded {
                    *font_status = FontLoadStatus::Loaded;
                    info!("字体加载状态已更新为：已加载");
                }
            }
            Some(bevy::asset::LoadState::Failed(_)) => warn!("字体加载失败"),
            _ => {}
        }
    }
}

pub fn get_font_handle(font_assets: &Res<FontAssets>) -> Handle<bevy::text::Font> {
    font_assets.bevy_font.clone()
}
