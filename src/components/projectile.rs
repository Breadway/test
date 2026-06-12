use bevy::prelude::*;

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub owner_is_player: bool,
    pub lifetime: f32,
    pub pierce: u32,
    pub damage_multiplier: f32,
}

#[derive(Component)]
pub struct EnemyProjectile {
    pub damage: f32,
}

#[derive(Component)]
pub struct WebProjectile {
    pub slow_factor: f32,
    pub slow_duration: f32,
}

#[derive(Component)]
pub struct SlowStatus {
    pub factor: f32,
    pub remaining: f32,
}
