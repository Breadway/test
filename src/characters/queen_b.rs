use bevy::prelude::*;
use crate::components::*;
use crate::components::player::CharacterType;
use crate::events::*;
use crate::state::GameState;

pub struct QueenBPlugin;

impl Plugin for QueenBPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            queen_b_summon_bee,
            apply_bee_velocity,
        ).run_if(in_state(GameState::Playing)));
    }
}

fn queen_b_summon_bee(
    mut commands: Commands,
    mut kill_events: EventReader<EnemyKilled>,
    player_q: Query<&PlayerStats, With<Player>>,
) {
    let Ok(stats) = player_q.get_single() else { return };
    if stats.character_type != CharacterType::QueenB { return }

    let current_bees = stats.bee_count;
    let max_bees = stats.bee_max;
    if current_bees >= max_bees { return }

    for ev in kill_events.read() {
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.85, 0.0),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
            Transform::from_xyz(ev.position.x, ev.position.y, 2.0),
            Enemy {
                enemy_type: crate::components::enemy::EnemyType::Roach, // reuse for ally
                speed: 200.0,
                damage: 8.0,
                attack_range: 20.0,
                attack_cooldown: 0.8,
                attack_timer: 0.0,
                xp_value: 0,
            },
            Health::new(1.0),
            Velocity::default(),
            AIState::default(),
            AggroRange(250.0),
            XpValue(0),
            Ally,
            BeeAlly { lifetime: 30.0 },
        ));
    }
}

fn apply_bee_velocity(
    time: Res<Time>,
    mut bee_q: Query<(&Velocity, &mut Transform), With<BeeAlly>>,
) {
    let dt = time.delta_secs();
    for (vel, mut transform) in &mut bee_q {
        transform.translation.x += vel.0.x * dt;
        transform.translation.y += vel.0.y * dt;
    }
}
