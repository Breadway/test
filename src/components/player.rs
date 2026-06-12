use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component, Clone)]
pub struct PlayerStats {
    pub base_speed: f32,
    pub attack_damage: f32,
    pub attack_rate: f32,
    pub projectile_speed: f32,
    pub projectile_range: f32,
    pub damage_multiplier: f32,
    pub character_type: CharacterType,
    // Heavy-specific
    pub foam_armor: f32,
    pub foam_armor_max: f32,
    // Sparky-specific
    pub heat: f32,
    pub heat_max: f32,
    // Doc-specific
    pub charge: f32,
    pub charge_max: f32,
    pub charging: bool,
    // Queen B-specific
    pub bee_count: u32,
    pub bee_max: u32,
}

impl PlayerStats {
    pub fn effective_speed(&self) -> f32 {
        match self.character_type {
            CharacterType::Heavy => {
                let armor_frac = (self.foam_armor / self.foam_armor_max).min(1.0);
                self.base_speed * (1.0 - armor_frac * 0.5)
            }
            _ => self.base_speed,
        }
    }
}

#[derive(Component, Default)]
pub struct AttackCooldown(pub f32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum CharacterType {
    #[default]
    Torch,
    QueenB,
    Mop,
    Heavy,
    Sparky,
    Doc,
}

impl CharacterType {
    pub fn display_name(&self) -> &'static str {
        match self {
            CharacterType::Torch  => "Torch",
            CharacterType::QueenB => "Queen B",
            CharacterType::Mop    => "Mop",
            CharacterType::Heavy  => "Heavy",
            CharacterType::Sparky => "Sparky",
            CharacterType::Doc    => "Doc",
        }
    }

    pub fn tagline(&self) -> &'static str {
        match self {
            CharacterType::Torch  => "Set everything on fire. Including yourself.",
            CharacterType::QueenB => "Let someone else do it.",
            CharacterType::Mop    => "Keep your distance. Seriously.",
            CharacterType::Heavy  => "More armor means more problems.",
            CharacterType::Sparky => "It's not a safety violation if it works.",
            CharacterType::Doc    => "The malpractice insurance ran out.",
        }
    }
}

#[derive(Resource, Default)]
pub struct CharacterSelection {
    pub character: CharacterType,
}
