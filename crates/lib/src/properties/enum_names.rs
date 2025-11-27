// ============================================================================
// CHILD ENUM MAPPING FUNCTIONS
// These are enums referenced by PropertyInt values via enum= attribute
// Generated from protocol.xml enum definitions
// ============================================================================

pub fn item_type_name(key: u32) -> String {
    match key {
        0x00000001 => "MeleeWeapon",
        0x00000002 => "Armor",
        0x00000004 => "Clothing",
        0x00000008 => "Jewelry",
        0x00000010 => "Creature",
        0x00000020 => "Food",
        0x00000040 => "Money",
        0x00000080 => "Misc",
        0x00000100 => "MissileWeapon",
        0x00000200 => "Container",
        0x00000400 => "Useless",
        0x00000800 => "Gem",
        0x00001000 => "SpellComponents",
        0x00002000 => "Writable",
        0x00004000 => "Key",
        0x00008000 => "Caster",
        0x00010000 => "Portal",
        0x00020000 => "Lockable",
        0x00040000 => "PromissoryNote",
        0x00080000 => "ManaStone",
        0x00100000 => "Service",
        0x00200000 => "MagicWieldable",
        0x00400000 => "CraftCookingBase",
        0x00800000 => "CraftAlchemyBase",
        0x02000000 => "CraftFletchingBase",
        0x04000000 => "CraftAlchemyIntermediate",
        0x08000000 => "CraftFletchingIntermediate",
        0x10000000 => "LifeStone",
        0x20000000 => "TinkeringTool",
        0x40000000 => "TinkeringMaterial",
        0x80000000 => "Gameboard",
        _ => return format!("ItemType_{key}"),
    }
    .to_string()
}

/// CreatureType enum
pub fn creature_type_name(key: u32) -> String {
    match key {
        1 => "Olthoi",
        2 => "Banderling",
        3 => "Drudge",
        4 => "Mosswart",
        5 => "Lugian",
        6 => "Tumerok",
        7 => "Mite",
        8 => "Tusker",
        9 => "PhyntosWasp",
        10 => "Rat",
        11 => "Auroch",
        12 => "Cow",
        13 => "Golem",
        14 => "Undead",
        15 => "Gromnie",
        16 => "Reedshark",
        17 => "Armoredillo",
        18 => "Fae",
        19 => "Virindi",
        20 => "Wisp",
        21 => "Knathtead",
        22 => "Shadow",
        23 => "Mattekar",
        24 => "Mumiyah",
        25 => "Rabbit",
        26 => "Sclavus",
        27 => "ShallowsShark",
        28 => "Monouga",
        29 => "Zefir",
        30 => "Skeleton",
        31 => "Human",
        32 => "Shreth",
        33 => "Chittick",
        34 => "Moarsman",
        35 => "OlthoiLarvae",
        36 => "Slithis",
        37 => "Deru",
        38 => "FireElemental",
        39 => "Snowman",
        40 => "Unknown",
        41 => "Bunny",
        42 => "LightningElemental",
        43 => "Rockslide",
        44 => "Grievver",
        45 => "Niffis",
        46 => "Ursuin",
        47 => "Crystal",
        48 => "HollowMinion",
        49 => "Scarecrow",
        50 => "Idol",
        51 => "Empyrean",
        52 => "Hopeslayer",
        53 => "Doll",
        54 => "Marionette",
        55 => "Carenzi",
        56 => "Siraluun",
        57 => "AunTumerok",
        58 => "HeaTumerok",
        59 => "Simulacrum",
        60 => "AcidElemental",
        61 => "FrostElemental",
        62 => "Elemental",
        63 => "Statue",
        64 => "Wall",
        65 => "AlteredHuman",
        66 => "Device",
        67 => "Harbinger",
        68 => "DarkSarcophagus",
        69 => "Chicken",
        70 => "GotrokLugian",
        71 => "Margul",
        72 => "BleachedRabbit",
        73 => "NastyRabbit",
        74 => "GrimacingRabbit",
        75 => "Burun",
        76 => "Target",
        77 => "Ghost",
        78 => "Fiun",
        79 => "Eater",
        80 => "Penguin",
        81 => "Ruschk",
        82 => "Thrungus",
        83 => "ViamontianKnight",
        84 => "Remoran",
        85 => "Swarm",
        86 => "Moar",
        87 => "EnchantedArms",
        88 => "Sleech",
        89 => "Mukkir",
        90 => "Merwart",
        91 => "Food",
        92 => "ParadoxOlthoi",
        93 => "Harvest",
        94 => "Energy",
        95 => "Apparition",
        96 => "Aerbax",
        97 => "Touched",
        98 => "BlightedMoarsman",
        99 => "GearKnight",
        100 => "Gurog",
        101 => "Anekshay",
        _ => return format!("CreatureType_{key}"),
    }
    .to_string()
}

/// DamageType enum (mask values)
pub fn damage_type_name(key: u32) -> String {
    match key {
        0x00 => "Undef",
        0x01 => "Slash",
        0x02 => "Pierce",
        0x04 => "Bludgeon",
        0x08 => "Cold",
        0x10 => "Fire",
        0x20 => "Acid",
        0x40 => "Electric",
        0x80 => "Health",
        0x100 => "Stamina",
        0x200 => "Mana",
        0x400 => "Nether",
        0x800 => "Base",
        _ => return format!("DamageType_{key}"),
    }
    .to_string()
}

/// SkillId enum
pub fn skill_id_name(key: u32) -> String {
    match key {
        0x01 => "Axe",
        0x02 => "Bow",
        0x03 => "Crossbow",
        0x04 => "Dagger",
        0x05 => "Mace",
        0x06 => "MeleeDefense",
        0x07 => "MissileDefense",
        0x08 => "Sling",
        0x09 => "Spear",
        0x0A => "Staff",
        0x0B => "Sword",
        0x0C => "ThrownWeapons",
        0x0D => "UnarmedCombat",
        0x0E => "ArcaneLore",
        0x0F => "MagicDefense",
        0x10 => "ManaConversion",
        0x11 => "Spellcraft",
        0x12 => "ItemTinkering",
        0x13 => "AssessPerson",
        0x14 => "Deception",
        0x15 => "Healing",
        0x16 => "Jump",
        0x17 => "Lockpick",
        0x18 => "Run",
        0x19 => "Awareness",
        0x1A => "ArmorRepair",
        0x1B => "AssessCreature",
        0x1C => "WeaponTinkering",
        0x1D => "ArmorTinkering",
        0x1E => "MagicItemTinkering",
        0x1F => "CreatureEnchantment",
        0x20 => "ItemEnchantment",
        0x21 => "LifeMagic",
        0x22 => "WarMagic",
        0x23 => "Leadership",
        0x24 => "Loyalty",
        0x25 => "Fletching",
        0x26 => "Alchemy",
        0x27 => "Cooking",
        0x28 => "Salvaging",
        0x29 => "TwoHandedCombat",
        0x2A => "Gearcraft",
        0x2B => "VoidMagic",
        0x2C => "HeavyWeapons",
        0x2D => "LightWeapons",
        0x2E => "FinesseWeapons",
        0x2F => "MissileWeapons",
        0x31 => "DualWield",
        0x32 => "Recklessness",
        0x33 => "SneakAttack",
        0x34 => "DirtyFighting",
        0x35 => "Challenge",
        0x36 => "Summoning",
        _ => return format!("SkillId_{key}"),
    }
    .to_string()
}

/// EquipmentSet enum
pub fn equipment_set_name(key: u32) -> String {
    match key {
        0 => "None",
        1 => "Test",
        2 => "Test2",
        3 => "Unknown3",
        4 => "CarraidasBenediction",
        5 => "NobleRelic",
        6 => "AncientRelic",
        7 => "AlduressaRelic",
        8 => "Ninja",
        9 => "EmpyreanRings",
        10 => "ArmMindHeart",
        11 => "ArmorPerfectLight",
        12 => "ArmorPerfectLight2",
        13 => "Soldiers",
        14 => "Adepts",
        15 => "Archers",
        16 => "Defenders",
        17 => "Tinkers",
        18 => "Crafters",
        19 => "Hearty",
        20 => "Dexterous",
        21 => "Wise",
        22 => "Swift",
        23 => "Hardened",
        24 => "Reinforced",
        25 => "Interlocking",
        26 => "Flameproof",
        27 => "Acidproof",
        28 => "Coldproof",
        29 => "Lightningproof",
        30 => "SocietyArmor",
        31 => "ColosseumClothing",
        32 => "GraveyardClothing",
        33 => "OlthoiClothing",
        34 => "NoobieArmor",
        35 => "AetheriaDefense",
        36 => "AetheriaDestruction",
        37 => "AetheriaFury",
        38 => "AetheriaGrowth",
        39 => "AetheriaVigor",
        40 => "RareDamageResistance",
        41 => "RareDamageBoost",
        42..=48 => "OlthoiArmor_Variant",
        49..=90 => "CloakSkill_Variant",
        91..=129 => "ShroudedSoul_Variant",
        130 => "ShimmeringShadowsSet",
        131..=136 => "SocietyAccessory_Variant",
        137 => "GauntletGarb",
        138 => "ParagonMissile",
        139 => "ParagonCaster",
        140 => "ParagonMelee",
        _ => return format!("EquipmentSet_{key}"),
    }
    .to_string()
}

/// EquipMask enum (mask values) - decomposed as comma-separated flags
pub fn equip_mask_name(key: u32) -> String {
    // EquipMask is a bitflag - decompose into individual flags and join with ", "
    let flags: &[(u32, &str)] = &[
        (0x00000001, "HeadWear"),
        (0x00000002, "ChestWear"),
        (0x00000004, "AbdomenWear"),
        (0x00000008, "UpperArmWear"),
        (0x00000010, "LowerArmWear"),
        (0x00000020, "HandWear"),
        (0x00000040, "UpperLegWear"),
        (0x00000080, "LowerLegWear"),
        (0x00000100, "FootWear"),
        (0x00000200, "ChestArmor"),
        (0x00000400, "AbdomenArmor"),
        (0x00000800, "UpperArmArmor"),
        (0x00001000, "LowerArmArmor"),
        (0x00002000, "UpperLegArmor"),
        (0x00004000, "LowerLegArmor"),
        (0x00008000, "Necklace"),
        (0x00010000, "RightBracelet"),
        (0x00020000, "LeftBracelet"),
        (0x00040000, "RightRing"),
        (0x00080000, "LeftRing"),
        (0x00100000, "MeleeWeapon"),
        (0x00200000, "Shield"),
        (0x00400000, "MissileWeapon"),
        (0x00800000, "Ammunition"),
        (0x01000000, "Wand"),
    ];

    let names: Vec<&str> = flags
        .iter()
        .filter(|(bit, _)| key & bit != 0)
        .map(|(_, name)| *name)
        .collect();

    if names.is_empty() {
        format!("EquipMask_{key}")
    } else {
        names.join(", ")
    }
}

/// CombatMode enum
pub fn combat_mode_name(key: u32) -> String {
    match key {
        0x1 => "NonCombat",
        0x2 => "Melee",
        0x4 => "Missile",
        0x8 => "Magic",
        _ => return format!("CombatMode_{key}"),
    }
    .to_string()
}

/// HeritageGroup enum
pub fn heritage_group_name(key: u32) -> String {
    match key {
        0 => "Invalid",
        1 => "Aluvian",
        2 => "Gharundim",
        3 => "Sho",
        4 => "Viamontian",
        5 => "Shadowbound",
        6 => "Gearknight",
        7 => "Tumerok",
        8 => "Lugian",
        9 => "Empyrean",
        10 => "Penumbraen",
        11 => "Undead",
        12 => "Olthoi",
        13 => "OlthoiAcid",
        _ => return format!("HeritageGroup_{key}"),
    }
    .to_string()
}

/// WeaponType enum
pub fn weapon_type_name(key: u32) -> String {
    match key {
        0 => "Undef",
        1 => "Unarmed",
        2 => "Sword",
        3 => "Axe",
        4 => "Mace",
        5 => "Spear",
        6 => "Dagger",
        7 => "Staff",
        8 => "Bow",
        9 => "Crossbow",
        10 => "Thrown",
        11 => "TwoHanded",
        12 => "Magic",
        _ => return format!("WeaponType_{key}"),
    }
    .to_string()
}

/// MaterialType enum
pub fn material_type_name(key: u32) -> String {
    match key {
        1 => "Ceramic",
        2 => "Porcelain",
        4 => "Linen",
        5 => "Satin",
        6 => "Silk",
        7 => "Velvet",
        8 => "Wool",
        10 => "Agate",
        11 => "Amber",
        12 => "Amethyst",
        13 => "Aquamarine",
        14 => "Azurite",
        15 => "BlackGarnet",
        16 => "BlackOpal",
        17 => "Bloodstone",
        18 => "Carnelian",
        19 => "Citrine",
        20 => "Diamond",
        21 => "Emerald",
        22 => "FireOpal",
        23 => "GreenGarnet",
        24 => "GreenJade",
        25 => "Hematite",
        26 => "ImperialTopaz",
        27 => "Jet",
        28 => "LapisLazuli",
        29 => "LavenderJade",
        30 => "Malachite",
        31 => "Moonstone",
        32 => "Onyx",
        33 => "Opal",
        34 => "Peridot",
        35 => "RedGarnet",
        36 => "RedJade",
        37 => "RoseQuartz",
        38 => "Ruby",
        39 => "Sapphire",
        40 => "SmokeyQuartz",
        41 => "Sunstone",
        42 => "TigerEye",
        43 => "Tourmaline",
        44 => "Turquoise",
        45 => "WhiteJade",
        46 => "WhiteQuartz",
        47 => "WhiteSapphire",
        48 => "YellowGarnet",
        49 => "YellowTopaz",
        50 => "Zircon",
        51 => "Ivory",
        52 => "Leather",
        53 => "ArmoredilloHide",
        54 => "GromnieHide",
        55 => "ReedSharkHide",
        57 => "Brass",
        58 => "Bronze",
        59 => "Copper",
        60 => "Gold",
        61 => "Iron",
        62 => "Pyreal",
        63 => "Silver",
        64 => "Steel",
        66 => "Alabaster",
        67 => "Granite",
        68 => "Marble",
        69 => "Obsidian",
        70 => "Sandstone",
        71 => "Serpentine",
        73 => "Ebony",
        74 => "Mahogany",
        75 => "Oak",
        76 => "Pine",
        77 => "Teak",
        _ => return format!("MaterialType_{key}"),
    }
    .to_string()
}

/// Gender enum
pub fn gender_name(key: u32) -> String {
    match key {
        0 => "Invalid",
        1 => "Male",
        2 => "Female",
        _ => return format!("Gender_{key}"),
    }
    .to_string()
}

/// AttackType enum (mask values)
pub fn attack_type_name(key: u32) -> String {
    match key {
        0x0000 => "Undef",
        0x0001 => "Punch",
        0x0002 => "Thrust",
        0x0004 => "Slash",
        0x0008 => "Kick",
        0x0010 => "OffhandPunch",
        0x0020 => "DoubleSlash",
        0x0040 => "TripleSlash",
        0x0080 => "DoubleThrust",
        0x0100 => "TripleThrust",
        0x0200 => "OffhandThrust",
        0x0400 => "OffhandSlash",
        0x0800 => "OffhandDoubleSlash",
        0x1000 => "OffhandTripleSlash",
        0x2000 => "OffhandDoubleThrust",
        0x4000 => "OffhandTripleThrust",
        _ => return format!("AttackType_{key}"),
    }
    .to_string()
}

/// AttackHeight enum
pub fn attack_height_name(key: u32) -> String {
    match key {
        0x01 => "High",
        0x02 => "Medium",
        0x03 => "Low",
        _ => return format!("AttackHeight_{key}"),
    }
    .to_string()
}

/// CombatStyle enum (mask values)
pub fn combat_style_name(key: u32) -> String {
    match key {
        0x00000 => "Undef",
        0x00001 => "Unarmed",
        0x00002 => "OneHanded",
        0x00004 => "OneHandedAndShield",
        0x00008 => "TwoHanded",
        0x00010 => "Bow",
        0x00020 => "Crossbow",
        0x00040 => "Sling",
        0x00080 => "ThrownWeapon",
        0x00100 => "DualWield",
        0x00200 => "Magic",
        0x00400 => "Atlatl",
        0x00800 => "ThrownShield",
        0x10000 => "StubbornMagic",
        0x20000 => "StubbornProjectile",
        0x40000 => "StubbornMelee",
        0x80000 => "StubbornMissile",
        _ => return format!("CombatStyle_{key}"),
    }
    .to_string()
}

/// Placement enum
pub fn placement_name(key: u32) -> String {
    match key {
        0 => "Default",
        1 => "RightHandCombat",
        2 => "RightHandNonCombat",
        3 => "LeftHand",
        4 => "Belt",
        5 => "Quiver",
        6 => "Shield",
        7 => "LeftWeapon",
        8 => "LeftUnarmed",
        0x33 => "SpecialCrossbowBolt",
        0x34 => "MissileFlight",
        0x65 => "Resting",
        0x66 => "Other",
        0x67 => "Hook",
        _ => return format!("Placement_{key}"),
    }
    .to_string()
}

/// WieldRequirement enum
pub fn wield_requirement_name(key: u32) -> String {
    match key {
        0 => "Undef",
        1 => "Skill",
        2 => "RawSkill",
        3 => "Attrib",
        4 => "RawAttrib",
        5 => "SecondaryAttrib",
        6 => "RawSecondaryAttrib",
        7 => "Level",
        8 => "Training",
        9 => "IntStat",
        10 => "BoolStat",
        11 => "CreatureType",
        12 => "HeritageType",
        _ => return format!("WieldRequirement_{key}"),
    }
    .to_string()
}

/// CoverageMask enum (mask values)
pub fn coverage_mask_name(key: u32) -> String {
    match key {
        0x00000002 => "UpperLegsUnderwear",
        0x00000004 => "LowerLegsUnderwear",
        0x00000008 => "ChestUnderwear",
        0x00000010 => "AbdomenUnderwear",
        0x00000020 => "UpperArmsUnderwear",
        0x00000040 => "LowerArmsUnderwear",
        0x00000100 => "UpperLegs",
        0x00000200 => "LowerLegs",
        0x00000400 => "Chest",
        0x00000800 => "Abdomen",
        0x00001000 => "UpperArms",
        0x00002000 => "LowerArms",
        0x00004000 => "Head",
        0x00008000 => "Hands",
        0x00010000 => "Feet",
        _ => return format!("CoverageMask_{key}"),
    }
    .to_string()
}

/// AmmoType enum
pub fn ammo_type_name(key: u32) -> String {
    match key {
        0 => "None",
        1 => "Arrow",
        2 => "Bolt",
        4 => "Atlatl",
        8 => "ThrownWeapon",
        _ => return format!("AmmoType_{key}"),
    }
    .to_string()
}

/// CombatUse enum
pub fn combat_use_name(key: u32) -> String {
    match key {
        0 => "None",
        1 => "Melee",
        2 => "Missile",
        4 => "Ammo",
        8 => "Shield",
        16 => "TwoHanded",
        _ => return format!("CombatUse_{key}"),
    }
    .to_string()
}

/// ParentLocation enum
pub fn parent_location_name(key: u32) -> String {
    match key {
        0 => "None",
        1 => "RightHand",
        2 => "LeftHand",
        3 => "Shield",
        4 => "Belt",
        5 => "Quiver",
        6 => "Hearldry",
        7 => "Mouth",
        8 => "LeftWeapon",
        9 => "LeftUnarmed",
        _ => return format!("ParentLocation_{key}"),
    }
    .to_string()
}

/// RadarColor enum
pub fn radar_color_name(key: u32) -> String {
    match key {
        0 => "Default",
        1 => "Blue",
        2 => "Gold",
        3 => "White",
        4 => "Purple",
        5 => "Red",
        6 => "Pink",
        7 => "Green",
        8 => "Yellow",
        9 => "Cyan",
        10 => "BrightGreen",
        _ => return format!("RadarColor_{key}"),
    }
    .to_string()
}

/// RadarBehavior enum
pub fn radar_behavior_name(key: u32) -> String {
    match key {
        0 => "Undefined",
        1 => "ShowNever",
        2 => "ShowMovement",
        3 => "ShowAttacking",
        4 => "ShowAlways",
        _ => return format!("RadarBehavior_{key}"),
    }
    .to_string()
}

/// ArmorType enum
pub fn armor_type_name(key: u32) -> String {
    match key {
        0 => "None",
        1 => "Cloth",
        2 => "Leather",
        4 => "StuddedLeather",
        8 => "Scalemail",
        16 => "Chainmail",
        32 => "Metal",
        _ => return format!("ArmorType_{key}"),
    }
    .to_string()
}

/// ImbuedEffectType enum (mask values)
pub fn imbued_effect_type_name(key: u32) -> String {
    match key {
        0 => "Undef",
        0x0001 => "CriticalStrike",
        0x0002 => "CripplingBlow",
        0x0004 => "ArmorRending",
        0x0008 => "SlashRending",
        0x0010 => "PierceRending",
        0x0020 => "BludgeonRending",
        0x0040 => "AcidRending",
        0x0080 => "ColdRending",
        0x0100 => "ElectricRending",
        0x0200 => "FireRending",
        0x0400 => "MeleeDefense",
        0x0800 => "MissileDefense",
        0x1000 => "MagicDefense",
        0x2000 => "Spellbook",
        0x4000 => "NetherRending",
        0x20000000 => "IgnoreSomeMagicProjectileDamage",
        0x40000000 => "AlwaysCritical",
        0x80000000 => "IgnoreAllArmor",
        _ => return format!("ImbuedEffectType_{key}"),
    }
    .to_string()
}

/// UiEffects enum (mask values)
pub fn ui_effects_name(key: u32) -> String {
    match key {
        0x0000 => "Undef",
        0x0001 => "Magical",
        0x0002 => "Poisoned",
        0x0004 => "BoostHealth",
        0x0008 => "BoostMana",
        0x0010 => "BoostStamina",
        0x0020 => "Fire",
        0x0040 => "Lightning",
        0x0080 => "Frost",
        0x0100 => "Acid",
        0x0200 => "Bludgeoning",
        0x0400 => "Slashing",
        0x0800 => "Piercing",
        0x1000 => "Nether",
        _ => return format!("UiEffects_{key}"),
    }
    .to_string()
}

/// Usable enum (mask values)
pub fn usable_name(key: u32) -> String {
    match key {
        0x00000001 => "No",
        0x00000010 => "Self",
        0x00000012 => "Wielded",
        0x00000014 => "Contained",
        0x00000018 => "Viewed",
        0x00000020 => "Remote",
        0x00000030 => "NeverWalk",
        0x00000040 => "ObjSelf",
        0x00000800 => "ContainedViewedRemote",
        0x00000810 => "ContainedViewedRemoteNeverWalk",
        0x00001000 => "ViewedRemote",
        0x00001010 => "ViewedRemoteNeverWalk",
        0x00008000 => "SourceWieldedTargetWielded",
        0x00014000 => "SourceWieldedTargetContained",
        0x00018000 => "SourceWieldedTargetViewed",
        0x00030000 => "SourceWieldedTargetRemote",
        0x00040000 => "SourceWieldedTargetRemoteNeverWalk",
        0x00080000 => "SourceContainedTargetWielded",
        0x000C0000 => "SourceContainedTargetContained",
        0x000C8000 => "SourceContainedTargetSelfOrContained",
        _ => return format!("Usable_{key}"),
    }
    .to_string()
}

/// BondedStatus enum
pub fn bonded_status_name(key: u32) -> String {
    match key {
        0 => "Normal",
        1 => "Bonded",
        2 => "Sticky",
        0xFFFFFFFF => "Slippery",
        0xFFFFFFFE => "Destroy",
        _ => return format!("BondedStatus_{key}"),
    }
    .to_string()
}

/// AttunedStatus enum
pub fn attuned_status_name(key: u32) -> String {
    match key {
        0 => "Normal",
        1 => "Attuned",
        2 => "Sticky",
        _ => return format!("AttunedStatus_{key}"),
    }
    .to_string()
}

/// HouseType enum
pub fn house_type_name(key: u32) -> String {
    match key {
        1 => "Cottage",
        2 => "Villa",
        3 => "Mansion",
        4 => "Apartment",
        _ => return format!("HouseType_{key}"),
    }
    .to_string()
}

/// HookType enum (mask values)
pub fn hook_type_name(key: u32) -> String {
    match key {
        0x0001 => "Floor",
        0x0002 => "Wall",
        _ => return format!("HookType_{key}"),
    }
    .to_string()
}

/// PaletteTemplate enum
pub fn palette_template_name(key: u32) -> String {
    match key {
        0 => "Undef",
        1 => "AquaBlue",
        2 => "Blue",
        3 => "BluePurple",
        4 => "Brown",
        5 => "DarkBlue",
        6 => "DeepBrown",
        7 => "DeepGreen",
        8 => "Green",
        9 => "Grey",
        10 => "LightBlue",
        11 => "Maroon",
        12 => "Navy",
        13 => "Purple",
        14 => "Red",
        15 => "RedPurple",
        16 => "Rose",
        17 => "Yellow",
        18 => "YellowBrown",
        19 => "Copper",
        20 => "Silver",
        21 => "Gold",
        22 => "Aqua",
        23..=38 => "MetalVariant",
        39 => "Black",
        40 => "Bronze",
        41..=88 => "ColorVariant",
        _ => return format!("PaletteTemplate_{key}"),
    }
    .to_string()
}

/// FactionBits enum (mask values)
pub fn faction_bits_name(key: u32) -> String {
    match key {
        0 => "None",
        0x01 => "CelestialHand",
        0x02 => "EldrytchWeb",
        0x04 => "RadiantBlood",
        _ => return format!("FactionBits_{key}"),
    }
    .to_string()
}

/// PortalBitmask enum
pub fn portal_bitmask_name(key: u32) -> String {
    match key {
        0x00 => "Unrestricted",
        0x01 => "NoPk",
        0x02 => "NoPKLite",
        0x04 => "NoNPK",
        0x08 => "NoSummon",
        0x10 => "NoRecall",
        0x20 => "OnlyOlthoiPCs",
        0x40 => "NoOlthoiPCs",
        0x80 => "NoVitae",
        0x100 => "NoNewAccounts",
        _ => return format!("PortalBitmask_{key}"),
    }
    .to_string()
}

/// AetheriaBitfield enum
pub fn aetheria_bitfield_name(key: u32) -> String {
    match key {
        0 => "None",
        0x1 => "Blue",
        0x2 => "Yellow",
        0x4 => "Red",
        _ => return format!("AetheriaBitfield_{key}"),
    }
    .to_string()
}

/// SummoningMastery enum
pub fn summoning_mastery_name(key: u32) -> String {
    match key {
        0 => "Undef",
        1 => "Primalist",
        2 => "Necromancer",
        3 => "Naturalist",
        _ => return format!("SummoningMastery_{key}"),
    }
    .to_string()
}
