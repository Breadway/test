use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::data::enemies::room_waves;
use crate::data::lore::COLLECTIBLE_DOCS;
use crate::events::*;
use crate::resources::*;
use crate::state::GameState;

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), (setup_room_graph, enter_first_room).chain())
            .add_systems(Update, (
                wave_spawn_tick,
                check_room_clear,
                handle_room_transition,
                projectile_out_of_bounds_cleanup,
                lore_doc_pickup,
                lore_popup_dismiss,
            ).run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::MapSelect), spawn_map_select_ui)
            .add_systems(OnExit(GameState::MapSelect), despawn_map_select_ui)
            .add_systems(Update, map_select_interaction.run_if(in_state(GameState::MapSelect)))
            // Terminal-state cleanup: despawn all run entities. Registered on
            // OnEnter of terminal states (NOT OnExit(Playing)) so the player,
            // enemies, and projectiles survive UpgradeSelect/MapSelect pauses.
            .add_systems(OnEnter(GameState::GameOver), cleanup_run_entities)
            .add_systems(OnEnter(GameState::Victory), cleanup_run_entities)
            .add_systems(OnEnter(GameState::MainMenu), cleanup_run_entities)
            .add_systems(OnEnter(GameState::CharacterSelect), cleanup_run_entities);
    }
}

#[derive(Component)]
pub struct RoomWall;

#[derive(Component)]
pub struct RoomFloor;

#[derive(Component)]
pub struct RoomToastText {
    pub timer: f32,
}

#[derive(Component)]
pub struct LoreDoc(pub usize);

#[derive(Component)]
pub struct LorePopup;

pub fn setup_room_graph(
    mut graph: ResMut<RoomGraph>,
    mut rng: ResMut<GameRng>,
    player_q: Query<(), With<Player>>,
) {
    // Only regenerate at run start (no player yet) — not when resuming from
    // UpgradeSelect/MapSelect mid-run.
    if !player_q.is_empty() {
        return;
    }
    graph.nodes = generate_room_graph(&mut rng.0);
    graph.current_node = 0;
}

fn enter_first_room(
    mut commands: Commands,
    mut room_state: ResMut<RoomState>,
    mut game_manager: ResMut<GameManager>,
    mut spawn_queue: ResMut<SpawnQueue>,
    mut rng: ResMut<GameRng>,
    sprites: Res<crate::data::pixel_art::GameSprites>,
    player_q: Query<(), With<Player>>,
    graph: Res<RoomGraph>,
) {
    // Same guard: only on fresh run start
    if !player_q.is_empty() {
        return;
    }
    game_manager.enemies_remaining = 0;
    setup_room(&mut commands, &mut room_state, &mut game_manager, &mut spawn_queue, &mut rng, &sprites, &graph, 0);
}

fn setup_room(
    commands: &mut Commands,
    room_state: &mut RoomState,
    game_manager: &mut GameManager,
    spawn_queue: &mut SpawnQueue,
    rng: &mut GameRng,
    sprites: &crate::data::pixel_art::GameSprites,
    graph: &RoomGraph,
    room_index: usize,
) {
    room_state.room_index = room_index;
    room_state.waves = room_waves(room_index);
    room_state.wave_index = 0;
    room_state.spawn_timer = 0.0;
    spawn_queue.pending.clear();
    game_manager.enemies_remaining = 0;

    spawn_room_geometry(commands, sprites, room_state.bounds);

    // Lore collectible: 30% chance in combat rooms
    let is_combat = graph.nodes.get(room_index)
        .map(|n| matches!(n.room_type, RoomType::Combat | RoomType::Unknown))
        .unwrap_or(false);
    if is_combat && rng.0.gen_bool(0.3) {
        let doc_index = rng.0.gen_range(0..COLLECTIBLE_DOCS.len());
        let bounds = room_state.bounds;
        let x = rng.0.gen_range(bounds.min.x + 80.0..bounds.max.x - 80.0);
        let y = rng.0.gen_range(bounds.min.y + 80.0..bounds.max.y - 80.0);
        commands.spawn((
            Sprite {
                image: sprites.lore_doc.clone(),
                color: Color::srgb(1.0, 0.95, 0.5),
                custom_size: Some(Vec2::new(16.0, 20.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 2.0),
            LoreDoc(doc_index),
        ));
    }

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

fn spawn_room_geometry(
    commands: &mut Commands,
    sprites: &crate::data::pixel_art::GameSprites,
    bounds: Rect,
) {
    let wall_thickness = 32.0;
    let tile = 16.0; // pixel art tile resolution
    let w = bounds.width();
    let h = bounds.height();
    let cx = (bounds.min.x + bounds.max.x) / 2.0;
    let cy = (bounds.min.y + bounds.max.y) / 2.0;

    // Floor — single large tiled sprite covering the play area
    commands.spawn((
        Sprite {
            image: sprites.floor.clone(),
            color: Color::srgb(0.12, 0.12, 0.15),
            custom_size: Some(Vec2::new(w, h)),
            image_mode: bevy::sprite::SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: tile,
            },
            ..default()
        },
        Transform::from_xyz(cx, cy, -1.0),
        RoomFloor,
    ));

    // Walls — four sides, tiled
    let walls: &[(f32, f32, f32, f32)] = &[
        (cx, bounds.max.y + wall_thickness / 2.0, w + wall_thickness * 2.0, wall_thickness),
        (cx, bounds.min.y - wall_thickness / 2.0, w + wall_thickness * 2.0, wall_thickness),
        (bounds.min.x - wall_thickness / 2.0, cy, wall_thickness, h),
        (bounds.max.x + wall_thickness / 2.0, cy, wall_thickness, h),
    ];

    for &(x, y, wall_w, wall_h) in walls {
        commands.spawn((
            Sprite {
                image: sprites.wall.clone(),
                color: Color::srgb(0.55, 0.55, 0.62),
                custom_size: Some(Vec2::new(wall_w, wall_h)),
                image_mode: bevy::sprite::SpriteImageMode::Tiled {
                    tile_x: true,
                    tile_y: true,
                    stretch_value: tile,
                },
                ..default()
            },
            Transform::from_xyz(x, y, 0.0),
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
    if game_manager.enemies_remaining > 0 || !spawn_queue.pending.is_empty() { return }
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
    spawn_queue: Res<SpawnQueue>,
    mut room_cleared: EventWriter<RoomCleared>,
    mut cleared_flag: Local<bool>,
) {
    let all_waves_done = room_state.wave_index >= room_state.waves.len();
    let no_enemies = game_manager.enemies_remaining == 0;
    // Spawns still pending in the queue have not incremented enemies_remaining yet
    let no_pending = spawn_queue.pending.is_empty();

    if all_waves_done && no_enemies && no_pending && !*cleared_flag {
        *cleared_flag = true;
        room_cleared.send(RoomCleared);
    }

    if !all_waves_done || !no_enemies || !no_pending {
        *cleared_flag = false;
    }
}

fn handle_room_transition(
    mut commands: Commands,
    mut events: EventReader<RoomTransitionRequest>,
    mut room_state: ResMut<RoomState>,
    mut game_manager: ResMut<GameManager>,
    mut spawn_queue: ResMut<SpawnQueue>,
    mut graph: ResMut<RoomGraph>,
    mut rng: ResMut<GameRng>,
    sprites: Res<crate::data::pixel_art::GameSprites>,
    cleanup_q: Query<Entity, Or<(With<RoomWall>, With<RoomFloor>, With<EnemyProjectile>, With<WebProjectile>, With<Projectile>, With<LoreDoc>)>>,
) {
    for ev in events.read() {
        for e in &cleanup_q { commands.entity(e).despawn_recursive(); }

        let prev = graph.current_node;
        if let Some(node) = graph.nodes.get_mut(prev) {
            node.completed = true;
        }
        graph.current_node = ev.destination_room;

        setup_room(&mut commands, &mut room_state, &mut game_manager, &mut spawn_queue, &mut rng, &sprites, &graph, ev.destination_room);
        game_manager.rooms_cleared += 1;
    }
}

fn projectile_out_of_bounds_cleanup(
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

// ---- Lore collectibles ----

fn lore_doc_pickup(
    mut commands: Commands,
    mut game_manager: ResMut<GameManager>,
    player_q: Query<&Transform, With<Player>>,
    doc_q: Query<(Entity, &Transform, &LoreDoc)>,
    popup_q: Query<(), With<LorePopup>>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let pp = player_t.translation.truncate();

    for (entity, doc_t, doc) in &doc_q {
        if pp.distance(doc_t.translation.truncate()) < 26.0 {
            commands.entity(entity).despawn_recursive();
            if !game_manager.discovered_docs.contains(&doc.0) {
                game_manager.discovered_docs.push(doc.0);
            }
            if !popup_q.is_empty() { continue }

            let (title, body) = COLLECTIBLE_DOCS[doc.0];
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(40.0),
                    left: Val::Percent(25.0),
                    width: Val::Percent(50.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.03, 0.95)),
                BorderColor(Color::srgb(0.8, 0.75, 0.4)),
                LorePopup,
            )).with_children(|popup| {
                popup.spawn((
                    Text::new(title),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.95, 0.5)),
                ));
                popup.spawn((
                    Text::new(body),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.75)),
                ));
                popup.spawn((
                    Text::new("[ Space to file away ]"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.5, 0.45)),
                ));
            });
        }
    }
}

fn lore_popup_dismiss(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    popup_q: Query<Entity, With<LorePopup>>,
) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Escape) {
        for e in &popup_q {
            commands.entity(e).despawn_recursive();
        }
    }
}

// ---- Map node selection (branching paths) ----

#[derive(Component)]
struct MapSelectRoot;

#[derive(Component)]
struct MapNodeButton(usize);

fn spawn_map_select_ui(
    mut commands: Commands,
    graph: Res<RoomGraph>,
) {
    let next = graph.available_next();

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(24.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        MapSelectRoot,
    )).with_children(|root| {
        root.spawn((
            Text::new("NEXT DEPLOYMENT ZONE"),
            TextFont { font_size: 28.0, ..default() },
            TextColor(Color::srgb(1.0, 0.7, 0.1)),
        ));

        root.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(24.0),
            ..default()
        }).with_children(|row| {
            for &node_id in &next {
                let Some(node) = graph.nodes.get(node_id) else { continue };
                let (type_label, color) = match node.room_type {
                    RoomType::Combat  => ("COMBAT", Color::srgb(0.8, 0.3, 0.2)),
                    RoomType::Elite   => ("ELITE",  Color::srgb(0.8, 0.2, 0.8)),
                    RoomType::Shop    => ("SUPPLY", Color::srgb(0.2, 0.7, 0.3)),
                    RoomType::Unknown => ("???",    Color::srgb(0.6, 0.6, 0.6)),
                    RoomType::Boss    => ("BOSS",   Color::srgb(1.0, 0.1, 0.1)),
                };

                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        min_height: Val::Px(110.0),
                        padding: UiRect::all(Val::Px(14.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.14)),
                    BorderColor(color),
                    MapNodeButton(node_id),
                )).with_children(|card| {
                    card.spawn((
                        Text::new(format!("ZONE {:02} — {}", node.id, type_label)),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(color),
                    ));
                    card.spawn((
                        Text::new(node.lore_name.clone()),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                });
            }
        });
    });
}

fn despawn_map_select_ui(
    mut commands: Commands,
    q: Query<Entity, With<MapSelectRoot>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn map_select_interaction(
    interaction_q: Query<(&Interaction, &MapNodeButton), (Changed<Interaction>, With<Button>)>,
    mut transition_event: EventWriter<RoomTransitionRequest>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed {
            transition_event.send(RoomTransitionRequest { destination_room: btn.0 });
            next_state.set(GameState::Playing);
            break;
        }
    }
}

// ---- Terminal-state cleanup ----

fn cleanup_run_entities(
    mut commands: Commands,
    q: Query<Entity, Or<(
        With<Player>,
        With<Enemy>,
        With<BeeAlly>,
        With<Projectile>,
        With<EnemyProjectile>,
        With<WebProjectile>,
        With<RoomWall>,
        With<RoomFloor>,
        With<LoreDoc>,
        With<LorePopup>,
        With<crate::plugins::visual::ImpactParticle>,
    )>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
