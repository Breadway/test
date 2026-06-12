pub mod doc;
pub mod heavy;
pub mod mop;
pub mod queen_b;
pub mod sparky;
pub mod torch;

use bevy::prelude::*;

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
