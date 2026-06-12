use bevy::prelude::*;

#[derive(Component)]
pub struct XpValue(pub u32);

#[derive(Resource, Default)]
pub struct XpManager {
    pub current_xp: u32,
    pub current_level: u32,
    pub xp_this_level: u32,
    pub xp_to_next: u32,
    pub level_path: Option<LevelPath>,
    pub sub_path: Option<SubPath>,
}

impl XpManager {
    pub fn new() -> Self {
        Self {
            current_xp: 0,
            current_level: 1,
            xp_this_level: 0,
            xp_to_next: xp_for_level(1),
            level_path: None,
            sub_path: None,
        }
    }
}

pub fn xp_for_level(level: u32) -> u32 {
    50 + level * 40 + level * level * 10
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LevelPath {
    Aggressive,
    Defensive,
    Utility,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SubPath {
    AggressiveA,
    AggressiveB,
    DefensiveA,
    DefensiveB,
    UtilityA,
    UtilityB,
}
