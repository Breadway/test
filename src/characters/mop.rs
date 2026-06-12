use bevy::prelude::*;
use crate::components::*;
use crate::components::player::CharacterType;
use crate::state::GameState;

pub struct MopPlugin;

impl Plugin for MopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            mop_melee_attack,
            swing_vfx_fade,
        ).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
struct SwingVfx(f32);

/// Melee swing in a ±60° arc. Distance-tiered damage:
/// inside 30% of reach = 0x ("Safety Violation"),
/// beyond 75% of reach = 3x ("Sanitize"), otherwise 1x.
fn mop_melee_attack(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mut player_q: Query<(&Transform, &Facing, &PlayerStats, &mut AttackCooldown, Option<&Stunned>), With<Player>>,
    mut enemy_q: Query<(&Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
) {
    let Ok((player_t, facing, stats, mut cooldown, stunned)) = player_q.get_single_mut() else { return };
    if stats.character_type != CharacterType::Mop { return }
    if cooldown.0 > 0.0 || stunned.is_some() { return }
    if !buttons.pressed(MouseButton::Left) { return }

    let pp = player_t.translation.truncate();
    let aim = if facing.0 != Vec2::ZERO { facing.0 } else { Vec2::X };
    let reach = stats.projectile_range;
    let min_range = reach * 0.3;
    let sweet_spot = reach * 0.75;
    let half_arc_cos = (60.0f32).to_radians().cos();

    for (enemy_t, mut health) in &mut enemy_q {
        let to_enemy = enemy_t.translation.truncate() - pp;
        let dist = to_enemy.length();
        if dist > reach || dist < 1.0 { continue }
        if to_enemy.normalize().dot(aim) < half_arc_cos { continue }

        let multiplier = if dist < min_range {
            0.0
        } else if dist > sweet_spot {
            3.0
        } else {
            1.0
        };
        let damage = stats.attack_damage * stats.damage_multiplier * multiplier;
        if damage > 0.0 {
            health.take_damage(damage);
        }
    }

    // Swing visual: brief arc-shaped sprite in the aim direction
    let vfx_pos = pp + aim * reach * 0.6;
    commands.spawn((
        Sprite {
            color: Color::srgba(0.8, 0.9, 1.0, 0.7),
            custom_size: Some(Vec2::new(reach * 0.9, 14.0)),
            ..default()
        },
        Transform::from_xyz(vfx_pos.x, vfx_pos.y, 6.0)
            .with_rotation(Quat::from_rotation_z(aim.y.atan2(aim.x))),
        SwingVfx(0.12),
    ));

    cooldown.0 = 1.0 / stats.attack_rate;
}

fn swing_vfx_fade(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut SwingVfx, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut vfx, mut sprite) in &mut q {
        vfx.0 -= dt;
        if vfx.0 <= 0.0 {
            commands.entity(entity).despawn_recursive();
        } else {
            sprite.color = sprite.color.with_alpha((vfx.0 / 0.12) * 0.7);
        }
    }
}
