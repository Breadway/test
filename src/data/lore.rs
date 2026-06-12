pub const DEATH_MESSAGES: &[&str] = &[
    "CONTRACT TERMINATED.\nCLAIM STATUS: DENIED\n\"Acts of Arthropod not covered.\"",
    "CONTRACTOR UNRECOVERED.\nNOTE: Equipment salvageable. Contractor: less so.",
    "PERFORMANCE REVIEW: INCOMPLETE\nReason: No longer performing.",
    "OSHA INCIDENT #4,271 LOGGED.\nEstimated resolution: next contractor.",
    "NEXT OF KIN NOTIFIED: N/A\n\"We tried to find someone who cared.\"",
    "HAZARD PAY CLAIM: PROCESSING\nEstimated payout: $0.00 (see Clause 47-B)",
    "THANK YOU FOR YOUR SERVICE.\nYour service has been noted and filed.\nUnder B, for: Bugs.",
    "BODY UNRECOVERED.\nEquipment unrecovered.\nWage garnishment continues.",
    "CAUSE OF DEATH: Workplace Enthusiasm\n\"Technically preventable. Technically.\"",
    "CONTRACTOR #____ STATUS: RESOLVED\n\"The number was higher. We started over.\"",
    "BMG HR MEMO: Vacancy filled.\nPosting live at 8 AM.\nSign-on bonus: $200.",
    "SECTOR INCIDENT LOG:\n[ENTRY REDACTED BY LEGAL]\n\"You didn't see this.\"",
];

pub const ROOM_ADJECTIVES: &[&str] = &[
    "Former", "Abandoned", "Condemned", "Flooded", "Restricted",
    "Evacuated", "Compromised", "Contaminated", "Secured", "Unsafe",
];

pub const ROOM_NOUNS: &[&str] = &[
    "Breakroom", "Corridor", "Server Room", "Archive", "Loading Dock",
    "Stairwell", "Cafeteria", "Storage Room", "Maintenance Bay", "Office",
    "Daycare", "Gymnasium", "Parking Structure", "Boiler Room", "Rooftop",
];

pub const LOADING_TIPS: &[&str] = &[
    "TIP: Fire spreads to nearby enemies. Stand back.",
    "TIP: Heavy's armor slows you down. Damage speeds you back up.",
    "TIP: Sparky's chain lightning does more damage per chain link.",
    "BMG REMINDER: Your life insurance expired at signing.",
    "TIP: Roaches teach AOE value. They travel in families.",
    "BMG MEMO: Report all incidents. They will be ignored.",
    "TIP: Beetles are armored. Aim for their undersides.",
    "FUN FACT: Contractor #0447 is you. Ask why we started at 447.",
    "TIP: Wasps maintain distance. Close the gap.",
    "BMG NOTICE: This facility is not haunted. Do not investigate the noises.",
    "TIP: Level 10 unlocks a major ability branch. Choose wisely.",
    "WARNING: Do not proceed past Sector 4-XQ. This is not a suggestion.",
    "TIP: Queen B's bees shield against damage. Keep them alive.",
    "BMG REMINDER: NDAs are retroactively eternal. Even death is not an excuse.",
];

pub const COLLECTIBLE_DOCS: &[(&str, &str)] = &[
    ("MEMO FRAGMENT #1", "...recommend immediate transition from\nPhase 1 protocol. Liability exposure\nexceeds acceptable threshold.\nEmployee numbering to restart at 0447\nper attached schedule...\n[TORN]"),
    ("PERSONNEL FILE", "CONTRACTOR #0446\nSTATUS: [REDACTED]\nFINAL DEPLOYMENT: Sector 4-XQ\nNOTES: \"Do not send additional teams\nuntil [REMAINDER ILLEGIBLE]\""),
    ("VERDANTIX SAFETY REPORT", "[SIGNED: S. CHEN]\nCompound V-7 stabilizer removal\npresents unacceptable risk to—\n[STAMP: APPROVED FOR PRODUCTION]\n[SECOND STAMP: CONCERNS NOTED]"),
    ("GARY LETTER #3 OF 7", "...and if you're reading this,\nthey've already won.\nCheck the original patents.\nCheck who filed them. Check—\n[BURNED]"),
    ("DIVISION 8 CHARTER", "PURPOSE: Deniable remediation\nworkforce. Contractor classification\nensures liability non-attachment.\nRecruitment targets: the indebted,\nthe criminal, the [REDACTED].\nMORTALITY EXPECTATION: see Appendix C"),
    ("LEGAL MEMO #7-CRIMSON", "RE: Acceptable Casualty Thresholds\n\nAfter review, Legal recommends:\nMonths 1-3: 67% (acceptable)\nMonths 4-6: 45% (target)\nMonth 6+: \"market stabilization\"\n\nNote: Do not distribute."),
];
