use bevy::prelude::*;

#[derive(Component, Default, Clone)]
pub struct Velocity(pub Vec2);

#[derive(Component, Clone)]
pub struct Speed(pub f32);

impl Default for Speed {
    fn default() -> Self {
        Self(150.0)
    }
}

#[derive(Component, Default, Clone)]
pub struct Facing(pub Vec2);

#[derive(Component)]
pub struct Collider {
    pub half_extents: Vec2,
}

#[derive(Component)]
pub struct Wall;
