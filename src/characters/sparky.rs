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

const HEAT_PER_SHOT: f32 = 12.0;
const HEAT_DECAY_PER_SEC: f32 = 20.0;

fn sparky_chain_lightning(
    mut hit_events: EventReader<ProjectileHit>,
    player_q: Query<&PlayerStats, With<Player>>,
    grid: Res<SpatialGrid>,
    mut enemy_q: Query<(&Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
) {
    let Ok(stats) = player_q.get_single() else { return };
    if stats.character_type != CharacterType::Sparky { return }

    let chain_range = 150.0;
    let chain_count = 3;

    for ev in hit_events.read() {
        let mut chain_damage = ev.damage * 1.25;
        let Ok((t, _)) = enemy_q.get(ev.target) else { continue };
        let mut last_pos = t.translation.truncate();
        let mut hit_entities = vec![ev.target];

        for _ in 0..chain_count {
            let candidates = grid.query_radius(last_pos, chain_range);
            let mut best: Option<(Entity, f32, Vec2)> = None;
            for candidate in candidates {
                if hit_entities.contains(&candidate) { continue }
                // get() fails for Dead entities (filtered out of the query)
                let Ok((t, health)) = enemy_q.get(candidate) else { continue };
                if health.is_dead() { continue }
                let cpos = t.translation.truncate();
                let d = last_pos.distance(cpos);
                if d <= chain_range && best.map_or(true, |(_, bd, _)| d < bd) {
                    best = Some((candidate, d, cpos));
                }
            }
            if let Some((next_entity, _, cpos)) = best {
                hit_entities.push(next_entity);
                if let Ok((_, mut health)) = enemy_q.get_mut(next_entity) {
                    health.take_damage(chain_damage);
                }
                last_pos = cpos;
                chain_damage *= 1.25;
            } else {
                break;
            }
        }
    }
}

/// Heat builds per actual shot fired (not per frame holding the button).
/// At 100%: forced discharge — 30% max HP self-damage and a 1s stun.
fn sparky_heat_management(
    time: Res<Time>,
    mut commands: Commands,
    mut shot_events: EventReader<ShotFired>,
    mut player_q: Query<(Entity, &mut PlayerStats, &Health), With<Player>>,
    mut player_damage: EventWriter<PlayerDamaged>,
) {
    let Ok((entity, mut stats, health)) = player_q.get_single_mut() else { return };
    if stats.character_type != CharacterType::Sparky {
        shot_events.clear();
        return;
    }

    let shots = shot_events.read().count();
    stats.heat += shots as f32 * HEAT_PER_SHOT;
    stats.heat = (stats.heat - HEAT_DECAY_PER_SEC * time.delta_secs()).max(0.0);

    if stats.heat >= stats.heat_max {
        stats.heat = 0.0;
        player_damage.send(PlayerDamaged {
            amount: health.max * 0.3,
            source: DamageSource::Electric,
        });
        commands.entity(entity).insert(Stunned(1.0));
    }
}
