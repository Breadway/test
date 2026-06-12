use bevy::prelude::*;
use crate::components::*;
use crate::components::player::CharacterType;
use crate::state::GameState;

pub struct HeavyPlugin;

impl Plugin for HeavyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            heavy_foam_spray,
            heavy_armor_visual,
        ).run_if(in_state(GameState::Playing)));
    }
}

const FOAM_DAMAGE: f32 = 6.0;
const FOAM_ARMOR_PER_SPRAY: f32 = 15.0;

/// Foam spray: short cone that damages and slows enemies, and coats Heavy in
/// ablative armor. Armor absorption happens in `player_damage_handler`.
fn heavy_foam_spray(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mut player_q: Query<(&Transform, &Facing, &mut PlayerStats, &mut AttackCooldown, Option<&Stunned>), With<Player>>,
    mut enemy_q: Query<(&Transform, &mut Health, &mut Velocity), (With<Enemy>, Without<Dead>, Without<Player>)>,
) {
    let Ok((player_t, facing, mut stats, mut cooldown, stunned)) = player_q.get_single_mut() else { return };
    if stats.character_type != CharacterType::Heavy { return }
    if cooldown.0 > 0.0 || stunned.is_some() { return }
    if !buttons.pressed(MouseButton::Left) { return }

    let pp = player_t.translation.truncate();
    let aim = if facing.0 != Vec2::ZERO { facing.0 } else { Vec2::X };
    let reach = stats.projectile_range;
    let half_arc_cos = (45.0f32).to_radians().cos();

    for (enemy_t, mut health, mut vel) in &mut enemy_q {
        let to_enemy = enemy_t.translation.truncate() - pp;
        let dist = to_enemy.length();
        if dist > reach || dist < 1.0 { continue }
        if to_enemy.normalize().dot(aim) < half_arc_cos { continue }

        health.take_damage(FOAM_DAMAGE * stats.damage_multiplier);
        vel.0 *= 0.4; // foam bogs enemies down
    }

    stats.foam_armor = (stats.foam_armor + FOAM_ARMOR_PER_SPRAY).min(stats.foam_armor_max);

    // Spray visual
    let vfx_pos = pp + aim * reach * 0.5;
    commands.spawn((
        Sprite {
            color: Color::srgba(0.7, 0.85, 0.7, 0.5),
            custom_size: Some(Vec2::new(reach * 0.8, 40.0)),
            ..default()
        },
        Transform::from_xyz(vfx_pos.x, vfx_pos.y, 6.0)
            .with_rotation(Quat::from_rotation_z(aim.y.atan2(aim.x))),
        FoamVfx(0.15),
    ));

    cooldown.0 = 1.0 / stats.attack_rate;
}

#[derive(Component)]
struct FoamVfx(f32);

fn heavy_armor_visual(
    time: Res<Time>,
    mut commands: Commands,
    mut player_q: Query<(&PlayerStats, &mut Sprite), With<Player>>,
    mut vfx_q: Query<(Entity, &mut FoamVfx, &mut Sprite), Without<Player>>,
) {
    let dt = time.delta_secs();

    for (stats, mut sprite) in player_q.iter_mut() {
        if stats.character_type != CharacterType::Heavy { continue }
        let t = (stats.foam_armor / stats.foam_armor_max).min(1.0);
        sprite.color = Color::srgb(0.4 + t * 0.4, 0.5 + t * 0.2, 0.4 + t * 0.1);
    }

    for (entity, mut vfx, mut sprite) in &mut vfx_q {
        vfx.0 -= dt;
        if vfx.0 <= 0.0 {
            commands.entity(entity).despawn_recursive();
        } else {
            sprite.color = sprite.color.with_alpha((vfx.0 / 0.15) * 0.5);
        }
    }
}
