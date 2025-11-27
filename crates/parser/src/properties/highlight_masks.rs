// Highlight mask functions for armor, weapon, and resist enchantments
// These decompose bitflags into string representations

pub fn armor_highlight_mask_name(key: u16) -> String {
    let flags: &[(u16, &str)] = &[
        (0x0001, "ArmorLevel"),
        (0x0002, "SlashingProtection"),
        (0x0004, "PiercingProtection"),
        (0x0008, "BludgeoningProtection"),
        (0x0010, "ColdProtection"),
        (0x0020, "FireProtection"),
        (0x0040, "AcidProtection"),
        (0x0080, "ElectricalProtection"),
    ];

    let names: Vec<&str> = flags
        .iter()
        .filter(|(bit, _)| key & bit != 0)
        .map(|(_, name)| *name)
        .collect();

    if names.is_empty() {
        format!("ArmorHighlight_{key}")
    } else {
        names.join(", ")
    }
}

/// WeaponHighlightMask bitflags
pub fn weapon_highlight_mask_name(key: u16) -> String {
    let flags: &[(u16, &str)] = &[
        (0x0001, "AttackSkill"),
        (0x0002, "MeleeDefense"),
        (0x0004, "Speed"),
        (0x0008, "Damage"),
        (0x0010, "DamageVariance"),
        (0x0020, "DamageMod"),
    ];

    let names: Vec<&str> = flags
        .iter()
        .filter(|(bit, _)| key & bit != 0)
        .map(|(_, name)| *name)
        .collect();

    if names.is_empty() {
        format!("WeaponHighlight_{key}")
    } else {
        names.join(", ")
    }
}

/// ResistHighlightMask bitflags
pub fn resist_highlight_mask_name(key: u16) -> String {
    let flags: &[(u16, &str)] = &[
        (0x0001, "ResistSlash"),
        (0x0002, "ResistPierce"),
        (0x0004, "ResistBludgeon"),
        (0x0008, "ResistFire"),
        (0x0010, "ResistCold"),
        (0x0020, "ResistAcid"),
        (0x0040, "ResistElectric"),
        (0x0080, "ResistHealthBoost"),
        (0x0100, "ResistStaminaDrain"),
        (0x0200, "ResistStaminaBoost"),
        (0x0400, "ResistManaDrain"),
        (0x0800, "ResistManaBoost"),
        (0x1000, "ManaConversionMod"),
        (0x2000, "ElementalDamageMod"),
        (0x4000, "ResistNether"),
    ];

    let names: Vec<&str> = flags
        .iter()
        .filter(|(bit, _)| key & bit != 0)
        .map(|(_, name)| *name)
        .collect();

    if names.is_empty() {
        format!("ResistHighlight_{key}")
    } else {
        names.join(", ")
    }
}
