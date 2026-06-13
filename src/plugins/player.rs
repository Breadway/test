use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::components::*;
use crate::data::characters::stats_for_character;
use crate::events::*;
use crate::resources::*;
use crate::state::GameState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, (
                player_movement,
                player_aim,
                player_attack,
                cooldown_tick,
                stun_tick,
                apply_velocity_player,
                player_damage_handler,
                damage_flash_tick,
                slow_status_tick,
            ).run_if(in_state(GameState::Playing)));
    }
}

pub const PLAYER_Z: f32 = 10.0;

fn spawn_player(
    mut commands: Commands,
    selection: Res<CharacterSelection>,
    sprites: Res<crate::data::pixel_art::GameSprites>,
    existing: Query<Entity, With<Player>>,
) {
    // Re-entering Playing (from UpgradeSelect/MapSelect) must not spawn a second player
    if !existing.is_empty() {
        return;
    }
    let stats = stats_for_character(selection.character);
    let color = player_color(selection.character);

    commands.spawn((
        Sprite {
            image: sprites.player.clone(),
            color,
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, PLAYER_Z),
        Player,
        AttackCooldown(0.0),
        Velocity::default(),
        Facing::default(),
        Health::new(stats.max_health),
        stats,
    ));
}

fn player_color(ct: crate::components::player::CharacterType) -> Color {
    use crate::components::player::CharacterType::*;
    match ct {
        Torch  => Color::srgb(1.0, 0.4, 0.1),
        QueenB => Color::srgb(1.0, 0.85, 0.1),
        Mop    => Color::srgb(0.4, 0.7, 1.0),
        Heavy  => Color::srgb(0.4, 0.5, 0.4),
        Sparky => Color::srgb(0.9, 0.95, 0.2),
        Doc    => Color::srgb(0.7, 0.2, 0.9),
    }
}

fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&PlayerStats, &mut Velocity, Option<&Stunned>, Option<&SlowStatus>), With<Player>>,
) {
    let Ok((stats, mut vel, stunned, slow)) = q.get_single_mut() else { return };

    if stunned.is_some() {
        vel.0 = Vec2::ZERO;
        return;
    }

    let speed = stats.effective_speed() * slow.map_or(1.0, |s| s.factor);
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)    { dir.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)  { dir.y -= 1.0; }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)  { dir.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { dir.x += 1.0; }

    vel.0 = if dir != Vec2::ZERO { dir.normalize() * speed } else { Vec2::ZERO };
}

fn stun_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Stunned)>,
) {
    let dt = time.delta_secs();
    for (entity, mut stun) in &mut q {
        stun.0 -= dt;
        if stun.0 <= 0.0 {
            commands.entity(entity).remove::<Stunned>();
        }
    }
}

fn player_aim(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut q: Query<(&Transform, &mut Facing), With<Player>>,
) {
    let Ok(window) = windows.get_single() else { return };
    let Ok((cam, cam_gt)) = camera_q.get_single() else { return };
    let Ok((player_t, mut facing)) = q.get_single_mut() else { return };

    if let Some(cursor) = window.cursor_position() {
        if let Ok(world_pos) = cam.viewport_to_world_2d(cam_gt, cursor) {
            let dir = world_pos - player_t.translation.truncate();
            if dir.length_squared() > 1.0 {
                facing.0 = dir.normalize();
            }
        }
    }
}

/// Standard projectile attack. Characters with bespoke attacks (Queen B's
/// pheromone darts excepted, Mop melee, Heavy foam, Doc charge beam) are
/// excluded here and handled in their own character systems.
fn player_attack(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    sprites: Res<crate::data::pixel_art::GameSprites>,
    mut shot_fired: EventWriter<ShotFired>,
    mut q: Query<(&Transform, &Facing, &PlayerStats, &mut AttackCooldown, Option<&Stunned>), With<Player>>,
) {
    use crate::components::player::CharacterType::*;
    let Ok((transform, facing, stats, mut cooldown, stunned)) = q.get_single_mut() else { return };
    if cooldown.0 > 0.0 || stunned.is_some() { return }

    // These characters do not fire standard projectiles:
    // QueenB cannot deal direct damage; Mop is melee; Heavy sprays foam; Doc charges.
    if matches!(stats.character_type, QueenB | Mop | Heavy | Doc) { return }

    if buttons.pressed(MouseButton::Left) {
        let dir = if facing.0 != Vec2::ZERO { facing.0 } else { Vec2::X };
        let origin = transform.translation.truncate() + dir * 18.0;

        commands.spawn((
            Sprite {
                image: sprites.bullet.clone(),
                color: projectile_color(stats.character_type),
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..default()
            },
            Transform::from_xyz(origin.x, origin.y, 5.0),
            Projectile {
                damage: stats.attack_damage * stats.damage_multiplier,
                lifetime: stats.projectile_range / stats.projectile_speed,
                pierce: 0,
                damage_multiplier: 1.0,
                hit_entities: Vec::new(),
            },
            Velocity(dir * stats.projectile_speed),
        ));

        shot_fired.send(ShotFired);
        cooldown.0 = 1.0 / stats.attack_rate;
    }
}

fn projectile_color(ct: crate::components::player::CharacterType) -> Color {
    use crate::components::player::CharacterType::*;
    match ct {
        Torch  => Color::srgb(1.0, 0.5, 0.0),
        QueenB => Color::srgb(1.0, 0.9, 0.0),
        Mop    => Color::srgb(0.8, 0.9, 1.0),
        Heavy  => Color::srgb(0.6, 0.8, 0.6),
        Sparky => Color::srgb(0.8, 0.9, 1.0),
        Doc    => Color::srgb(0.8, 0.3, 1.0),
    }
}

fn cooldown_tick(
    time: Res<Time>,
    mut q: Query<&mut AttackCooldown>,
) {
    let dt = time.delta_secs();
    for mut cd in &mut q {
        if cd.0 > 0.0 { cd.0 -= dt; }
    }
}

fn apply_velocity_player(
    time: Res<Time>,
    room_state: Res<RoomState>,
    mut q: Query<(&Velocity, &mut Transform), With<Player>>,
) {
    let dt = time.delta_secs();
    let bounds = room_state.bounds;
    let half_size = Vec2::new(14.0, 14.0);

    for (vel, mut transform) in &mut q {
        let new_pos = transform.translation.truncate() + vel.0 * dt;
        transform.translation.x = new_pos.x.clamp(bounds.min.x + half_size.x, bounds.max.x - half_size.x);
        transform.translation.y = new_pos.y.clamp(bounds.min.y + half_size.y, bounds.max.y - half_size.y);
    }
}

/// The ONLY consumer of PlayerDamaged. All character-specific mitigation
/// (Heavy's foam armor, Queen B's bee shield, path damage reduction) is
/// applied here so damage is processed exactly once.
fn player_damage_handler(
    mut commands: Commands,
    mut events: EventReader<PlayerDamaged>,
    mut q: Query<(Entity, &mut Health, &mut PlayerStats), With<Player>>,
    bee_q: Query<Entity, With<BeeAlly>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    use crate::components::player::CharacterType::*;
    let Ok((player_entity, mut health, mut stats)) = q.get_single_mut() else { return };

    let mut bees: Vec<Entity> = Vec::new();
    let mut flash: Option<Color> = None;

    for ev in events.read() {
        let mut amount = ev.amount * (1.0 - stats.damage_reduction).max(0.0);

        match stats.character_type {
            Heavy => {
                // Foam armor absorbs first; losing armor speeds Heavy back up
                if stats.foam_armor > 0.0 {
                    let absorbed = amount.min(stats.foam_armor);
                    stats.foam_armor -= absorbed;
                    amount -= absorbed;
                }
            }
            QueenB => {
                // Shared Fate: damage kills bees before touching HP (1 bee per 4 dmg)
                if bees.is_empty() {
                    bees = bee_q.iter().collect();
                }
                while amount >= 4.0 && !bees.is_empty() {
                    let bee = bees.pop().unwrap();
                    commands.entity(bee).despawn_recursive();
                    stats.bee_count = stats.bee_count.saturating_sub(1);
                    amount -= 4.0;
                }
            }
            _ => {}
        }

        if amount > 0.0 {
            health.take_damage(amount);
            // Threshold skips Torch's tiny per-frame self-burn ticks
            if amount >= 1.0 {
                flash = Some(match ev.source {
                    DamageSource::OwnFire | DamageSource::Electric => Color::srgb(1.0, 0.6, 0.1),
                    _ => Color::srgb(1.0, 0.2, 0.2),
                });
            }
        }
    }

    if let Some(color) = flash {
        commands.entity(player_entity).insert(DamageFlash { timer: 0.15, color });
    }

    if health.is_dead() {
        next_state.set(GameState::GameOver);
    }
}

fn damage_flash_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut DamageFlash, &mut Sprite, &PlayerStats), With<Player>>,
) {
    let dt = time.delta_secs();
    for (entity, mut flash, mut sprite, stats) in &mut q {
        flash.timer -= dt;
        if flash.timer <= 0.0 {
            sprite.color = player_color(stats.character_type);
            commands.entity(entity).remove::<DamageFlash>();
        } else {
            sprite.color = flash.color;
        }
    }
}

fn slow_status_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut SlowStatus)>,
) {
    let dt = time.delta_secs();
    for (entity, mut slow) in &mut q {
        slow.remaining -= dt;
        if slow.remaining <= 0.0 {
            commands.entity(entity).remove::<SlowStatus>();
        }
    }
}
