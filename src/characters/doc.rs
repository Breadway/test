use bevy::prelude::*;
use crate::components::*;
use crate::components::player::CharacterType;
use crate::events::*;
use crate::resources::*;
use crate::state::GameState;
use rand::Rng;

pub struct DocPlugin;

impl Plugin for DocPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            doc_charge_system,
        ).run_if(in_state(GameState::Playing)));
    }
}

fn doc_charge_system(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut rng: ResMut<GameRng>,
    sprites: Res<crate::data::pixel_art::GameSprites>,
    mut player_q: Query<(&Transform, &Facing, &mut PlayerStats, &mut AttackCooldown, Option<&Stunned>), With<Player>>,
    mut player_damage: EventWriter<PlayerDamaged>,
) {
    let Ok((transform, facing, mut stats, mut cooldown, stunned)) = player_q.get_single_mut() else { return };
    if stats.character_type != CharacterType::Doc { return }

    if stunned.is_some() {
        stats.charging = false;
        stats.charge = 0.0;
        return;
    }

    if buttons.pressed(MouseButton::Left) {
        stats.charging = true;
        stats.charge = (stats.charge + 60.0 * time.delta_secs()).min(stats.charge_max);

        // Critical instability: >75% charge gambles 5%/sec on a misfire
        if stats.charge > stats.charge_max * 0.75 {
            let chance = 0.05 * time.delta_secs();
            let roll: f32 = rng.0.gen();
            if roll < chance {
                player_damage.send(PlayerDamaged {
                    amount: 20.0,
                    source: DamageSource::Electric,
                });
                stats.charge = 0.0;
                stats.charging = false;
            }
        }
    } else if stats.charging && cooldown.0 <= 0.0 {
        stats.charging = false;
        let charge_frac = stats.charge / stats.charge_max;
        let damage = stats.attack_damage * charge_frac * charge_frac * stats.damage_multiplier;
        stats.charge = 0.0;

        if damage > 1.0 {
            let dir = if facing.0 != Vec2::ZERO { facing.0 } else { Vec2::X };
            let origin = transform.translation.truncate() + dir * 18.0;

            commands.spawn((
                Sprite {
                    image: sprites.bullet.clone(),
                    color: Color::srgb(0.8, 0.3, 1.0),
                    custom_size: Some(Vec2::new(14.0, 6.0)),
                    ..default()
                },
                Transform::from_xyz(origin.x, origin.y, 5.0)
                    .with_rotation(Quat::from_rotation_z(dir.y.atan2(dir.x))),
                Projectile {
                    damage,
                    lifetime: stats.projectile_range / stats.projectile_speed,
                    pierce: 5,
                    damage_multiplier: 1.0,
                    hit_entities: Vec::new(),
                },
                Velocity(dir * stats.projectile_speed),
            ));

            cooldown.0 = 1.0 / stats.attack_rate;
        }
    }
}
