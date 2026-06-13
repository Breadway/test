use crate::components::player::{CharacterType, PlayerStats};

fn base_stats(ct: CharacterType) -> PlayerStats {
    PlayerStats {
        max_health: 120.0,
        base_speed: 170.0,
        attack_damage: 15.0,
        attack_rate: 8.0,
        projectile_speed: 350.0,
        projectile_range: 250.0,
        damage_multiplier: 1.0,
        damage_reduction: 0.0,
        character_type: ct,
        foam_armor: 0.0,
        foam_armor_max: 200.0,
        heat: 0.0,
        heat_max: 100.0,
        charge: 0.0,
        charge_max: 100.0,
        charging: false,
        bee_count: 0,
        bee_max: 30,
    }
}

pub fn stats_for_character(ct: CharacterType) -> PlayerStats {
    let mut s = base_stats(ct);
    match ct {
        CharacterType::Torch => {
            s.max_health = 120.0;
            s.base_speed = 180.0;
            s.attack_damage = 12.0;
            s.attack_rate = 10.0;
            s.projectile_range = 200.0;
        }
        CharacterType::QueenB => {
            s.max_health = 100.0; // bees absorb damage before HP
            s.base_speed = 160.0;
            s.attack_damage = 0.0;
            s.damage_multiplier = 0.0;
            s.projectile_speed = 280.0;
            s.projectile_range = 300.0;
        }
        CharacterType::Mop => {
            s.max_health = 150.0; // melee needs the bulk
            s.attack_damage = 30.0;
            s.attack_rate = 2.0;
            s.projectile_speed = 350.0; // unused (melee), kept non-zero defensively
            s.projectile_range = 130.0; // melee reach
        }
        CharacterType::Heavy => {
            s.max_health = 180.0;
            s.base_speed = 200.0;
            s.attack_rate = 5.0; // foam spray rate
            s.projectile_range = 110.0; // spray cone reach
        }
        CharacterType::Sparky => {
            s.max_health = 110.0;
            s.base_speed = 175.0;
            s.attack_damage = 20.0;
            s.attack_rate = 4.0;
            s.projectile_speed = 500.0;
            s.projectile_range = 180.0;
        }
        CharacterType::Doc => {
            s.max_health = 90.0; // glass cannon
            s.base_speed = 165.0;
            s.attack_damage = 50.0;
            s.attack_rate = 1.0;
            s.projectile_speed = 600.0;
            s.projectile_range = 400.0;
        }
    }
    s
}
