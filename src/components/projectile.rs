use bevy::prelude::*;

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub lifetime: f32,
    pub pierce: u32,
    pub damage_multiplier: f32,
    /// Enemies already struck — piercing projectiles overlap a target for
    /// several frames and must not hit it once per frame.
    pub hit_entities: Vec<Entity>,
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
