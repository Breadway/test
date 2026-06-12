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
            .add_systems(OnExit(GameState::Playing), despawn_player)
            .add_systems(Update, (
                player_movement,
                player_aim,
                player_attack,
                cooldown_tick,
                apply_velocity_player,
                player_damage_handler,
                slow_status_tick,
            ).run_if(in_state(GameState::Playing)));
    }
}

pub const PLAYER_Z: f32 = 10.0;

fn spawn_player(
    mut commands: Commands,
    selection: Res<CharacterSelection>,
) {
    let stats = stats_for_character(selection.character);
    let color = player_color(selection.character);

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, PLAYER_Z),
        Player,
        stats,
        AttackCooldown(0.0),
        Velocity::default(),
        Facing::default(),
        Health::new(120.0),
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

fn despawn_player(
    mut commands: Commands,
    q: Query<Entity, With<Player>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&PlayerStats, &mut Velocity), With<Player>>,
) {
    let Ok((stats, mut vel)) = q.get_single_mut() else { return };
    let speed = stats.effective_speed();

    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)    { dir.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)  { dir.y -= 1.0; }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)  { dir.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { dir.x += 1.0; }

    vel.0 = if dir != Vec2::ZERO { dir.normalize() * speed } else { Vec2::ZERO };
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

fn player_attack(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mut q: Query<(&Transform, &Facing, &PlayerStats, &mut AttackCooldown), With<Player>>,
    game_manager: Res<GameManager>,
) {
    let Ok((transform, facing, stats, mut cooldown)) = q.get_single_mut() else { return };
    if cooldown.0 > 0.0 { return }

    // Queen B can't deal direct damage
    if stats.character_type == crate::components::player::CharacterType::QueenB { return }

    if buttons.pressed(MouseButton::Left) {
        let dir = if facing.0 != Vec2::ZERO { facing.0 } else { Vec2::X };
        let origin = transform.translation.truncate() + dir * 18.0;

        commands.spawn((
            Sprite {
                color: projectile_color(stats.character_type),
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..default()
            },
            Transform::from_xyz(origin.x, origin.y, 5.0),
            Projectile {
                damage: stats.attack_damage * stats.damage_multiplier,
                owner_is_player: true,
                lifetime: stats.projectile_range / stats.projectile_speed,
                pierce: 0,
                damage_multiplier: 1.0,
            },
            Velocity(dir * stats.projectile_speed),
        ));

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

fn player_damage_handler(
    mut events: EventReader<PlayerDamaged>,
    mut q: Query<&mut Health, With<Player>>,
    mut game_manager: ResMut<GameManager>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(mut health) = q.get_single_mut() else { return };
    for ev in events.read() {
        health.take_damage(ev.amount);
        if health.is_dead() {
            next_state.set(GameState::GameOver);
        }
    }
}

fn slow_status_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut SlowStatus, &mut Speed)>,
) {
    let dt = time.delta_secs();
    for (entity, mut slow, mut speed) in &mut q {
        slow.remaining -= dt;
        if slow.remaining <= 0.0 {
            speed.0 /= slow.factor;
            commands.entity(entity).remove::<SlowStatus>();
        }
    }
}
