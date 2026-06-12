use bevy::prelude::*;
use crate::components::*;
use crate::data::enemies::room_waves;
use crate::data::lore::*;
use crate::events::*;
use crate::resources::*;
use crate::state::GameState;
use rand::Rng;

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), (setup_room_graph, enter_first_room).chain())
            .add_systems(Update, (
                wave_spawn_tick,
                check_room_clear,
                handle_room_transition,
                enemy_projectile_cleanup,
            ).run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::MapSelect), rebuild_walls_for_next_room);
    }
}

#[derive(Component)]
pub struct RoomWall;

#[derive(Component)]
pub struct RoomToastText {
    pub timer: f32,
}

fn setup_room_graph(
    mut graph: ResMut<RoomGraph>,
    mut rng: ResMut<GameRng>,
) {
    graph.nodes.clear();
    let adjectives = ROOM_ADJECTIVES;
    let nouns = ROOM_NOUNS;

    // Generate a 25-node graph with branching paths
    for i in 0..25 {
        let adj = adjectives[rng.0.gen_range(0..adjectives.len())];
        let noun = nouns[rng.0.gen_range(0..nouns.len())];
        let lore_name = format!("{} {}", adj, noun);

        let room_type = match i {
            11 | 24 => RoomType::Boss,
            6 | 16  => RoomType::Shop,
            3 | 13 | 20 => RoomType::Elite,
            _ => RoomType::Combat,
        };

        // Each node connects to 1-2 next nodes
        let children = if i < 24 {
            let next = i + 1;
            if i % 3 == 0 && i + 2 < 25 {
                vec![next, next + 1]
            } else {
                vec![next]
            }
        } else {
            vec![]
        };

        graph.nodes.push(RoomNode {
            id: i,
            room_type,
            depth: i,
            children,
            completed: false,
            lore_name,
        });
    }
    graph.current_node = 0;
}

fn enter_first_room(
    mut commands: Commands,
    mut room_state: ResMut<RoomState>,
    mut game_manager: ResMut<GameManager>,
    mut spawn_queue: ResMut<SpawnQueue>,
    graph: Res<RoomGraph>,
) {
    game_manager.enemies_remaining = 0;
    setup_room(&mut commands, &mut room_state, &mut game_manager, &mut spawn_queue, 0);
}

fn setup_room(
    commands: &mut Commands,
    room_state: &mut RoomState,
    game_manager: &mut GameManager,
    spawn_queue: &mut SpawnQueue,
    room_index: usize,
) {
    room_state.room_index = room_index;
    room_state.waves = room_waves(room_index);
    room_state.wave_index = 0;
    room_state.spawn_timer = 0.0;
    room_state.spawn_queue.clear();
    spawn_queue.pending.clear();
    game_manager.enemies_remaining = 0;

    // Spawn walls
    spawn_walls(commands, room_state.bounds);

    // Queue first wave
    if let Some(wave) = room_state.waves.first() {
        for &(enemy_type, count) in &wave.spawns {
            for _ in 0..count {
                spawn_queue.pending.push_back(crate::resources::spawn::SpawnRequest {
                    enemy_type,
                    position: Vec2::ZERO,
                });
            }
        }
        room_state.wave_index = 1;
    }
}

fn spawn_walls(commands: &mut Commands, bounds: Rect) {
    let wall_thickness = 24.0;
    let w = bounds.width();
    let h = bounds.height();
    let cx = (bounds.min.x + bounds.max.x) / 2.0;
    let cy = (bounds.min.y + bounds.max.y) / 2.0;

    let walls = [
        // Top
        (cx, bounds.max.y + wall_thickness / 2.0, w + wall_thickness * 2.0, wall_thickness),
        // Bottom
        (cx, bounds.min.y - wall_thickness / 2.0, w + wall_thickness * 2.0, wall_thickness),
        // Left
        (bounds.min.x - wall_thickness / 2.0, cy, wall_thickness, h),
        // Right
        (bounds.max.x + wall_thickness / 2.0, cy, wall_thickness, h),
    ];

    for (x, y, wall_w, wall_h) in walls {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.18, 0.18, 0.2),
                custom_size: Some(Vec2::new(wall_w, wall_h)),
                ..default()
            },
            Transform::from_xyz(x, y, 0.5),
            RoomWall,
        ));
    }
}

fn wave_spawn_tick(
    time: Res<Time>,
    mut room_state: ResMut<RoomState>,
    mut spawn_queue: ResMut<SpawnQueue>,
    game_manager: Res<GameManager>,
) {
    // If enemies still alive from previous wave, wait
    if game_manager.enemies_remaining > 0 { return }
    if room_state.wave_index >= room_state.waves.len() { return }

    room_state.spawn_timer -= time.delta_secs();
    if room_state.spawn_timer > 0.0 { return }

    let wave = room_state.waves[room_state.wave_index].clone();
    for &(enemy_type, count) in &wave.spawns {
        for _ in 0..count {
            spawn_queue.pending.push_back(crate::resources::spawn::SpawnRequest {
                enemy_type,
                position: Vec2::ZERO,
            });
        }
    }
    room_state.wave_index += 1;
    room_state.spawn_timer = wave.spawn_delay;
}

fn check_room_clear(
    game_manager: Res<GameManager>,
    room_state: Res<RoomState>,
    mut room_cleared: EventWriter<RoomCleared>,
    mut cleared_flag: Local<bool>,
) {
    let all_waves_done = room_state.wave_index >= room_state.waves.len();
    let no_enemies = game_manager.enemies_remaining == 0;

    if all_waves_done && no_enemies && !*cleared_flag {
        *cleared_flag = true;
        room_cleared.send(RoomCleared);
    }

    if !all_waves_done || !no_enemies {
        *cleared_flag = false;
    }
}

fn handle_room_transition(
    mut commands: Commands,
    mut events: EventReader<RoomTransitionRequest>,
    mut room_state: ResMut<RoomState>,
    mut game_manager: ResMut<GameManager>,
    mut spawn_queue: ResMut<SpawnQueue>,
    wall_q: Query<Entity, With<RoomWall>>,
    enemy_proj_q: Query<Entity, Or<(With<EnemyProjectile>, With<WebProjectile>)>>,
) {
    for ev in events.read() {
        // Despawn walls and projectiles
        for e in &wall_q { commands.entity(e).despawn_recursive(); }
        for e in &enemy_proj_q { commands.entity(e).despawn_recursive(); }

        setup_room(&mut commands, &mut room_state, &mut game_manager, &mut spawn_queue, ev.destination_room);
        game_manager.rooms_cleared += 1;
    }
}

fn rebuild_walls_for_next_room(
    mut commands: Commands,
    wall_q: Query<Entity, With<RoomWall>>,
    room_state: Res<RoomState>,
) {
    for e in &wall_q { commands.entity(e).despawn_recursive(); }
    spawn_walls(&mut commands, room_state.bounds);
}

fn enemy_projectile_cleanup(
    mut commands: Commands,
    room_state: Res<RoomState>,
    q: Query<(Entity, &Transform), Or<(With<EnemyProjectile>, With<WebProjectile>)>>,
) {
    let bounds = room_state.bounds.inflate(100.0);
    for (entity, transform) in &q {
        let pos = transform.translation.truncate();
        if !bounds.contains(pos) {
            commands.entity(entity).despawn_recursive();
        }
    }
}
