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
            .add_systems(Update, (
                process_spawn_queue,
                chase_ai,
                spider_ai,
                wasp_ai,
                boss_ai,
                larvae_evolve,
                apply_enemy_velocity,
                enemy_contact_damage,
            ).run_if(in_state(GameState::Playing)));
    }
}

const MAX_SPAWNS_PER_FRAME: usize = 15;

fn spawn_request_position(room_state: &RoomState, i: usize) -> Vec2 {
    let bounds = room_state.bounds;
    let side = i % 4;
    let frac = ((i as f32) * 0.137).fract();
    match side {
        0 => Vec2::new(bounds.min.x + 20.0, bounds.min.y + bounds.height() * frac),
        1 => Vec2::new(bounds.max.x - 20.0, bounds.min.y + bounds.height() * frac),
        2 => Vec2::new(bounds.min.x + bounds.width() * frac, bounds.min.y + 20.0),
        _ => Vec2::new(bounds.min.x + bounds.width() * frac, bounds.max.y - 20.0),
    }
}

fn process_spawn_queue(
    mut commands: Commands,
    mut queue: ResMut<SpawnQueue>,
    mut game_manager: ResMut<GameManager>,
    room_state: Res<RoomState>,
) {
    let spawnable = queue.pending.len().min(MAX_SPAWNS_PER_FRAME);
    for n in 0..spawnable {
        let Some(req) = queue.pending.pop_front() else { break };
        let i = game_manager.total_kills as usize + queue.pending.len() + n;
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
        AggroRange(cfg.aggro_range),
    ));

    match enemy_type {
        EnemyType::Wasp => { entity.insert(Flying); }
        EnemyType::Larvae => { entity.insert(EvolveTimer(15.0)); }
        EnemyType::Boss => {
            entity.insert(Boss { phase: 1, summon_timer: 10.0, burst_timer: 4.0 });
        }
        _ => {}
    }
}

/// Shared chase AI for ground melee enemies: Roach, Larvae, Beetle.
/// Damage dealing lives exclusively in `enemy_contact_damage`.
fn chase_ai(
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_q: Query<(&Enemy, &mut Velocity, &Transform, &AggroRange), (Without<Player>, Without<Boss>, Without<Dead>)>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let player_pos = player_t.translation.truncate();

    for (enemy, mut vel, transform, aggro) in &mut enemy_q {
        if !matches!(enemy.enemy_type, EnemyType::Roach | EnemyType::Larvae | EnemyType::Beetle) {
            continue;
        }
        let pos = transform.translation.truncate();
        let dist = pos.distance(player_pos);

        // AI culling beyond 800px
        if dist > 800.0 {
            vel.0 *= 0.95;
            continue;
        }

        if dist < aggro.0 && dist > enemy.attack_range {
            let dir = (player_pos - pos).normalize();
            vel.0 = dir * enemy.speed;
        } else {
            vel.0 = Vec2::ZERO;
        }
    }
}

fn spider_ai(
    time: Res<Time>,
    mut commands: Commands,
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_q: Query<(&mut Enemy, &mut Velocity, &Transform, &AggroRange), (Without<Player>, Without<Dead>)>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let player_pos = player_t.translation.truncate();
    let dt = time.delta_secs();

    for (mut enemy, mut vel, transform, aggro) in &mut enemy_q {
        if enemy.enemy_type != EnemyType::Spider { continue }
        let pos = transform.translation.truncate();
        let dist = pos.distance(player_pos);

        if dist > 800.0 {
            vel.0 *= 0.95;
            continue;
        }

        if dist < aggro.0 {
            let preferred_dist = 200.0;
            if dist > preferred_dist + 30.0 {
                let dir = (player_pos - pos).normalize();
                vel.0 = dir * enemy.speed;
            } else if dist < preferred_dist - 30.0 {
                let dir = (pos - player_pos).normalize();
                vel.0 = dir * enemy.speed * 0.7;
            } else {
                let to_player = (player_pos - pos).normalize();
                let strafe = Vec2::new(-to_player.y, to_player.x);
                vel.0 = strafe * enemy.speed * 0.5;

                // Web shot: slows the player, no damage (slow-only by design)
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
    mut enemy_q: Query<(&mut Enemy, &mut Velocity, &Transform), (Without<Player>, With<Flying>, Without<Dead>)>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let player_pos = player_t.translation.truncate();
    let dt = time.delta_secs();

    for (mut enemy, mut vel, transform) in &mut enemy_q {
        if enemy.enemy_type != EnemyType::Wasp { continue }
        let pos = transform.translation.truncate();
        let dist = pos.distance(player_pos);

        if dist > 900.0 {
            vel.0 *= 0.95;
            continue;
        }

        let preferred_range = 240.0;

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

/// Boss: 3 phases by HP fraction.
/// P1 (>66%): chase. P2 (33-66%): +40% speed, summons roaches.
/// P3 (<33%): +80% speed, radial projectile bursts.
fn boss_ai(
    time: Res<Time>,
    mut commands: Commands,
    mut queue: ResMut<SpawnQueue>,
    mut phase_events: EventWriter<BossPhaseChanged>,
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut boss_q: Query<(&Enemy, &mut Boss, &Health, &mut Velocity, &Transform, &mut Sprite), Without<Dead>>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let player_pos = player_t.translation.truncate();
    let dt = time.delta_secs();

    for (enemy, mut boss, health, mut vel, transform, mut sprite) in &mut boss_q {
        let pos = transform.translation.truncate();
        let frac = health.fraction();

        let target_phase = if frac > 0.66 { 1 } else if frac > 0.33 { 2 } else { 3 };
        if target_phase != boss.phase {
            boss.phase = target_phase;
            phase_events.send(BossPhaseChanged { new_phase: target_phase });
            sprite.color = match target_phase {
                2 => Color::srgb(0.8, 0.2, 0.0),
                3 => Color::srgb(1.0, 0.1, 0.3),
                _ => Color::srgb(0.6, 0.0, 0.0),
            };
        }

        let speed_mult = match boss.phase {
            2 => 1.4,
            3 => 1.8,
            _ => 1.0,
        };

        let dist = pos.distance(player_pos);
        if dist > enemy.attack_range {
            let dir = (player_pos - pos).normalize();
            vel.0 = dir * enemy.speed * speed_mult;
        } else {
            vel.0 = Vec2::ZERO;
        }

        // Phase 2+: summon adds
        if boss.phase >= 2 {
            boss.summon_timer -= dt;
            if boss.summon_timer <= 0.0 {
                boss.summon_timer = 10.0;
                for k in 0..5 {
                    let angle = k as f32 * std::f32::consts::TAU / 5.0;
                    let offset = Vec2::new(angle.cos(), angle.sin()) * 60.0;
                    queue.pending.push_back(crate::resources::spawn::SpawnRequest {
                        enemy_type: EnemyType::Roach,
                        position: pos + offset,
                    });
                }
            }
        }

        // Phase 3: radial projectile burst
        if boss.phase >= 3 {
            boss.burst_timer -= dt;
            if boss.burst_timer <= 0.0 {
                boss.burst_timer = 4.0;
                for k in 0..8 {
                    let angle = k as f32 * std::f32::consts::TAU / 8.0;
                    let dir = Vec2::new(angle.cos(), angle.sin());
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(1.0, 0.2, 0.2),
                            custom_size: Some(Vec2::new(10.0, 10.0)),
                            ..default()
                        },
                        Transform::from_xyz(pos.x, pos.y, 3.0),
                        EnemyProjectile { damage: enemy.damage * 0.6 },
                        Velocity(dir * 220.0),
                    ));
                }
            }
        }
    }
}

/// Larvae evolve in place into Roaches — in-place swap avoids enemy-count churn.
fn larvae_evolve(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut EvolveTimer, &mut Enemy, &mut Health, &mut Sprite, &mut XpValue), Without<Dead>>,
) {
    let dt = time.delta_secs();
    for (entity, mut timer, mut enemy, mut health, mut sprite, mut xp) in &mut q {
        timer.0 -= dt;
        if timer.0 <= 0.0 {
            enemy.enemy_type = EnemyType::Roach;
            enemy.speed = ROACH.move_speed;
            enemy.damage = ROACH.damage;
            enemy.attack_range = ROACH.attack_range;
            enemy.attack_cooldown = ROACH.attack_cooldown;
            enemy.xp_value = ROACH.xp_value;
            health.max = ROACH.max_health;
            health.current = ROACH.max_health;
            sprite.color = ROACH.color;
            sprite.custom_size = Some(ROACH.size);
            xp.0 = ROACH.xp_value;
            commands.entity(entity).remove::<EvolveTimer>();
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

/// The single source of enemy melee damage (Spider/Wasp attack at range).
fn enemy_contact_damage(
    time: Res<Time>,
    mut player_damage: EventWriter<PlayerDamaged>,
    player_q: Query<&Transform, With<Player>>,
    mut enemy_q: Query<(&Transform, &mut Enemy), (Without<Player>, Without<Dead>)>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let pp = player_t.translation.truncate();
    let dt = time.delta_secs();

    for (et, mut enemy) in &mut enemy_q {
        if matches!(enemy.enemy_type, EnemyType::Spider | EnemyType::Wasp) { continue }
        let ep = et.translation.truncate();
        let reach = enemy.attack_range.max(24.0);
        if pp.distance(ep) < reach {
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
