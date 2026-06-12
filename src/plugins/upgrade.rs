use bevy::prelude::*;
use rand::seq::SliceRandom;
use crate::components::*;
use crate::components::xp::{LevelPath, SubPath, XpManager};
use crate::data::upgrades::*;
use crate::events::*;
use crate::resources::*;
use crate::state::GameState;

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(UpgradeState::default())
            .insert_resource(PendingChoice::default())
            .add_systems(OnEnter(GameState::UpgradeSelect), show_choice_ui)
            .add_systems(Update, handle_card_click.run_if(in_state(GameState::UpgradeSelect)))
            .add_systems(OnExit(GameState::UpgradeSelect), hide_choice_ui)
            .add_systems(Update, (
                room_cleared_handler,
                path_branch_handler,
            ).run_if(in_state(GameState::Playing)));
    }
}

/// What the UpgradeSelect screen is currently offering.
#[derive(Resource, Default, Clone, PartialEq)]
pub enum PendingChoice {
    #[default]
    Upgrades,
    Path(u32),
}

#[derive(Clone, Copy)]
enum CardChoice {
    Upgrade(UpgradeId),
    Path(LevelPath),
    SubPath(SubPath),
}

#[derive(Component)]
struct ChoiceUIRoot;

#[derive(Component)]
struct ChoiceCard(CardChoice);

fn room_cleared_handler(
    mut events: EventReader<RoomCleared>,
    mut pending: ResMut<PendingChoice>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _ in events.read() {
        *pending = PendingChoice::Upgrades;
        next_state.set(GameState::UpgradeSelect);
    }
}

/// Level 10/20 path branches interrupt combat with a choice screen.
fn path_branch_handler(
    mut events: EventReader<PathBranchAvailable>,
    mut pending: ResMut<PendingChoice>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for ev in events.read() {
        *pending = PendingChoice::Path(ev.level);
        next_state.set(GameState::UpgradeSelect);
    }
}

struct CardDef {
    choice: CardChoice,
    name: &'static str,
    description: String,
    flavor: &'static str,
    color: (Color, Color), // (bg, border)
}

fn upgrade_card_def(id: UpgradeId) -> Option<CardDef> {
    let data = get_upgrade(id)?;
    let (bg, border) = match data.category {
        UpgradeCategory::Power     => (Color::srgb(0.35, 0.1, 0.1), Color::srgb(0.9, 0.2, 0.2)),
        UpgradeCategory::Defense   => (Color::srgb(0.1, 0.2, 0.35), Color::srgb(0.2, 0.5, 0.9)),
        UpgradeCategory::Utility   => (Color::srgb(0.1, 0.25, 0.1), Color::srgb(0.2, 0.8, 0.3)),
        UpgradeCategory::Character => (Color::srgb(0.25, 0.1, 0.3), Color::srgb(0.7, 0.2, 0.9)),
    };
    Some(CardDef {
        choice: CardChoice::Upgrade(id),
        name: data.name,
        description: data.description.to_string(),
        flavor: data.flavor,
        color: (bg, border),
    })
}

fn path_card_defs(level: u32, chosen_path: Option<LevelPath>) -> Vec<CardDef> {
    let aggressive = (Color::srgb(0.35, 0.1, 0.1), Color::srgb(0.9, 0.2, 0.2));
    let defensive = (Color::srgb(0.1, 0.2, 0.35), Color::srgb(0.2, 0.5, 0.9));
    let utility = (Color::srgb(0.1, 0.25, 0.1), Color::srgb(0.2, 0.8, 0.3));

    if level == 10 {
        vec![
            CardDef {
                choice: CardChoice::Path(LevelPath::Aggressive),
                name: "ESCALATION CLAUSE",
                description: "+30% damage".into(),
                flavor: "\"Resolve the problem with extreme prejudice.\"",
                color: aggressive,
            },
            CardDef {
                choice: CardChoice::Path(LevelPath::Defensive),
                name: "SURVIVAL ADDENDUM",
                description: "+50 max HP, +10% damage reduction".into(),
                flavor: "\"Live contractors file more reports.\"",
                color: defensive,
            },
            CardDef {
                choice: CardChoice::Path(LevelPath::Utility),
                name: "EFFICIENCY RIDER",
                description: "+20% speed, +15% attack rate".into(),
                flavor: "\"Time is money. Your time is our money.\"",
                color: utility,
            },
        ]
    } else {
        match chosen_path {
            Some(LevelPath::Aggressive) => vec![
                CardDef {
                    choice: CardChoice::SubPath(SubPath::AggressiveA),
                    name: "TOTAL WAR DOCTRINE",
                    description: "+50% damage".into(),
                    flavor: "\"Collateral is a rounding error.\"",
                    color: aggressive,
                },
                CardDef {
                    choice: CardChoice::SubPath(SubPath::AggressiveB),
                    name: "RAPID RESPONSE UNIT",
                    description: "+40% attack rate".into(),
                    flavor: "\"Faster. The bugs aren't waiting.\"",
                    color: aggressive,
                },
            ],
            Some(LevelPath::Defensive) => vec![
                CardDef {
                    choice: CardChoice::SubPath(SubPath::DefensiveA),
                    name: "FORTIFICATION GRANT",
                    description: "+100 max HP".into(),
                    flavor: "\"A bigger meat shield is still a shield.\"",
                    color: defensive,
                },
                CardDef {
                    choice: CardChoice::SubPath(SubPath::DefensiveB),
                    name: "FULL COVERAGE PLAN",
                    description: "+15% damage reduction, full heal".into(),
                    flavor: "\"Premium tier. Someone else is paying.\"",
                    color: defensive,
                },
            ],
            _ => vec![
                CardDef {
                    choice: CardChoice::SubPath(SubPath::UtilityA),
                    name: "EXPEDITED LOGISTICS",
                    description: "+30% movement speed".into(),
                    flavor: "\"Outrun the problem. File later.\"",
                    color: utility,
                },
                CardDef {
                    choice: CardChoice::SubPath(SubPath::UtilityB),
                    name: "PROCESS OPTIMIZATION",
                    description: "+25% attack rate, +10% damage".into(),
                    flavor: "\"Synergy. Whatever that means.\"",
                    color: utility,
                },
            ],
        }
    }
}

fn show_choice_ui(
    mut commands: Commands,
    pending: Res<PendingChoice>,
    upgrade_state: Res<UpgradeState>,
    xp_mgr: Res<XpManager>,
    mut rng: ResMut<GameRng>,
) {
    let (title, subtitle, cards) = match *pending {
        PendingChoice::Upgrades => {
            let pool: Vec<UpgradeId> = general_upgrades()
                .into_iter()
                .filter(|id| !upgrade_state.owned.contains(id))
                .collect();
            let mut pool = if pool.is_empty() { general_upgrades() } else { pool };
            pool.shuffle(&mut rng.0);
            let cards: Vec<CardDef> = pool.iter().take(3).filter_map(|&id| upgrade_card_def(id)).collect();
            ("PERFORMANCE REVIEW", "Choose 1 of 3 upgrades", cards)
        }
        PendingChoice::Path(level) => {
            let cards = path_card_defs(level, xp_mgr.level_path);
            ("CAREER ADVANCEMENT", "Choose your path. This is permanent.", cards)
        }
    };

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
        ChoiceUIRoot,
    )).with_children(|parent| {
        parent.spawn((
            Text::new(title),
            TextFont { font_size: 32.0, ..default() },
            TextColor(Color::srgb(1.0, 0.7, 0.1)),
        ));
        parent.spawn((
            Text::new(subtitle),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));

        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(24.0),
            ..default()
        }).with_children(|row| {
            for card in &cards {
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
                    BackgroundColor(card.color.0),
                    BorderColor(card.color.1),
                    ChoiceCard(card.choice),
                )).with_children(|c| {
                    c.spawn((
                        Text::new(card.name),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                    c.spawn((
                        Text::new(card.description.clone()),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgb(0.85, 0.85, 0.85)),
                    ));
                    c.spawn((
                        Text::new(card.flavor),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(Color::srgb(0.55, 0.55, 0.55)),
                    ));
                });
            }
        });
    });
}

fn handle_card_click(
    interaction_q: Query<(&Interaction, &ChoiceCard), (Changed<Interaction>, With<Button>)>,
    mut upgrade_state: ResMut<UpgradeState>,
    mut xp_mgr: ResMut<XpManager>,
    mut player_q: Query<(&mut PlayerStats, &mut Health), With<Player>>,
    mut game_manager: ResMut<GameManager>,
    mut next_state: ResMut<NextState<GameState>>,
    mut transition_event: EventWriter<RoomTransitionRequest>,
    pending: Res<PendingChoice>,
    graph: Res<RoomGraph>,
) {
    for (interaction, card) in &interaction_q {
        if *interaction != Interaction::Pressed { continue }
        let Ok((mut stats, mut health)) = player_q.get_single_mut() else { return };

        match card.0 {
            CardChoice::Upgrade(upgrade_id) => {
                upgrade_state.owned.push(upgrade_id);
                if let Some(data) = get_upgrade(upgrade_id) {
                    (data.apply)(&mut stats);
                }
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

                // Advance the run: victory at terminal node, map choice on
                // branches, direct transition on a single path.
                let next_rooms = graph.available_next();
                match next_rooms.len() {
                    0 => next_state.set(GameState::Victory),
                    1 => {
                        transition_event.send(RoomTransitionRequest { destination_room: next_rooms[0] });
                        next_state.set(GameState::Playing);
                    }
                    _ => next_state.set(GameState::MapSelect),
                }
            }
            CardChoice::Path(path) => {
                xp_mgr.level_path = Some(path);
                match path {
                    LevelPath::Aggressive => { stats.damage_multiplier *= 1.3; }
                    LevelPath::Defensive => {
                        health.max += 50.0;
                        health.heal(50.0);
                        stats.damage_reduction = (stats.damage_reduction + 0.10).min(0.8);
                    }
                    LevelPath::Utility => {
                        stats.base_speed *= 1.2;
                        stats.attack_rate *= 1.15;
                    }
                }
                // Path choices happen mid-combat: resume, no room transition
                next_state.set(GameState::Playing);
            }
            CardChoice::SubPath(sub) => {
                xp_mgr.sub_path = Some(sub);
                match sub {
                    SubPath::AggressiveA => { stats.damage_multiplier *= 1.5; }
                    SubPath::AggressiveB => { stats.attack_rate *= 1.4; }
                    SubPath::DefensiveA => {
                        health.max += 100.0;
                        health.heal(100.0);
                    }
                    SubPath::DefensiveB => {
                        stats.damage_reduction = (stats.damage_reduction + 0.15).min(0.8);
                        let max = health.max;
                        health.heal(max);
                    }
                    SubPath::UtilityA => { stats.base_speed *= 1.3; }
                    SubPath::UtilityB => {
                        stats.attack_rate *= 1.25;
                        stats.damage_multiplier *= 1.1;
                    }
                }
                next_state.set(GameState::Playing);
            }
        }

        let _ = pending; // PendingChoice is reset by the next handler that sets it
        break;
    }
}

fn hide_choice_ui(
    mut commands: Commands,
    q: Query<Entity, With<ChoiceUIRoot>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
