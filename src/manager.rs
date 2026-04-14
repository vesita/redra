use bevy::app::prelude::*;

pub mod system;
pub mod font;
pub mod data_processing;

#[derive(Default)]
pub struct Manager {
}

impl Plugin for Manager {
    fn build(&self, app: &mut App) {
        app.add_plugins(font::FontPlugin);
    }
}