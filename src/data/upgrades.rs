use crate::components::player::PlayerStats;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UpgradeId {
    // Power
    DamageBoost,
    PiercingShots,
    RapidFire,
    ExtraProjectile,
    // Defense
    MaxHpUp,
    RegenOnKill,
    ArmorPlating,
    // Utility
    MoveSpeedUp,
    CooldownReduction,
    DoubleXP,
    // Character specific
    TorchInferno,
    TorchPhoenix,
    QueenBRoyalJelly,
    MopExtension,
    HeavyFortify,
    SparkySupercharge,
    DocOverclock,
}

#[derive(Clone)]
pub enum UpgradeCategory {
    Power,
    Defense,
    Utility,
    Character,
}

pub struct UpgradeData {
    pub id: UpgradeId,
    pub name: &'static str,
    pub description: &'static str,
    pub flavor: &'static str,
    pub category: UpgradeCategory,
    pub apply: fn(&mut PlayerStats),
}

pub static ALL_UPGRADES: &[UpgradeData] = &[
    UpgradeData {
        id: UpgradeId::DamageBoost,
        name: "Performance Review: Positive",
        description: "+20% damage",
        flavor: "\"Exceptional results. Keep up the acceptable rate of survival.\"",
        category: UpgradeCategory::Power,
        apply: |s| { s.damage_multiplier *= 1.2; },
    },
    UpgradeData {
        id: UpgradeId::PiercingShots,
        name: "Armor-Piercing Memo",
        description: "Projectiles pierce through 2 additional enemies",
        flavor: "\"CC to: everyone. Re: penetration policy revision.\"",
        category: UpgradeCategory::Power,
        apply: |s| { s.attack_damage *= 1.1; },
    },
    UpgradeData {
        id: UpgradeId::RapidFire,
        name: "Expedited Processing",
        description: "+25% attack speed",
        flavor: "\"Timeline moved up. Adjust accordingly or be adjusted.\"",
        category: UpgradeCategory::Power,
        apply: |s| { s.attack_rate *= 1.25; },
    },
    UpgradeData {
        id: UpgradeId::ExtraProjectile,
        name: "Redundancy Initiative",
        description: "+15% damage",
        flavor: "\"Belt AND suspenders. And a second belt.\"",
        category: UpgradeCategory::Power,
        apply: |s| { s.attack_damage *= 1.15; },
    },
    UpgradeData {
        id: UpgradeId::MaxHpUp,
        name: "Premium Benefits Package",
        description: "+30 max HP",
        flavor: "\"Dental not included. Or vision. Or anything useful.\"",
        category: UpgradeCategory::Defense,
        apply: |_s| { /* applied via event in health system */ },
    },
    UpgradeData {
        id: UpgradeId::RegenOnKill,
        name: "Contingency Clause",
        description: "Regain 3 HP per kill",
        flavor: "\"Death benefits do not apply if death is work-related.\"",
        category: UpgradeCategory::Defense,
        apply: |_s| { /* flag set via resource */ },
    },
    UpgradeData {
        id: UpgradeId::ArmorPlating,
        name: "Standard-Issue Padding",
        description: "+15% damage reduction",
        flavor: "\"Warranty void upon contact with mandibles.\"",
        category: UpgradeCategory::Defense,
        apply: |s| { s.damage_multiplier *= 0.85; },
    },
    UpgradeData {
        id: UpgradeId::MoveSpeedUp,
        name: "Efficiency Optimization",
        description: "+15% movement speed",
        flavor: "\"Move faster. The bugs are also moving faster.\"",
        category: UpgradeCategory::Utility,
        apply: |s| { s.base_speed *= 1.15; },
    },
    UpgradeData {
        id: UpgradeId::CooldownReduction,
        name: "Streamlined Protocol",
        description: "+20% attack speed",
        flavor: "\"Paperwork reduction in progress. Estimated completion: never.\"",
        category: UpgradeCategory::Utility,
        apply: |s| { s.attack_rate *= 1.2; },
    },
    UpgradeData {
        id: UpgradeId::DoubleXP,
        name: "On-the-Job Training",
        description: "+50% XP gain",
        flavor: "\"Learning opportunities abound. Mostly: don't die.\"",
        category: UpgradeCategory::Utility,
        apply: |_s| { /* handled in XP system via flag */ },
    },
    UpgradeData {
        id: UpgradeId::TorchInferno,
        name: "Inferno Protocol",
        description: "Fire spreads faster and lasts longer",
        flavor: "\"Technically still within code. Technically.\"",
        category: UpgradeCategory::Character,
        apply: |s| { s.attack_damage *= 1.3; },
    },
    UpgradeData {
        id: UpgradeId::TorchPhoenix,
        name: "Phoenix Mode",
        description: "Deal more damage at low HP",
        flavor: "\"[WARNING: Not responsible for bystander incidents]\"",
        category: UpgradeCategory::Character,
        apply: |s| { s.damage_multiplier *= 1.4; },
    },
    UpgradeData {
        id: UpgradeId::QueenBRoyalJelly,
        name: "Royal Jelly Boost",
        description: "Bees deal more damage",
        flavor: "\"They're not aggressive. They're motivated.\"",
        category: UpgradeCategory::Character,
        apply: |s| { s.damage_multiplier *= 1.3; },
    },
    UpgradeData {
        id: UpgradeId::MopExtension,
        name: "Extended Handle",
        description: "+20% weapon range",
        flavor: "\"OSHA does not recommend. OSHA was not consulted.\"",
        category: UpgradeCategory::Character,
        apply: |s| { s.projectile_range *= 1.2; },
    },
    UpgradeData {
        id: UpgradeId::HeavyFortify,
        name: "Reinforced Padding",
        description: "+40 max foam armor",
        flavor: "\"Foam not rated for sustained arthropod aggression.\"",
        category: UpgradeCategory::Character,
        apply: |s| { s.foam_armor_max += 40.0; },
    },
    UpgradeData {
        id: UpgradeId::SparkySupercharge,
        name: "Supercharge",
        description: "Chain lightning hits more targets",
        flavor: "\"This voids 23 warranties and one restraining order.\"",
        category: UpgradeCategory::Character,
        apply: |s| { s.attack_damage *= 1.25; },
    },
    UpgradeData {
        id: UpgradeId::DocOverclock,
        name: "Overclock",
        description: "Charge builds 30% faster",
        flavor: "\"Side effects: tremors, hubris, thermal events.\"",
        category: UpgradeCategory::Character,
        apply: |s| { s.attack_rate *= 1.3; },
    },
];

pub fn get_upgrade(id: UpgradeId) -> Option<&'static UpgradeData> {
    ALL_UPGRADES.iter().find(|u| u.id == id)
}

pub fn general_upgrades() -> Vec<UpgradeId> {
    vec![
        UpgradeId::DamageBoost,
        UpgradeId::PiercingShots,
        UpgradeId::RapidFire,
        UpgradeId::ExtraProjectile,
        UpgradeId::MaxHpUp,
        UpgradeId::RegenOnKill,
        UpgradeId::ArmorPlating,
        UpgradeId::MoveSpeedUp,
        UpgradeId::CooldownReduction,
        UpgradeId::DoubleXP,
    ]
}
