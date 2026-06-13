use bevy::prelude::*;
use crate::components::*;
use crate::components::player::CharacterType;
use crate::events::*;
use crate::resources::*;
use crate::state::GameState;

pub struct TorchPlugin;

impl Plugin for TorchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            torch_ignite_on_hit,
            fire_damage_tick,
            fire_spread_system,
            fire_visual_tint,
            torch_self_damage,
        ).run_if(in_state(GameState::Playing)));
    }
}

fn torch_ignite_on_hit(
    mut commands: Commands,
    mut hit_events: EventReader<ProjectileHit>,
    player_q: Query<&PlayerStats, With<Player>>,
    enemy_q: Query<(), (With<Enemy>, Without<Dead>)>,
) {
    let Ok(stats) = player_q.get_single() else { return };
    if stats.character_type != CharacterType::Torch { return }

    for ev in hit_events.read() {
        if enemy_q.contains(ev.target) {
            commands.entity(ev.target).insert((
                FireStatus::new(8.0),
                OnFire,
            ));
        }
    }
}

/// Damage-only: death detection is centralized in `detect_enemy_death`.
fn fire_damage_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Health, &mut FireStatus), (With<OnFire>, Without<Dead>)>,
) {
    let dt = time.delta_secs();
    for (entity, mut health, mut fire) in &mut q {
        fire.duration -= dt;
        health.take_damage(fire.dps * dt);

        if fire.duration <= 0.0 {
            commands.entity(entity).remove::<(FireStatus, OnFire)>();
        }
    }
}

fn fire_spread_system(
    time: Res<Time>,
    mut commands: Commands,
    grid: Res<SpatialGrid>,
    mut on_fire_q: Query<(Entity, &Transform, &mut FireStatus), (With<OnFire>, Without<Dead>)>,
    spreadable_q: Query<(), (With<Enemy>, Without<OnFire>, Without<Dead>)>,
    transform_q: Query<&Transform>,
) {
    let dt = time.delta_secs();

    let mut spread_actions: Vec<(Entity, Vec2, f32)> = Vec::new();
    for (entity, transform, mut fire) in &mut on_fire_q {
        fire.spread_cooldown_remaining -= dt;
        if fire.spread_cooldown_remaining <= 0.0 {
            fire.spread_cooldown_remaining = fire.spread_cooldown;
            spread_actions.push((entity, transform.translation.truncate(), fire.spread_radius));
        }
    }

    for (from_entity, pos, radius) in spread_actions {
        let candidates = grid.query_radius(pos, radius);
        let mut spread_count = 0;
        for candidate in candidates {
            if spread_count >= 5 { break }
            if candidate == from_entity { continue }
            if !spreadable_q.contains(candidate) { continue }
            let Ok(t) = transform_q.get(candidate) else { continue };
            let dist = pos.distance(t.translation.truncate());
            if dist <= radius {
                commands.entity(candidate).insert((
                    FireStatus::new(6.0),
                    OnFire,
                ));
                spread_count += 1;
            }
        }
    }
}

fn fire_visual_tint(
    mut q: Query<(&mut Sprite, &FireStatus), With<OnFire>>,
) {
    for (mut sprite, fire) in &mut q {
        let t = (fire.duration / 8.0).min(1.0);
        sprite.color = Color::srgb(1.0, 0.3 + t * 0.2, 0.0);
    }
}

fn torch_self_damage(
    time: Res<Time>,
    grid: Res<SpatialGrid>,
    player_q: Query<(&Transform, &PlayerStats), With<Player>>,
    transform_q: Query<&Transform, With<Enemy>>,
    on_fire_q: Query<(), With<OnFire>>,
    mut player_damage: EventWriter<PlayerDamaged>,
) {
    let Ok((player_t, stats)) = player_q.get_single() else { return };
    if stats.character_type != CharacterType::Torch { return }

    let pp = player_t.translation.truncate();
    let candidates = grid.query_radius(pp, 80.0);

    for candidate in candidates {
        if !on_fire_q.contains(candidate) { continue }
        let Ok(t) = transform_q.get(candidate) else { continue };
        if pp.distance(t.translation.truncate()) <= 80.0 {
            player_damage.send(PlayerDamaged {
                amount: 8.0 * time.delta_secs(),
                source: DamageSource::OwnFire,
            });
            return; // one self-damage tick per frame max
        }
    }
}
