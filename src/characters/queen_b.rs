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
            bee_ai,
            bee_attack,
            apply_bee_velocity,
            bee_lifetime,
        ).run_if(in_state(GameState::Playing)));
    }
}

const BEE_DAMAGE: f32 = 8.0;
const BEE_ATTACK_COOLDOWN: f32 = 0.8;
const BEE_LIFETIME: f32 = 30.0;

/// Bees are pure allies: no Enemy component, so enemy AI, contact damage,
/// fire spread, and player projectiles all ignore them.
fn queen_b_summon_bee(
    mut commands: Commands,
    mut kill_events: EventReader<EnemyKilled>,
    mut player_q: Query<&mut PlayerStats, With<Player>>,
) {
    let Ok(mut stats) = player_q.get_single_mut() else { return };
    if stats.character_type != CharacterType::QueenB { return }

    for ev in kill_events.read() {
        if stats.bee_count >= stats.bee_max { break }
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.85, 0.0),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
            Transform::from_xyz(ev.position.x, ev.position.y, 2.0),
            Velocity::default(),
            BeeAlly { lifetime: BEE_LIFETIME, attack_timer: 0.0 },
        ));
        stats.bee_count += 1;
    }
}

fn bee_ai(
    player_q: Query<&Transform, (With<Player>, Without<BeeAlly>)>,
    mut bee_q: Query<(&mut Velocity, &Transform), With<BeeAlly>>,
    enemy_q: Query<&Transform, (With<Enemy>, Without<Dead>, Without<BeeAlly>, Without<Player>)>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let player_pos = player_t.translation.truncate();

    for (mut vel, bee_t) in &mut bee_q {
        let bee_pos = bee_t.translation.truncate();
        let mut closest_dist = f32::MAX;
        let mut closest_target = None;

        for enemy_t in &enemy_q {
            let ep = enemy_t.translation.truncate();
            let d = bee_pos.distance(ep);
            if d < closest_dist && d < 300.0 {
                closest_dist = d;
                closest_target = Some(ep);
            }
        }

        if let Some(target) = closest_target {
            let dir = (target - bee_pos).normalize();
            vel.0 = dir * 200.0;
        } else {
            let to_player = player_pos - bee_pos;
            if to_player.length() > 40.0 {
                vel.0 = to_player.normalize() * 160.0;
            } else {
                vel.0 *= 0.9;
            }
        }
    }
}

fn bee_attack(
    time: Res<Time>,
    mut bee_q: Query<(&Transform, &mut BeeAlly)>,
    mut enemy_q: Query<(&Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
) {
    let dt = time.delta_secs();
    for (bee_t, mut bee) in &mut bee_q {
        bee.attack_timer -= dt;
        if bee.attack_timer > 0.0 { continue }
        let bee_pos = bee_t.translation.truncate();

        for (enemy_t, mut health) in &mut enemy_q {
            if bee_pos.distance(enemy_t.translation.truncate()) < 22.0 {
                health.take_damage(BEE_DAMAGE);
                bee.attack_timer = BEE_ATTACK_COOLDOWN;
                break;
            }
        }
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

fn bee_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut bee_q: Query<(Entity, &mut BeeAlly)>,
    mut player_q: Query<&mut PlayerStats, With<Player>>,
) {
    let dt = time.delta_secs();
    for (entity, mut bee) in &mut bee_q {
        bee.lifetime -= dt;
        if bee.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
            if let Ok(mut stats) = player_q.get_single_mut() {
                stats.bee_count = stats.bee_count.saturating_sub(1);
            }
        }
    }
}
