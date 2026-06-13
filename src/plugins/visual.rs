use bevy::prelude::*;
use crate::components::*;
use crate::data::pixel_art::GameSprites;
use crate::state::GameState;

pub struct VisualPlugin;

impl Plugin for VisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_enemy_hp_bars,
                enemy_flash_tick,
                update_impact_particles,
                lore_doc_pulse,
                player_direction_flip,
            )
            .run_if(in_state(GameState::Playing)),
        );
    }
}

// ── Components ────────────────────────────────────────────────────────────────

/// Brief white-flash effect on a damaged enemy.
#[derive(Component)]
pub struct EnemyFlash {
    pub timer: f32,
    pub original_color: Color,
}

/// Marker for the HP bar fill child sprite.
#[derive(Component)]
pub struct EnemyHpBarFill;

/// Tiny spark pixel emitted on projectile impact.
#[derive(Component)]
pub struct ImpactParticle {
    pub vel: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

// ── Constants ─────────────────────────────────────────────────────────────────

pub const HP_BAR_W: f32 = 22.0;
const HP_BAR_H: f32 = 3.0;

/// Spawn a two-child HP bar (bg + fill) relative to the enemy sprite.
/// Call from `spawn_enemy` with the entity builder.
pub fn attach_hp_bar(builder: &mut ChildBuilder, sprite_half_h: f32) {
    let y = sprite_half_h + 5.0;

    // Background
    builder.spawn((
        Sprite {
            color: Color::srgb(0.15, 0.0, 0.0),
            custom_size: Some(Vec2::new(HP_BAR_W, HP_BAR_H)),
            ..default()
        },
        Transform::from_xyz(0.0, y, 0.2),
    ));

    // Fill (starts full)
    builder.spawn((
        Sprite {
            color: Color::srgb(0.85, 0.1, 0.1),
            custom_size: Some(Vec2::new(HP_BAR_W, HP_BAR_H)),
            ..default()
        },
        Transform::from_xyz(0.0, y, 0.3),
        EnemyHpBarFill,
    ));
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn update_enemy_hp_bars(
    enemy_q: Query<(&Health, &Children), (With<Enemy>, Without<Dead>)>,
    mut fill_q: Query<(&mut Sprite, &mut Transform), With<EnemyHpBarFill>>,
) {
    for (health, children) in &enemy_q {
        for &child in children.iter() {
            let Ok((mut sprite, mut transform)) = fill_q.get_mut(child) else { continue };
            let frac = health.fraction().clamp(0.0, 1.0);
            let w = (HP_BAR_W * frac).max(0.5);
            sprite.custom_size = Some(Vec2::new(w, HP_BAR_H));
            // Left-align: shift left so the bar shrinks from the right
            transform.translation.x = -(HP_BAR_W - w) * 0.5;
            // Color: green → yellow → red
            sprite.color = if frac > 0.6 {
                Color::srgb(0.1 + (1.0 - frac) * 1.3, 0.8, 0.1)
            } else if frac > 0.3 {
                Color::srgb(0.9, 0.7, 0.1)
            } else {
                Color::srgb(0.9, 0.1, 0.1)
            };
        }
    }
}

pub fn enemy_flash_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut EnemyFlash, &mut Sprite)>,
) {
    use bevy::color::Mix;
    let dt = time.delta_secs();
    for (entity, mut flash, mut sprite) in &mut q {
        flash.timer -= dt;
        if flash.timer <= 0.0 {
            sprite.color = flash.original_color;
            commands.entity(entity).remove::<EnemyFlash>();
        } else {
            let t = (flash.timer / 0.12).clamp(0.0, 1.0);
            sprite.color = Color::WHITE.mix(&flash.original_color, 1.0 - t);
        }
    }
}

pub fn update_impact_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut ImpactParticle, &mut Transform, &mut Sprite)>,
) {
    use bevy::color::Alpha;
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform, mut sprite) in &mut q {
        particle.lifetime -= dt;
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        } else {
            transform.translation.x += particle.vel.x * dt;
            transform.translation.y += particle.vel.y * dt;
            particle.vel *= (1.0 - dt * 5.0).max(0.0);
            let frac = particle.lifetime / particle.max_lifetime;
            sprite.color.set_alpha(frac * frac);
        }
    }
}

fn lore_doc_pulse(
    time: Res<Time>,
    mut q: Query<&mut Transform, With<crate::plugins::room::LoreDoc>>,
) {
    let s = (time.elapsed_secs() * 2.5).sin() * 0.1 + 1.0;
    for mut t in &mut q {
        t.scale = Vec3::splat(s);
    }
}

fn player_direction_flip(
    mut q: Query<(&Facing, &mut Sprite), With<Player>>,
) {
    for (facing, mut sprite) in &mut q {
        if facing.0.x < -0.05 {
            sprite.flip_x = true;
        } else if facing.0.x > 0.05 {
            sprite.flip_x = false;
        }
    }
}

/// Spawn 6 spark particles radiating outward from `pos`.
pub fn spawn_sparks(
    commands: &mut Commands,
    sprites: &GameSprites,
    pos: Vec2,
    base_color: Color,
) {
    use std::f32::consts::TAU;
    for k in 0..6 {
        let angle = k as f32 * TAU / 6.0;
        let speed = 90.0 + (k as f32 * 17.0) % 60.0;
        let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
        commands.spawn((
            Sprite {
                image: sprites.spark.clone(),
                color: base_color,
                custom_size: Some(Vec2::splat(3.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 7.0),
            ImpactParticle {
                vel,
                lifetime: 0.22,
                max_lifetime: 0.22,
            },
        ));
    }
}
