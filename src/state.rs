use bevy::prelude::*;

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    #[default]
    MainMenu,
    CharacterSelect,
    Playing,
    UpgradeSelect,
    MapSelect,
    GameOver,
    Victory,
}
