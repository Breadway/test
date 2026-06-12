use bevy::prelude::*;
use crate::components::*;
use crate::components::player::CharacterType;
use crate::events::*;
use crate::resources::*;
use crate::state::GameState;

pub struct SparkyPlugin;

impl Plugin for SparkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            sparky_chain_lightning,
            sparky_heat_management,
        ).run_if(in_state(GameState::Playing)));
    }
}

fn sparky_chain_lightning(
    mut hit_events: EventReader<ProjectileHit>,
    player_q: Query<&PlayerStats, With<Player>>,
    grid: Res<SpatialGrid>,
    mut health_q: Query<(&mut Health, &Transform), With<Enemy>>,
    mut kill_events: EventWriter<EnemyKilled>,
    xp_q: Query<&XpValue, With<Enemy>>,
    transform_q: Query<&Transform, With<Enemy>>,
) {
    let Ok(stats) = player_q.get_single() else { return };
    if stats.character_type != CharacterType::Sparky { return }

    let chain_range = 150.0;
    let chain_count = 3;

    for ev in hit_events.read() {
        let mut chain_damage = ev.damage * 1.25;
        let mut last_pos = {
            let Ok((_, t)) = health_q.get(ev.target) else { continue };
            t.translation.truncate()
        };
        let mut hit_entities = vec![ev.target];

        for _ in 0..chain_count {
            let candidates = grid.query_radius(last_pos, chain_range);
            let mut best: Option<(Entity, f32)> = None;
            for candidate in candidates {
                if hit_entities.contains(&candidate) { continue }
                if let Ok(t) = transform_q.get(candidate) {
                    let d = last_pos.distance(t.translation.truncate());
                    if best.is_none() || d < best.unwrap().1 {
                        best = Some((candidate, d));
                    }
                }
            }
            if let Some((next_entity, _)) = best {
                hit_entities.push(next_entity);
                if let Ok((mut health, t)) = health_q.get_mut(next_entity) {
                    health.take_damage(chain_damage);
                    if health.is_dead() {
                        let xp = xp_q.get(next_entity).map(|x| x.0).unwrap_or(0);
                        kill_events.send(EnemyKilled {
                            entity: next_entity,
                            position: t.translation.truncate(),
                            xp_value: xp,
                        });
                    }
                    last_pos = t.translation.truncate();
                }
                chain_damage *= 1.25;
            } else {
                break;
            }
        }
    }
}

fn sparky_heat_management(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut player_q: Query<(&mut PlayerStats, &mut Health), With<Player>>,
    mut player_damage: EventWriter<PlayerDamaged>,
) {
    let Ok((mut stats, health)) = player_q.get_single_mut() else { return };
    if stats.character_type != CharacterType::Sparky { return }

    if buttons.pressed(MouseButton::Left) {
        stats.heat += 12.0 * time.delta_secs();
    } else {
        stats.heat = (stats.heat - 20.0 * time.delta_secs()).max(0.0);
    }

    if stats.heat >= stats.heat_max {
        stats.heat = 0.0;
        player_damage.send(PlayerDamaged {
            amount: health.max * 0.3,
            source: DamageSource::Electric,
        });
    }
}
