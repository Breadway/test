use bevy::prelude::*;

pub struct EnemyConfig {
    pub max_health: f32,
    pub move_speed: f32,
    pub damage: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub xp_value: u32,
    pub size: Vec2,
    pub color: Color,
    pub aggro_range: f32,
}

pub const ROACH: EnemyConfig = EnemyConfig {
    max_health: 30.0,
    move_speed: 120.0,
    damage: 8.0,
    attack_range: 28.0,
    attack_cooldown: 1.0,
    xp_value: 5,
    size: Vec2::new(24.0, 16.0),
    color: Color::srgb(0.42, 0.25, 0.08),
    aggro_range: 600.0,
};

pub const SPIDER: EnemyConfig = EnemyConfig {
    max_health: 45.0,
    move_speed: 85.0,
    damage: 15.0,
    attack_range: 220.0,
    attack_cooldown: 2.5,
    xp_value: 12,
    size: Vec2::new(22.0, 22.0),
    color: Color::srgb(0.25, 0.05, 0.35),
    aggro_range: 450.0,
};

pub const WASP: EnemyConfig = EnemyConfig {
    max_health: 25.0,
    move_speed: 160.0,
    damage: 12.0,
    attack_range: 280.0,
    attack_cooldown: 1.8,
    xp_value: 15,
    size: Vec2::new(18.0, 18.0),
    color: Color::srgb(0.95, 0.8, 0.1),
    aggro_range: 700.0,
};

pub const LARVAE: EnemyConfig = EnemyConfig {
    max_health: 15.0,
    move_speed: 70.0,
    damage: 5.0,
    attack_range: 28.0,
    attack_cooldown: 0.8,
    xp_value: 8,
    size: Vec2::new(20.0, 14.0),
    color: Color::srgb(0.7, 0.85, 0.3),
    aggro_range: 500.0,
};

pub const BEETLE: EnemyConfig = EnemyConfig {
    max_health: 120.0,
    move_speed: 60.0,
    damage: 20.0,
    attack_range: 30.0,
    attack_cooldown: 2.0,
    xp_value: 25,
    size: Vec2::new(30.0, 26.0),
    color: Color::srgb(0.1, 0.2, 0.5),
    aggro_range: 400.0,
};

pub const BOSS: EnemyConfig = EnemyConfig {
    max_health: 600.0,
    move_speed: 90.0,
    damage: 25.0,
    attack_range: 60.0,
    attack_cooldown: 1.5,
    xp_value: 200,
    size: Vec2::new(64.0, 64.0),
    color: Color::srgb(0.6, 0.0, 0.0),
    aggro_range: 1000.0,
};

#[derive(Clone, Debug)]
pub struct WaveConfig {
    pub spawns: Vec<(crate::components::enemy::EnemyType, u32)>,
    pub spawn_delay: f32,
}

pub fn room_waves(room_index: usize) -> Vec<WaveConfig> {
    let scale = 1.0 + room_index as f32 * 0.12;
    let roach_count = ((8.0 + room_index as f32 * 3.5) * scale).min(80.0) as u32;

    use crate::components::enemy::EnemyType::*;

    match room_index {
        0 => vec![
            WaveConfig { spawns: vec![(Roach, 10)], spawn_delay: 0.2 },
        ],
        1 => vec![
            WaveConfig { spawns: vec![(Roach, 15)], spawn_delay: 0.15 },
        ],
        2 => vec![
            WaveConfig { spawns: vec![(Roach, 12), (Spider, 2)], spawn_delay: 0.15 },
        ],
        3 => vec![
            WaveConfig { spawns: vec![(Roach, 15), (Spider, 3)], spawn_delay: 0.12 },
        ],
        4 => vec![
            WaveConfig { spawns: vec![(Roach, 10), (Wasp, 5)], spawn_delay: 0.15 },
        ],
        5..=9 => vec![
            WaveConfig { spawns: vec![(Roach, roach_count), (Spider, 3 + room_index as u32 / 2), (Wasp, room_index as u32 / 2)], spawn_delay: 0.1 },
        ],
        10 => vec![
            WaveConfig { spawns: vec![(Roach, 30), (Spider, 8), (Wasp, 8), (Beetle, 3)], spawn_delay: 0.08 },
        ],
        11 => vec![
            WaveConfig { spawns: vec![(Boss, 1)], spawn_delay: 1.0 },
        ],
        12..=20 => vec![
            WaveConfig { spawns: vec![(Roach, roach_count.min(60)), (Spider, 5 + room_index as u32 / 3), (Wasp, 4 + room_index as u32 / 3), (Beetle, 2 + room_index as u32 / 5)], spawn_delay: 0.08 },
        ],
        21..=24 => vec![
            WaveConfig { spawns: vec![(Roach, 50), (Spider, 10), (Wasp, 10), (Beetle, 8), (Larvae, 15)], spawn_delay: 0.06 },
        ],
        _ => vec![
            WaveConfig { spawns: vec![(Boss, 1)], spawn_delay: 1.0 },
        ],
    }
}
