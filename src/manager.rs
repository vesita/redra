use bevy::app::prelude::*;

pub mod font;

#[derive(Default)]
pub struct Manager {
}

impl Plugin for Manager {
    fn build(&self, app: &mut App) {
        app.add_plugins(font::FontPlugin);
    }
}