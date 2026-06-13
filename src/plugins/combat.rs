use bevy::prelude::*;
use crate::components::*;
use crate::data::pixel_art::GameSprites;
use crate::events::*;
use crate::plugins::visual::{EnemyFlash, spawn_sparks};
use crate::resources::*;
use crate::state::GameState;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                move_projectiles,
                projectile_enemy_collision,
                projectile_lifetime,
                enemy_projectile_player_collision,
                web_projectile_player_collision,
                detect_enemy_death,
                enemy_death_system,
                rebuild_spatial_grid,
            ).chain().run_if(in_state(GameState::Playing)));
    }
}

fn rebuild_spatial_grid(
    mut grid: ResMut<SpatialGrid>,
    q: Query<(Entity, &Transform), (With<Enemy>, Without<Dead>)>,
) {
    grid.clear();
    for (entity, transform) in &q {
        grid.insert(entity, transform.translation.truncate());
    }
}

fn move_projectiles(
    time: Res<Time>,
    mut q: Query<(&Velocity, &mut Transform), Or<(With<Projectile>, With<EnemyProjectile>, With<WebProjectile>)>>,
) {
    let dt = time.delta_secs();
    for (vel, mut transform) in &mut q {
        transform.translation.x += vel.0.x * dt;
        transform.translation.y += vel.0.y * dt;
    }
}

/// Damage-only: death detection is centralized in `detect_enemy_death`.
fn projectile_enemy_collision(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    mut events: EventWriter<ProjectileHit>,
    mut projectile_q: Query<(Entity, &Transform, &mut Projectile)>,
    mut enemy_q: Query<(Entity, &Transform, &mut Health, &Sprite), (With<Enemy>, Without<Dead>)>,
) {
    for (proj_entity, proj_t, mut proj) in &mut projectile_q {
        let pp = proj_t.translation.truncate();
        let proj_size = 8.0f32;

        for (enemy_entity, enemy_t, mut health, enemy_sprite) in &mut enemy_q {
            if health.is_dead() { continue }
            if proj.hit_entities.contains(&enemy_entity) { continue }
            let ep = enemy_t.translation.truncate();

            if pp.distance(ep) < proj_size + 18.0 {
                let damage = proj.damage * proj.damage_multiplier;
                health.take_damage(damage);
                proj.hit_entities.push(enemy_entity);

                // Hit flash — store original color before overwriting
                commands.entity(enemy_entity).insert(EnemyFlash {
                    timer: 0.12,
                    original_color: enemy_sprite.color,
                });

                // Impact sparks at hit point
                let hit_pos = (pp + ep) * 0.5;
                spawn_sparks(&mut commands, &sprites, hit_pos, Color::srgb(1.0, 0.85, 0.2));

                events.send(ProjectileHit {
                    target: enemy_entity,
                    damage,
                });

                if proj.pierce == 0 {
                    commands.entity(proj_entity).despawn_recursive();
                    break;
                } else {
                    proj.pierce -= 1;
                }
            }
        }
    }
}

/// The single source of EnemyKilled events. Runs after all damage systems;
/// the Dead marker guarantees at most one kill event per entity.
pub fn detect_enemy_death(
    mut commands: Commands,
    mut kill_events: EventWriter<EnemyKilled>,
    q: Query<(Entity, &Health, &Transform, &XpValue), (With<Enemy>, Without<Dead>)>,
) {
    for (entity, health, transform, xp_val) in &q {
        if health.is_dead() {
            commands.entity(entity).insert(Dead);
            kill_events.send(EnemyKilled {
                entity,
                position: transform.translation.truncate(),
                xp_value: xp_val.0,
            });
        }
    }
}

pub fn enemy_death_system(
    mut commands: Commands,
    mut kill_events: EventReader<EnemyKilled>,
    mut game_manager: ResMut<GameManager>,
) {
    for ev in kill_events.read() {
        if let Some(entity_commands) = commands.get_entity(ev.entity) {
            entity_commands.despawn_recursive();
        }
        game_manager.enemies_remaining = game_manager.enemies_remaining.saturating_sub(1);
        game_manager.total_kills += 1;
    }
}

fn projectile_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Projectile)>,
) {
    let dt = time.delta_secs();
    for (entity, mut proj) in &mut q {
        proj.lifetime -= dt;
        if proj.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn enemy_projectile_player_collision(
    mut commands: Commands,
    mut player_damage: EventWriter<PlayerDamaged>,
    player_q: Query<&Transform, With<Player>>,
    proj_q: Query<(Entity, &Transform, &EnemyProjectile)>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let pp = player_t.translation.truncate();

    for (proj_e, proj_t, proj) in &proj_q {
        let ep = proj_t.translation.truncate();
        if pp.distance(ep) < 22.0 {
            player_damage.send(PlayerDamaged {
                amount: proj.damage,
                source: DamageSource::EnemyProjectile,
            });
            commands.entity(proj_e).despawn_recursive();
        }
    }
}

fn web_projectile_player_collision(
    mut commands: Commands,
    player_q: Query<(Entity, &Transform), With<Player>>,
    proj_q: Query<(Entity, &Transform, &WebProjectile)>,
) {
    let Ok((player_e, player_t)) = player_q.get_single() else { return };
    let pp = player_t.translation.truncate();

    for (proj_e, proj_t, web) in &proj_q {
        let ep = proj_t.translation.truncate();
        if pp.distance(ep) < 20.0 {
            commands.entity(player_e).insert(SlowStatus {
                factor: web.slow_factor,
                remaining: web.slow_duration,
            });
            commands.entity(proj_e).despawn_recursive();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::enemy::EnemyType;

    /// An entity reduced to 0 HP by multiple sources in one frame must
    /// produce exactly one EnemyKilled event across all frames.
    #[test]
    fn one_kill_event_per_enemy() {
        let mut app = App::new();
        app.add_event::<EnemyKilled>();
        app.insert_resource(GameManager::default());
        app.add_systems(Update, (detect_enemy_death, enemy_death_system).chain());

        let enemy = app.world_mut().spawn((
            Enemy {
                enemy_type: EnemyType::Roach,
                speed: 0.0, damage: 0.0, attack_range: 0.0,
                attack_cooldown: 0.0, attack_timer: 0.0, xp_value: 5,
            },
            Health { current: 0.0, max: 30.0 },
            Transform::default(),
            XpValue(5),
        )).id();

        // Run several frames; despawn is deferred so a buggy detect could re-fire
        for _ in 0..3 {
            app.update();
        }

        let gm = app.world().resource::<GameManager>();
        assert_eq!(gm.total_kills, 1, "exactly one kill must be counted");
        assert!(app.world().get_entity(enemy).is_err(), "enemy must be despawned");
    }
}
