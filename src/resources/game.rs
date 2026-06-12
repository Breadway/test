use bevy::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

#[derive(Resource)]
pub struct GameManager {
    pub enemies_remaining: u32,
    pub total_kills: u32,
    pub rooms_cleared: u32,
    pub run_seed: u64,
    pub double_xp: bool,
    pub regen_on_kill: f32,
}

impl Default for GameManager {
    fn default() -> Self {
        Self {
            enemies_remaining: 0,
            total_kills: 0,
            rooms_cleared: 0,
            run_seed: 42,
            double_xp: false,
            regen_on_kill: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct GameRng(pub ChaCha8Rng);

impl Default for GameRng {
    fn default() -> Self {
        Self(ChaCha8Rng::seed_from_u64(42))
    }
}

#[derive(Resource, Default)]
pub struct UpgradeState {
    pub owned: Vec<crate::data::upgrades::UpgradeId>,
    pub offered: Vec<crate::data::upgrades::UpgradeId>,
}
