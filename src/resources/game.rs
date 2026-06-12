use bevy::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Resource)]
pub struct GameManager {
    pub enemies_remaining: u32,
    pub total_kills: u32,
    pub rooms_cleared: u32,
    pub double_xp: bool,
    pub regen_on_kill: f32,
    pub discovered_docs: Vec<usize>,
}

impl Default for GameManager {
    fn default() -> Self {
        Self {
            enemies_remaining: 0,
            total_kills: 0,
            rooms_cleared: 0,
            double_xp: false,
            regen_on_kill: 0.0,
            discovered_docs: Vec::new(),
        }
    }
}

#[derive(Resource)]
pub struct GameRng(pub ChaCha8Rng);

impl GameRng {
    pub fn from_entropy() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::from_entropy()
    }
}

#[derive(Resource, Default)]
pub struct UpgradeState {
    pub owned: Vec<crate::data::upgrades::UpgradeId>,
}
