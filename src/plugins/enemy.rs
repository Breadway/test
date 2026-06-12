use bevy::prelude::*;
use crate::components::*;
use crate::data::enemies::*;
use crate::events::*;
use crate::resources::*;
use crate::state::GameState;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnExit(GameState::Playing), despawn_all_enemies)
            .add_systems(Update, (
                process_spawn_queue,
                roach_ai,
                spider_ai,
                wasp_ai,
                apply_enemy_velocity,
                enemy_contact_damage,
                bee_ally_ai,
                bee_ally_lifetime,
                larvae_evolve_check,
            ).run_if(in_state(GameState::Playing)));
    }
}

const MAX_SPAWNS_PER_FRAME: usize = 15;

fn spawn_request_position(room_state: &RoomState, i: usize) -> Vec2 {
    let bounds = room_state.bounds;
    let side = i % 4;
    match side {
        0 => Vec2::new(
            bounds.min.x + 20.0,
            bounds.min.y + (bounds.height() * (i as f32 / 40.0).fract()),
        ),
        1 => Vec2::new(
            bounds.max.x - 20.0,
            bounds.min.y + (bounds.height() * (i as f32 / 40.0).fract()),
        ),
        2 => Vec2::new(
            bounds.min.x + (bounds.width() * (i as f32 / 40.0).fract()),
            bounds.min.y + 20.0,
        ),
        _ => Vec2::new(
            bounds.min.x + (bounds.width() * (i as f32 / 40.0).fract()),
            bounds.max.y - 20.0,
        ),
    }
}

fn process_spawn_queue(
    mut commands: Commands,
    mut queue: ResMut<SpawnQueue>,
    mut game_manager: ResMut<GameManager>,
    room_state: Res<RoomState>,
) {
    let spawnable = queue.pending.len().min(MAX_SPAWNS_PER_FRAME);
    for _ in 0..spawnable {
        let Some(req) = queue.pending.pop_front() else { break };
        let i = game_manager.total_kills as usize + queue.pending.len();
        let pos = if req.position == Vec2::ZERO {
            spawn_request_position(&room_state, i)
        } else {
            req.position
        };
        spawn_enemy(&mut commands, req.enemy_type, pos);
        game_manager.enemies_remaining += 1;
    }
}

pub fn spawn_enemy(commands: &mut Commands, enemy_type: EnemyType, pos: Vec2) {
    let cfg = match enemy_type {
        EnemyType::Roach  => &ROACH,
        EnemyType::Spider => &SPIDER,
        EnemyType::Wasp   => &WASP,
        EnemyType::Larvae => &LARVAE,
        EnemyType::Beetle => &BEETLE,
        EnemyType::Boss   => &BOSS,
    };

    let mut entity = commands.spawn((
        Sprite {
            color: cfg.color,
            custom_size: Some(cfg.size),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        Enemy {
            enemy_type,
            speed: cfg.move_speed,
            damage: cfg.damage,
            attack_range: cfg.attack_range,
            attack_cooldown: cfg.attack_cooldown,
            attack_timer: 0.0,
            xp_value: cfg.xp_value,
        },
        Health::new(cfg.max_health),
        XpValue(cfg.xp_value),
        Velocity::default(),
        AIState::default(),
        AggroRange(cfg.aggro_range),
    ));

    if enemy_type == EnemyType::Wasp {
        entity.insert(Flying);
    }

    if enemy_type == EnemyType::Boss {
        entity.insert(Boss { phase: 1 });
    }
}

fn roach_ai(
    time: Res<Time>,
    spatial_grid: Res<SpatialGrid>,
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_q: Query<(&mut Enemy, &mut Velocity, &mut AIState, &Transform, &AggroRange), Without<Player>>,
    mut player_damage: EventWriter<PlayerDamaged>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let player_pos = player_t.translation.truncate();
    let dt = time.delta_secs();

    for (mut enemy, mut vel, mut ai_state, transform, aggro) in &mut enemy_q {
        if enemy.enemy_type != EnemyType::Roach { continue }
        let pos = transform.translation.truncate();
        let dist = pos.distance(player_pos);

        // AI culling beyond 800px
        if dist > 800.0 {
            vel.0 *= 0.95;
            continue;
        }

        if dist < aggro.0 {
            *ai_state = AIState::Chasing;
            if dist > enemy.attack_range {
                let dir = (player_pos - pos).normalize();
                vel.0 = dir * enemy.speed;
            } else {
                vel.0 = Vec2::ZERO;
                *ai_state = AIState::Attacking;
                enemy.attack_timer -= dt;
                if enemy.attack_timer <= 0.0 {
                    player_damage.send(PlayerDamaged {
                        amount: enemy.damage,
                        source: DamageSource::EnemyContact,
                    });
                    enemy.attack_timer = enemy.attack_cooldown;
                }
            }
        } else {
            *ai_state = AIState::Idle;
            vel.0 *= 0.9;
        }
    }
}

fn spider_ai(
    time: Res<Time>,
    mut commands: Commands,
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_q: Query<(&mut Enemy, &mut Velocity, &mut AIState, &Transform, &AggroRange), Without<Player>>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let player_pos = player_t.translation.truncate();
    let dt = time.delta_secs();

    for (mut enemy, mut vel, mut ai_state, transform, aggro) in &mut enemy_q {
        if enemy.enemy_type != EnemyType::Spider { continue }
        let pos = transform.translation.truncate();
        let dist = pos.distance(player_pos);

        if dist > 800.0 {
            vel.0 *= 0.95;
            continue;
        }

        if dist < aggro.0 {
            *ai_state = AIState::Chasing;
            let preferred_dist = 200.0;
            if dist > preferred_dist + 30.0 {
                let dir = (player_pos - pos).normalize();
                vel.0 = dir * enemy.speed;
            } else if dist < preferred_dist - 30.0 {
                let dir = (pos - player_pos).normalize();
                vel.0 = dir * enemy.speed * 0.7;
            } else {
                // Strafe
                let to_player = (player_pos - pos).normalize();
                let strafe = Vec2::new(-to_player.y, to_player.x);
                vel.0 = strafe * enemy.speed * 0.5;

                // Web attack
                enemy.attack_timer -= dt;
                if enemy.attack_timer <= 0.0 {
                    let dir = (player_pos - pos).normalize();
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(0.7, 0.7, 0.8),
                            custom_size: Some(Vec2::new(10.0, 10.0)),
                            ..default()
                        },
                        Transform::from_xyz(pos.x, pos.y, 3.0),
                        WebProjectile { slow_factor: 0.5, slow_duration: 3.0 },
                        Velocity(dir * 200.0),
                        EnemyProjectile { damage: enemy.damage * 0.5 },
                    ));
                    enemy.attack_timer = enemy.attack_cooldown;
                }
            }
        } else {
            vel.0 *= 0.9;
        }
    }
}

fn wasp_ai(
    time: Res<Time>,
    mut commands: Commands,
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_q: Query<(&mut Enemy, &mut Velocity, &mut AIState, &Transform, &AggroRange), (Without<Player>, With<Flying>)>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let player_pos = player_t.translation.truncate();
    let dt = time.delta_secs();

    for (mut enemy, mut vel, mut ai_state, transform, aggro) in &mut enemy_q {
        if enemy.enemy_type != EnemyType::Wasp { continue }
        let pos = transform.translation.truncate();
        let dist = pos.distance(player_pos);

        if dist > 900.0 {
            vel.0 *= 0.95;
            continue;
        }

        let preferred_range = 240.0;
        *ai_state = AIState::Chasing;

        if dist > preferred_range + 40.0 {
            let dir = (player_pos - pos).normalize();
            vel.0 = dir * enemy.speed;
        } else if dist < preferred_range - 40.0 {
            let dir = (pos - player_pos).normalize();
            vel.0 = dir * enemy.speed;
        } else {
            let to_player = (player_pos - pos).normalize();
            let strafe = Vec2::new(-to_player.y, to_player.x);
            vel.0 = strafe * enemy.speed * 0.6;

            enemy.attack_timer -= dt;
            if enemy.attack_timer <= 0.0 {
                let dir = (player_pos - pos).normalize();
                commands.spawn((
                    Sprite {
                        color: Color::srgb(1.0, 0.85, 0.1),
                        custom_size: Some(Vec2::new(8.0, 8.0)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x, pos.y, 3.0),
                    EnemyProjectile { damage: enemy.damage },
                    Velocity(dir * 260.0),
                ));
                enemy.attack_timer = enemy.attack_cooldown;
            }
        }
    }
}

fn apply_enemy_velocity(
    time: Res<Time>,
    room_state: Res<RoomState>,
    mut q: Query<(&Velocity, &mut Transform, Option<&Flying>), With<Enemy>>,
) {
    let dt = time.delta_secs();
    let bounds = room_state.bounds;
    let half = 14.0;

    for (vel, mut transform, flying) in &mut q {
        let new_pos = transform.translation.truncate() + vel.0 * dt;
        if flying.is_some() {
            transform.translation.x = new_pos.x;
            transform.translation.y = new_pos.y;
        } else {
            transform.translation.x = new_pos.x.clamp(bounds.min.x + half, bounds.max.x - half);
            transform.translation.y = new_pos.y.clamp(bounds.min.y + half, bounds.max.y - half);
        }
    }
}

fn enemy_contact_damage(
    time: Res<Time>,
    mut player_damage: EventWriter<PlayerDamaged>,
    player_q: Query<&Transform, With<Player>>,
    mut enemy_q: Query<(&Transform, &mut Enemy), Without<Player>>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let pp = player_t.translation.truncate();
    let dt = time.delta_secs();

    for (et, mut enemy) in &mut enemy_q {
        if enemy.enemy_type == EnemyType::Spider || enemy.enemy_type == EnemyType::Wasp { continue }
        let ep = et.translation.truncate();
        if pp.distance(ep) < 24.0 {
            enemy.attack_timer -= dt;
            if enemy.attack_timer <= 0.0 {
                player_damage.send(PlayerDamaged {
                    amount: enemy.damage,
                    source: DamageSource::EnemyContact,
                });
                enemy.attack_timer = enemy.attack_cooldown;
            }
        }
    }
}

fn bee_ally_ai(
    player_q: Query<&Transform, (With<Player>, Without<BeeAlly>)>,
    mut bee_q: Query<(&mut Velocity, &Transform, &mut Enemy), With<BeeAlly>>,
    enemy_q: Query<&Transform, (With<Enemy>, Without<BeeAlly>, Without<Player>)>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let player_pos = player_t.translation.truncate();

    for (mut vel, bee_t, mut enemy) in &mut bee_q {
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
            let dir = (player_pos - bee_pos).normalize();
            vel.0 = dir * 160.0;
        }
    }
}

fn bee_ally_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut BeeAlly)>,
) {
    let dt = time.delta_secs();
    for (entity, mut bee) in &mut q {
        bee.lifetime -= dt;
        if bee.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn larvae_evolve_check(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Enemy, &Transform)>,
) {
    // Larvae evolve into Roaches if not killed within 15 seconds (simplified: they don't actually have a timer yet)
    // This is a stub for future implementation
}

fn despawn_all_enemies(
    mut commands: Commands,
    q: Query<Entity, With<Enemy>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
