use bevy::prelude::*;
use crate::components::*;
use crate::components::player::CharacterType;
use crate::events::*;
use crate::state::GameState;

pub struct HeavyPlugin;

impl Plugin for HeavyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            heavy_foam_spray,
            heavy_armor_absorb,
            heavy_armor_visual,
        ).run_if(in_state(GameState::Playing)));
    }
}

fn heavy_foam_spray(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut player_q: Query<(&mut PlayerStats, &mut AttackCooldown), With<Player>>,
    enemy_q: Query<(&Transform, &mut Health), With<Enemy>>,
) {
    let Ok((mut stats, cooldown)) = player_q.get_single_mut() else { return };
    if stats.character_type != CharacterType::Heavy { return }

    if buttons.pressed(MouseButton::Left) && cooldown.0 <= 0.0 {
        let gained = 15.0;
        stats.foam_armor = (stats.foam_armor + gained).min(stats.foam_armor_max);
    }
}

fn heavy_armor_absorb(
    mut events: EventReader<PlayerDamaged>,
    mut player_q: Query<(&mut PlayerStats, &mut Health), With<Player>>,
) {
    let Ok((mut stats, mut health)) = player_q.get_single_mut() else { return };
    if stats.character_type != CharacterType::Heavy { return }

    for ev in events.read() {
        if stats.foam_armor > 0.0 {
            let absorbed = ev.amount.min(stats.foam_armor);
            stats.foam_armor -= absorbed;
            let remainder = ev.amount - absorbed;
            if remainder > 0.0 {
                health.take_damage(remainder);
            }
        }
    }
}

fn heavy_armor_visual(
    mut player_q: Query<(&PlayerStats, &mut Sprite), With<Player>>,
) {
    for (stats, mut sprite) in player_q.iter_mut() {
        if stats.character_type != CharacterType::Heavy { return }
        let t = (stats.foam_armor / stats.foam_armor_max).min(1.0);
        sprite.color = Color::srgb(0.4 + t * 0.4, 0.5 + t * 0.2, 0.4 + t * 0.1);
    }
}
