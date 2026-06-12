use bevy::prelude::*;
use crate::components::enemy::EnemyType;

#[derive(Clone)]
pub struct SpawnRequest {
    pub enemy_type: EnemyType,
    pub position: Vec2,
}

#[derive(Resource, Default)]
pub struct SpawnQueue {
    pub pending: std::collections::VecDeque<SpawnRequest>,
}
