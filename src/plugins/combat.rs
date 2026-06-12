use bevy::prelude::*;
use crate::components::*;
use crate::events::*;
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
                enemy_death_system,
                rebuild_spatial_grid,
            ).chain().run_if(in_state(GameState::Playing)));
    }
}

fn rebuild_spatial_grid(
    mut grid: ResMut<SpatialGrid>,
    q: Query<(Entity, &Transform), With<Enemy>>,
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

fn projectile_enemy_collision(
    mut commands: Commands,
    mut events: EventWriter<ProjectileHit>,
    mut kill_events: EventWriter<EnemyKilled>,
    mut projectile_q: Query<(Entity, &Transform, &mut Projectile)>,
    mut enemy_q: Query<(Entity, &Transform, &mut Health, &Enemy, &XpValue)>,
    game_manager: Res<GameManager>,
) {
    for (proj_entity, proj_t, mut proj) in &mut projectile_q {
        let pp = proj_t.translation.truncate();
        let proj_size = 8.0f32;

        for (enemy_entity, enemy_t, mut health, enemy, xp_val) in &mut enemy_q {
            let ep = enemy_t.translation.truncate();
            let hit_size = 18.0f32; // approximate enemy hit radius

            if pp.distance(ep) < proj_size + hit_size {
                let damage = proj.damage * proj.damage_multiplier;
                health.take_damage(damage);

                events.send(ProjectileHit {
                    projectile: proj_entity,
                    target: enemy_entity,
                    damage,
                });

                if health.is_dead() {
                    kill_events.send(EnemyKilled {
                        entity: enemy_entity,
                        position: ep,
                        xp_value: xp_val.0,
                    });
                }

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
    proj_q: Query<(Entity, &Transform, &EnemyProjectile), Without<WebProjectile>>,
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

fn enemy_death_system(
    mut commands: Commands,
    mut kill_events: EventReader<EnemyKilled>,
    mut game_manager: ResMut<GameManager>,
) {
    for ev in kill_events.read() {
        if let Some(mut entity_commands) = commands.get_entity(ev.entity) {
            entity_commands.despawn_recursive();
        }
        if game_manager.enemies_remaining > 0 {
            game_manager.enemies_remaining -= 1;
        }
        game_manager.total_kills += 1;
    }
}
