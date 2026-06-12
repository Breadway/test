use bevy::prelude::*;
use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::state::GameState;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), spawn_hud.after(crate::plugins::room::setup_room_graph))
            .add_systems(OnExit(GameState::Playing), despawn_hud)
            .add_systems(Update, (
                update_health_bar,
                update_xp_bar,
                update_level_display,
                update_kill_counter,
                update_boss_bar,
                show_room_toast,
                tick_room_toast,
            ).run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(GameState::MainMenu), despawn_main_menu)
            .add_systems(OnEnter(GameState::CharacterSelect), spawn_char_select)
            .add_systems(OnExit(GameState::CharacterSelect), despawn_char_select)
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), despawn_game_over)
            .add_systems(OnEnter(GameState::Victory), spawn_victory)
            .add_systems(OnExit(GameState::Victory), despawn_victory)
            .add_systems(Update, (
                main_menu_interaction,
                char_select_interaction,
                game_over_interaction,
            ));
    }
}

// ---- HUD markers ----
#[derive(Component)] struct HudRoot;
#[derive(Component)] struct HealthBarFill;
#[derive(Component)] struct XpBarFill;
#[derive(Component)] struct LevelText;
#[derive(Component)] struct KillCountText;
#[derive(Component)] struct RoomNameText;
#[derive(Component)] struct BossBarRoot;
#[derive(Component)] struct BossBarFill;

fn spawn_hud(mut commands: Commands, graph: Res<RoomGraph>) {
    let room_name = graph.nodes.get(0).map(|n| n.lore_name.clone()).unwrap_or_default();

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        HudRoot,
    )).with_children(|hud| {
        // Top row: HP + XP
        hud.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        }).with_children(|top| {
            // Health bar background
            top.spawn((
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(18.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.05, 0.05)),
                BorderColor(Color::srgb(0.6, 0.1, 0.1)),
            )).with_children(|bar| {
                bar.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.85, 0.15, 0.15)),
                    HealthBarFill,
                ));
            });

            // XP bar background
            top.spawn((
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(10.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.05, 0.05, 0.15)),
                BorderColor(Color::srgb(0.1, 0.1, 0.6)),
            )).with_children(|bar| {
                bar.spawn((
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.4, 1.0)),
                    XpBarFill,
                ));
            });

            // Level text
            top.spawn((
                Text::new("Lv. 1"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.9)),
                LevelText,
            ));
        });

        // Kill counter (top right)
        hud.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                ..default()
            },
        )).with_children(|corner| {
            corner.spawn((
                Text::new("Kills: 0"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.6, 0.8, 0.6)),
                KillCountText,
            ));
        });

        // Boss HP bar (bottom center, hidden until a boss exists)
        hud.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(28.0),
                left: Val::Percent(25.0),
                width: Val::Percent(50.0),
                height: Val::Px(20.0),
                border: UiRect::all(Val::Px(2.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.02, 0.02)),
            BorderColor(Color::srgb(0.7, 0.1, 0.1)),
            BossBarRoot,
        )).with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.85, 0.1, 0.1)),
                BossBarFill,
            ));
        });

        // Room name toast (top center)
        hud.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(30.0),
                left: Val::Percent(50.0),
                ..default()
            },
        )).with_children(|center| {
            center.spawn((
                Text::new(room_name),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgba(0.7, 0.7, 0.5, 1.0)),
                RoomNameText,
                crate::plugins::room::RoomToastText { timer: 3.0 },
            ));
        });
    });
}

fn despawn_hud(mut commands: Commands, q: Query<Entity, With<HudRoot>>) {
    for e in &q { commands.entity(e).despawn_recursive(); }
}

fn update_health_bar(
    player_q: Query<&Health, With<Player>>,
    mut bar_q: Query<&mut Node, With<HealthBarFill>>,
) {
    let Ok(health) = player_q.get_single() else { return };
    for mut node in &mut bar_q {
        node.width = Val::Percent(health.fraction() * 100.0);
    }
}

fn update_xp_bar(
    xp_mgr: Res<XpManager>,
    mut bar_q: Query<&mut Node, With<XpBarFill>>,
) {
    if xp_mgr.xp_to_next == 0 { return }
    let frac = (xp_mgr.xp_this_level as f32 / xp_mgr.xp_to_next as f32).min(1.0) * 100.0;
    for mut node in &mut bar_q {
        node.width = Val::Percent(frac);
    }
}

fn update_level_display(
    xp_mgr: Res<XpManager>,
    mut q: Query<&mut Text, With<LevelText>>,
) {
    if !xp_mgr.is_changed() { return }
    for mut text in &mut q {
        **text = format!("Lv. {}  [{}/{}]", xp_mgr.current_level, xp_mgr.xp_this_level, xp_mgr.xp_to_next);
    }
}

fn update_boss_bar(
    boss_q: Query<&Health, With<crate::components::enemy::Boss>>,
    mut root_q: Query<&mut Node, (With<BossBarRoot>, Without<BossBarFill>)>,
    mut fill_q: Query<&mut Node, With<BossBarFill>>,
) {
    let boss_health = boss_q.get_single().ok();
    for mut node in &mut root_q {
        node.display = if boss_health.is_some() { Display::Flex } else { Display::None };
    }
    if let Some(health) = boss_health {
        for mut node in &mut fill_q {
            node.width = Val::Percent(health.fraction() * 100.0);
        }
    }
}

fn update_kill_counter(
    gm: Res<GameManager>,
    mut q: Query<&mut Text, With<KillCountText>>,
) {
    if !gm.is_changed() { return }
    for mut text in &mut q {
        **text = format!("Kills: {}  Room: {}", gm.total_kills, gm.rooms_cleared + 1);
    }
}

fn show_room_toast(
    mut events: EventReader<RoomTransitionRequest>,
    graph: Res<RoomGraph>,
    mut text_q: Query<(&mut Text, &mut TextColor, &mut crate::plugins::room::RoomToastText)>,
) {
    for ev in events.read() {
        if let Some(node) = graph.nodes.get(ev.destination_room) {
            for (mut text, mut color, mut toast) in &mut text_q {
                **text = node.lore_name.clone();
                *color = TextColor(Color::srgba(0.7, 0.7, 0.5, 1.0));
                toast.timer = 3.0;
            }
        }
    }
}

fn tick_room_toast(
    time: Res<Time>,
    mut q: Query<(&mut TextColor, &mut crate::plugins::room::RoomToastText)>,
) {
    let dt = time.delta_secs();
    for (mut color, mut toast) in &mut q {
        if toast.timer > 0.0 {
            toast.timer -= dt;
            let alpha = (toast.timer / 1.5).min(1.0);
            let TextColor(c) = color.as_mut();
            *c = Color::srgba(0.7, 0.7, 0.5, alpha);
        }
    }
}

// ---- Main Menu ----
#[derive(Component)] struct MainMenuRoot;
#[derive(Component)] struct PlayButton;

fn spawn_main_menu(mut commands: Commands) {
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
        BackgroundColor(Color::srgb(0.05, 0.05, 0.08)),
        MainMenuRoot,
    )).with_children(|menu| {
        menu.spawn((
            Text::new("HAZARD PAY"),
            TextFont { font_size: 64.0, ..default() },
            TextColor(Color::srgb(1.0, 0.45, 0.1)),
        ));
        menu.spawn((
            Text::new("A Biosphere Management Group Initiative"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
        ));
        menu.spawn((
            Text::new("Contractor mortality: 67% (first 30 days)"),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::srgb(0.4, 0.4, 0.4)),
        ));
        menu.spawn((
            Button,
            Node {
                width: Val::Px(220.0),
                height: Val::Px(52.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.18, 0.18, 0.18)),
            BorderColor(Color::srgb(1.0, 0.45, 0.1)),
            PlayButton,
        )).with_children(|btn| {
            btn.spawn((
                Text::new("ACCEPT CONTRACT"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(1.0, 0.7, 0.3)),
            ));
        });

        menu.spawn((
            Text::new("WASD / Arrow Keys to move  •  Mouse to aim  •  Left Click to shoot"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.35, 0.35, 0.35)),
        ));
    });
}

fn despawn_main_menu(mut commands: Commands, q: Query<Entity, With<MainMenuRoot>>) {
    for e in &q { commands.entity(e).despawn_recursive(); }
}

fn main_menu_interaction(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::CharacterSelect);
        }
    }
}

// ---- Character Select ----
#[derive(Component)] struct CharSelectRoot;
#[derive(Component)] struct CharSelectButton(crate::components::player::CharacterType);

fn spawn_char_select(mut commands: Commands, mut rng: ResMut<GameRng>) {
    use crate::components::player::CharacterType::*;
    use rand::Rng;
    use crate::data::lore::LOADING_TIPS;
    let tip = LOADING_TIPS[rng.0.gen_range(0..LOADING_TIPS.len())];
    let chars = [
        (Torch,  Color::srgb(1.0, 0.4, 0.1),  "TORCH",   "Pyrotechnic",  "Fire spreads to all nearby enemies.", "Takes damage in own flames."),
        (QueenB, Color::srgb(1.0, 0.85, 0.1),  "QUEEN B", "Apiarist",     "Dead bugs become ally bees.",         "Cannot deal direct damage."),
        (Mop,    Color::srgb(0.4, 0.7, 1.0),   "MOP",     "Custodian",    "Max range hits deal 3x damage.",      "Zero damage at close range."),
        (Heavy,  Color::srgb(0.4, 0.5, 0.4),   "HEAVY",   "Intern",       "Foam armor absorbs hits.",            "More armor = much slower."),
        (Sparky, Color::srgb(0.9, 0.95, 0.2),  "SPARKY",  "Electrician",  "Lightning chains between enemies.",   "100% heat = forced discharge."),
        (Doc,    Color::srgb(0.7, 0.2, 0.9),   "DOC",     "Scientist",    "Charge beam: massive single damage.",  "Overcharge = self-damage."),
    ];

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
        BackgroundColor(Color::srgb(0.05, 0.05, 0.08)),
        CharSelectRoot,
    )).with_children(|root| {
        root.spawn((
            Text::new("SELECT CONTRACTOR"),
            TextFont { font_size: 32.0, ..default() },
            TextColor(Color::srgb(1.0, 0.7, 0.2)),
        ));

        root.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(16.0),
                row_gap: Val::Px(16.0),
                max_width: Val::Px(1100.0),
                ..default()
            },
        )).with_children(|grid| {
            for (ct, color, name, role, power, constraint) in &chars {
                grid.spawn((
                    Button,
                    Node {
                        width: Val::Px(165.0),
                        min_height: Val::Px(200.0),
                        padding: UiRect::all(Val::Px(14.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(color.to_srgba().red * 0.15, color.to_srgba().green * 0.15, color.to_srgba().blue * 0.15, 1.0)),
                    BorderColor(*color),
                    CharSelectButton(*ct),
                )).with_children(|card| {
                    card.spawn((
                        Text::new(*name),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(*color),
                    ));
                    card.spawn((
                        Text::new(*role),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                    card.spawn((
                        Text::new(format!("⚡ {}", power)),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(Color::srgb(0.85, 0.85, 0.85)),
                    ));
                    card.spawn((
                        Text::new(format!("⚠ {}", constraint)),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.4, 0.4)),
                    ));
                });
            }
        });

        root.spawn((
            Text::new(tip),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.45, 0.45, 0.4)),
        ));
    });
}

fn despawn_char_select(mut commands: Commands, q: Query<Entity, With<CharSelectRoot>>) {
    for e in &q { commands.entity(e).despawn_recursive(); }
}

fn char_select_interaction(
    interaction_q: Query<(&Interaction, &CharSelectButton), (Changed<Interaction>, With<Button>)>,
    mut selection: ResMut<CharacterSelection>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed {
            selection.character = btn.0;
            next_state.set(GameState::Playing);
        }
    }
}

// ---- Game Over ----
#[derive(Component)] struct GameOverRoot;
#[derive(Component)] struct RestartButton;

fn spawn_game_over(
    mut commands: Commands,
    game_manager: Res<GameManager>,
    mut rng: ResMut<GameRng>,
) {
    use rand::Rng;
    use crate::data::lore::{DEATH_MESSAGES, LOADING_TIPS};
    let msg = DEATH_MESSAGES[rng.0.gen_range(0..DEATH_MESSAGES.len())];
    let tip = LOADING_TIPS[rng.0.gen_range(0..LOADING_TIPS.len())];

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
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
        GameOverRoot,
    )).with_children(|root| {
        root.spawn((
            Text::new("CONTRACT TERMINATED"),
            TextFont { font_size: 40.0, ..default() },
            TextColor(Color::srgb(0.8, 0.1, 0.1)),
        ));
        root.spawn((
            Text::new(msg),
            TextFont { font_size: 15.0, ..default() },
            TextColor(Color::srgb(0.55, 0.55, 0.55)),
        ));
        root.spawn((
            Text::new(format!("TIP: {}", tip)),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.45, 0.45, 0.4)),
        ));
        root.spawn((
            Text::new(format!(
                "Rooms Cleared: {}  |  Enemies Killed: {}",
                game_manager.rooms_cleared, game_manager.total_kills
            )),
            TextFont { font_size: 17.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
        root.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            BorderColor(Color::srgb(0.6, 0.1, 0.1)),
            RestartButton,
        )).with_children(|btn| {
            btn.spawn((
                Text::new("TRY AGAIN"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.9, 0.4, 0.4)),
            ));
        });
    });
}

fn despawn_game_over(mut commands: Commands, q: Query<Entity, With<GameOverRoot>>) {
    for e in &q { commands.entity(e).despawn_recursive(); }
}

fn game_over_interaction(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_manager: ResMut<GameManager>,
    mut xp_mgr: ResMut<XpManager>,
    mut upgrade_state: ResMut<UpgradeState>,
    mut spawn_queue: ResMut<SpawnQueue>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            // Reset run state
            *game_manager = GameManager::default();
            *xp_mgr = XpManager::new();
            *upgrade_state = UpgradeState::default();
            spawn_queue.pending.clear();
            next_state.set(GameState::CharacterSelect);
        }
    }
}

// ---- Victory ----
#[derive(Component)] struct VictoryRoot;

fn spawn_victory(
    mut commands: Commands,
    game_manager: Res<GameManager>,
) {
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
        BackgroundColor(Color::srgba(0.0, 0.05, 0.0, 0.92)),
        VictoryRoot,
    )).with_children(|root| {
        root.spawn((
            Text::new("SECTOR CLEARED"),
            TextFont { font_size: 48.0, ..default() },
            TextColor(Color::srgb(0.3, 1.0, 0.3)),
        ));
        root.spawn((
            Text::new("\"Outstanding performance. Bonus: $0.00\""),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
        ));
        root.spawn((
            Text::new(format!(
                "Final Stats  |  Rooms: {}  |  Kills: {}",
                game_manager.rooms_cleared, game_manager.total_kills
            )),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.7, 0.9, 0.7)),
        ));
        root.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.18, 0.1)),
            BorderColor(Color::srgb(0.2, 0.8, 0.2)),
            RestartButton,
        )).with_children(|btn| {
            btn.spawn((
                Text::new("PLAY AGAIN"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.3, 1.0, 0.3)),
            ));
        });
    });
}

fn despawn_victory(mut commands: Commands, q: Query<Entity, With<VictoryRoot>>) {
    for e in &q { commands.entity(e).despawn_recursive(); }
}
