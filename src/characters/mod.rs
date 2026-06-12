pub mod doc;
pub mod heavy;
pub mod mop;
pub mod queen_b;
pub mod sparky;
pub mod torch;

use bevy::prelude::*;
use crate::components::player::CharacterType;
use crate::state::GameState;

pub struct CharactersPlugin;

impl Plugin for CharactersPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(torch::TorchPlugin)
            .add_plugins(queen_b::QueenBPlugin)
            .add_plugins(mop::MopPlugin)
            .add_plugins(heavy::HeavyPlugin)
            .add_plugins(sparky::SparkyPlugin)
            .add_plugins(doc::DocPlugin);
    }
}

pub fn is_playing_as(
    character: CharacterType,
) -> impl Fn(Query<&crate::components::player::PlayerStats, With<crate::components::player::Player>>) -> bool {
    move |q: Query<&crate::components::player::PlayerStats, With<crate::components::player::Player>>| {
        q.get_single().map(|s| s.character_type == character).unwrap_or(false)
    }
}
