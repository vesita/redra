use bevy::app::prelude::*;

pub mod font;
pub mod coding;
pub mod record;
pub mod parser_manager;

#[derive(Default)]
pub struct Manager {
}

impl Plugin for Manager {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(redra_net::NetworkPlugin)
            .add_plugins(font::FontPlugin)
            .add_plugins(parser_manager::ParserManagerPlugin)
            .add_plugins(record::RecorderPlugin)
            .add_plugins(record::PlayerPlugin);
    }
}
