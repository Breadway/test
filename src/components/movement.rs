use bevy::prelude::*;

#[derive(Component, Default, Clone)]
pub struct Velocity(pub Vec2);


#[derive(Component, Default, Clone)]
pub struct Facing(pub Vec2);

#[derive(Component)]
pub struct Stunned(pub f32);
