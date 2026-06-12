use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct FireStatus {
    pub duration: f32,
    pub dps: f32,
    pub spread_radius: f32,
    pub spread_cooldown: f32,
    pub spread_cooldown_remaining: f32,
}

#[derive(Component)]
pub struct OnFire;

impl FireStatus {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            dps: 5.0,
            spread_radius: 80.0,
            spread_cooldown: 0.5,
            spread_cooldown_remaining: 0.0,
        }
    }
}
