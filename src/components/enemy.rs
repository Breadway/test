use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub speed: f32,
    pub damage: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub attack_timer: f32,
    pub xp_value: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EnemyType {
    Roach,
    Spider,
    Wasp,
    Larvae,
    Beetle,
    Boss,
}

#[derive(Component)]
pub struct AggroRange(pub f32);

#[derive(Component)]
pub struct Flying;

#[derive(Component)]
pub struct Boss {
    pub phase: u8,
    pub summon_timer: f32,
    pub burst_timer: f32,
}

/// Larvae evolve into Roaches when this expires.
#[derive(Component)]
pub struct EvolveTimer(pub f32);

#[derive(Component)]
pub struct BeeAlly {
    pub lifetime: f32,
    pub attack_timer: f32,
}
