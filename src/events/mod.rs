use bevy::prelude::*;

#[derive(Event, Clone)]
pub struct EnemyKilled {
    pub entity: Entity,
    pub position: Vec2,
    pub xp_value: u32,
}

#[derive(Event, Clone)]
pub struct PlayerDamaged {
    pub amount: f32,
    pub source: DamageSource,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DamageSource {
    EnemyContact,
    EnemyProjectile,
    Fire,
    Acid,
    OwnFire,
    Electric,
    OtherEnemy,
}

#[derive(Event)]
pub struct RoomCleared;

#[derive(Event)]
pub struct LevelUp {
    pub new_level: u32,
}

#[derive(Event)]
pub struct PathBranchAvailable {
    pub level: u32,
}

#[derive(Event)]
pub struct UpgradeSelected {
    pub upgrade: crate::data::upgrades::UpgradeId,
}

#[derive(Event)]
pub struct FireSpread {
    pub from: Entity,
    pub to: Entity,
}

#[derive(Event)]
pub struct ProjectileHit {
    pub projectile: Entity,
    pub target: Entity,
    pub damage: f32,
}

#[derive(Event)]
pub struct RoomTransitionRequest {
    pub destination_room: usize,
}

#[derive(Event)]
pub struct PlayerHealed {
    pub amount: f32,
}

#[derive(Event)]
pub struct BossPhaseChanged {
    pub new_phase: u8,
}

#[derive(Event)]
pub struct UpgradesOffered {
    pub upgrades: Vec<crate::data::upgrades::UpgradeId>,
}
