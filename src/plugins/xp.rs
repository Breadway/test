use bevy::prelude::*;
use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::state::GameState;

pub struct XPPlugin;

impl Plugin for XPPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(XpManager::new())
            .add_systems(Update, (
                collect_xp,
                regen_on_kill,
            ).run_if(in_state(GameState::Playing)));
    }
}

fn collect_xp(
    mut kill_events: EventReader<EnemyKilled>,
    mut level_up: EventWriter<LevelUp>,
    mut path_branch: EventWriter<PathBranchAvailable>,
    mut xp_mgr: ResMut<XpManager>,
    mut health_q: Query<&mut Health, With<Player>>,
    game_manager: Res<GameManager>,
) {
    for ev in kill_events.read() {
        let gain = if game_manager.double_xp { ev.xp_value * 2 } else { ev.xp_value };
        xp_mgr.current_xp += gain;
        xp_mgr.xp_this_level += gain;

        // Level up loop (can multi-level in one tick)
        while xp_mgr.xp_this_level >= xp_mgr.xp_to_next {
            xp_mgr.xp_this_level -= xp_mgr.xp_to_next;
            xp_mgr.current_level += 1;
            let new_level = xp_mgr.current_level;
            xp_mgr.xp_to_next = xp_for_level(new_level);

            // Heal on level up
            if let Ok(mut health) = health_q.get_single_mut() {
                let heal = health.max * 0.10;
                health.heal(heal);
            }

            level_up.send(LevelUp { new_level });

            if new_level == 10 || new_level == 20 {
                path_branch.send(PathBranchAvailable { level: new_level });
            }
        }
    }
}

fn regen_on_kill(
    mut kill_events: EventReader<EnemyKilled>,
    mut health_q: Query<&mut Health, With<Player>>,
    game_manager: Res<GameManager>,
) {
    if game_manager.regen_on_kill <= 0.0 { return }
    let count = kill_events.read().count();
    if count == 0 { return }
    if let Ok(mut health) = health_q.get_single_mut() {
        health.heal(game_manager.regen_on_kill * count as f32);
    }
}
