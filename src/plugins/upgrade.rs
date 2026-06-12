use bevy::prelude::*;
use rand::seq::SliceRandom;
use crate::components::*;
use crate::data::upgrades::*;
use crate::events::*;
use crate::resources::*;
use crate::state::GameState;

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(UpgradeState::default())
            .add_systems(OnEnter(GameState::UpgradeSelect), show_upgrade_ui)
            .add_systems(Update, handle_upgrade_click.run_if(in_state(GameState::UpgradeSelect)))
            .add_systems(OnExit(GameState::UpgradeSelect), hide_upgrade_ui)
            .add_systems(Update, room_cleared_handler.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
struct UpgradeUIRoot;

#[derive(Component)]
struct UpgradeCard(UpgradeId);

fn room_cleared_handler(
    mut events: EventReader<RoomCleared>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _ in events.read() {
        next_state.set(GameState::UpgradeSelect);
    }
}

fn show_upgrade_ui(
    mut commands: Commands,
    mut upgrade_state: ResMut<UpgradeState>,
    mut rng: ResMut<GameRng>,
) {
    use rand::Rng;

    let all_general = general_upgrades();
    let available: Vec<UpgradeId> = all_general
        .into_iter()
        .filter(|id| !upgrade_state.owned.contains(id))
        .collect();

    let mut pool = available;
    if pool.is_empty() {
        pool = general_upgrades();
    }

    let count = pool.len().min(3);
    let mut offered = Vec::new();
    let mut indices: Vec<usize> = (0..pool.len()).collect();
    indices.shuffle(&mut rng.0);
    for &i in indices.iter().take(count) {
        offered.push(pool[i]);
    }
    // Pad to 3 if needed
    while offered.len() < 3 {
        offered.push(UpgradeId::DamageBoost);
    }

    upgrade_state.offered = offered.clone();

    // Spawn UI
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        UpgradeUIRoot,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("PERFORMANCE REVIEW"),
            TextFont { font_size: 32.0, ..default() },
            TextColor(Color::srgb(1.0, 0.7, 0.1)),
        ));
        parent.spawn((
            Text::new("Choose 1 of 3 upgrades"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));

        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(24.0),
                ..default()
            },
        )).with_children(|row| {
            for upgrade_id in &offered {
                if let Some(data) = get_upgrade(*upgrade_id) {
                    let (bg_color, border_color) = match data.category {
                        UpgradeCategory::Power   => (Color::srgb(0.35, 0.1, 0.1), Color::srgb(0.9, 0.2, 0.2)),
                        UpgradeCategory::Defense => (Color::srgb(0.1, 0.2, 0.35), Color::srgb(0.2, 0.5, 0.9)),
                        UpgradeCategory::Utility => (Color::srgb(0.1, 0.25, 0.1), Color::srgb(0.2, 0.8, 0.3)),
                        UpgradeCategory::Character => (Color::srgb(0.25, 0.1, 0.3), Color::srgb(0.7, 0.2, 0.9)),
                    };

                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(240.0),
                            min_height: Val::Px(180.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(bg_color),
                        BorderColor(border_color),
                        UpgradeCard(*upgrade_id),
                    )).with_children(|card| {
                        card.spawn((
                            Text::new(data.name),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                        card.spawn((
                            Text::new(data.description),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::srgb(0.85, 0.85, 0.85)),
                        ));
                        card.spawn((
                            Text::new(data.flavor),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::srgb(0.55, 0.55, 0.55)),
                        ));
                    });
                }
            }
        });
    });
}

fn handle_upgrade_click(
    interaction_q: Query<(&Interaction, &UpgradeCard), (Changed<Interaction>, With<Button>)>,
    mut upgrade_state: ResMut<UpgradeState>,
    mut player_q: Query<(&mut PlayerStats, &mut Health), With<Player>>,
    mut game_manager: ResMut<GameManager>,
    mut next_state: ResMut<NextState<GameState>>,
    mut transition_event: EventWriter<RoomTransitionRequest>,
    room_state: Res<RoomState>,
    graph: Res<RoomGraph>,
) {
    for (interaction, card) in &interaction_q {
        if *interaction != Interaction::Pressed { continue }

        let upgrade_id = card.0;
        upgrade_state.owned.push(upgrade_id);

        if let Ok((mut stats, mut health)) = player_q.get_single_mut() {
            // Apply upgrade
            if let Some(data) = get_upgrade(upgrade_id) {
                (data.apply)(&mut stats);
            }

            // Special handling for upgrades that need non-stat changes
            match upgrade_id {
                UpgradeId::MaxHpUp => {
                    health.max += 30.0;
                    health.heal(30.0);
                }
                UpgradeId::RegenOnKill => {
                    game_manager.regen_on_kill += 3.0;
                }
                UpgradeId::DoubleXP => {
                    game_manager.double_xp = true;
                }
                _ => {}
            }
        }

        // Determine next room
        let next_rooms = graph.available_next();
        if next_rooms.is_empty() {
            next_state.set(GameState::Victory);
        } else {
            let next_room = next_rooms[0];
            next_state.set(GameState::Playing);
            transition_event.send(RoomTransitionRequest { destination_room: next_room });
        }
        break;
    }
}

fn hide_upgrade_ui(
    mut commands: Commands,
    q: Query<Entity, With<UpgradeUIRoot>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
