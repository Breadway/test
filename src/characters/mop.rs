use bevy::prelude::*;
use crate::components::*;
use crate::components::player::CharacterType;
use crate::events::*;
use crate::state::GameState;

pub struct MopPlugin;

impl Plugin for MopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,
            mop_range_damage_modifier
            .run_if(in_state(GameState::Playing)));
    }
}

fn mop_range_damage_modifier(
    mut hit_events: EventReader<ProjectileHit>,
    player_q: Query<(&Transform, &PlayerStats), With<Player>>,
    mut enemy_q: Query<(&Transform, &mut Health), With<Enemy>>,
) {
    let Ok((player_t, stats)) = player_q.get_single() else { return };
    if stats.character_type != CharacterType::Mop { return }

    let player_pos = player_t.translation.truncate();
    let max_range = stats.projectile_range;
    let min_range = max_range * 0.3;

    for ev in hit_events.read() {
        if let Ok((enemy_t, mut health)) = enemy_q.get_mut(ev.target) {
            let dist = player_pos.distance(enemy_t.translation.truncate());
            if dist < min_range {
                // Undo the damage (already applied in combat system)
                health.heal(ev.damage);
            } else if dist > max_range * 0.75 {
                // Bonus damage for max range
                health.take_damage(ev.damage * 2.0);
            }
        }
    }
}
