use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use crate::reader::BinaryReader;

/// AppraisalFlags - determines which property sets are included
pub mod appraisal_flags {
    pub const INT_PROPERTIES: u32 = 0x00000001;
    pub const BOOL_PROPERTIES: u32 = 0x00000002;
    pub const FLOAT_PROPERTIES: u32 = 0x00000004;
    pub const STRING_PROPERTIES: u32 = 0x00000008;
    pub const SPELL_BOOK: u32 = 0x00000010;
    pub const WEAPON_PROFILE: u32 = 0x00000020;
    pub const HOOK_PROFILE: u32 = 0x00000040;
    pub const ARMOR_PROFILE: u32 = 0x00000080;
    pub const CREATURE_PROFILE: u32 = 0x00000100;
    pub const ARMOR_ENCH_RATING: u32 = 0x00000200;
    pub const RESIST_ENCH_RATING: u32 = 0x00000400;
    pub const WEAPON_ENCH_RATING: u32 = 0x00000800;
    pub const DATA_ID_PROPERTIES: u32 = 0x00001000;
    pub const INT64_PROPERTIES: u32 = 0x00002000;
    pub const BASE_ARMOR: u32 = 0x00004000;
}

/// ArmorProfile for protection values
#[derive(Debug, Clone, Serialize)]
pub struct ArmorProfile {
    #[serde(rename = "ProtSlashing")]
    pub prot_slashing: f32,
    #[serde(rename = "ProtPiercing")]
    pub prot_piercing: f32,
    #[serde(rename = "ProtBludgeoning")]
    pub prot_bludgeoning: f32,
    #[serde(rename = "ProtCold")]
    pub prot_cold: f32,
    #[serde(rename = "ProtFire")]
    pub prot_fire: f32,
    #[serde(rename = "ProtAcid")]
    pub prot_acid: f32,
    #[serde(rename = "ProtNether")]
    pub prot_nether: f32,
    #[serde(rename = "ProtLightning")]
    pub prot_lightning: f32,
}

impl ArmorProfile {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            prot_slashing: reader.read_f32()?,
            prot_piercing: reader.read_f32()?,
            prot_bludgeoning: reader.read_f32()?,
            prot_cold: reader.read_f32()?,
            prot_fire: reader.read_f32()?,
            prot_acid: reader.read_f32()?,
            prot_nether: reader.read_f32()?,
            prot_lightning: reader.read_f32()?,
        })
    }
}

/// WeaponProfile for weapon appraisal data (28 bytes)
#[derive(Debug, Clone, Serialize)]
pub struct WeaponProfile {
    #[serde(rename = "DamageType")]
    pub damage_type: u32,
    #[serde(rename = "WeaponTime")]
    pub weapon_time: u32,
    #[serde(rename = "WeaponSkill")]
    pub weapon_skill: u32,
    #[serde(rename = "WeaponDamage")]
    pub weapon_damage: u32,
    #[serde(rename = "DamageVariance")]
    pub damage_variance: f64,
    #[serde(rename = "DamageMod")]
    pub damage_mod: f64,
    #[serde(rename = "WeaponLength")]
    pub weapon_length: f64,
    #[serde(rename = "MaxVelocity")]
    pub max_velocity: f64,
    #[serde(rename = "WeaponOffense")]
    pub weapon_offense: f64,
    #[serde(rename = "MaxVelocityEstimated")]
    pub max_velocity_estimated: u32,
}

impl WeaponProfile {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            damage_type: reader.read_u32()?,
            weapon_time: reader.read_u32()?,
            weapon_skill: reader.read_u32()?,
            weapon_damage: reader.read_u32()?,
            damage_variance: reader.read_f64()?,
            damage_mod: reader.read_f64()?,
            weapon_length: reader.read_f64()?,
            max_velocity: reader.read_f64()?,
            weapon_offense: reader.read_f64()?,
            max_velocity_estimated: reader.read_u32()?,
        })
    }
}

/// HookProfile for hook appraisal data (4 bytes)
#[derive(Debug, Clone, Serialize)]
pub struct HookProfile {
    #[serde(rename = "Flags")]
    pub flags: u32,
}

impl HookProfile {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            flags: reader.read_u32()?,
        })
    }
}

/// CreatureProfile for creature appraisal data
#[derive(Debug, Clone, Serialize)]
pub struct CreatureProfile {
    #[serde(rename = "Health")]
    pub health: u32,
    #[serde(rename = "HealthMax")]
    pub health_max: u32,
    #[serde(rename = "Strength")]
    pub strength: u32,
    #[serde(rename = "Endurance")]
    pub endurance: u32,
    #[serde(rename = "Quickness")]
    pub quickness: u32,
    #[serde(rename = "Coordination")]
    pub coordination: u32,
    #[serde(rename = "Focus")]
    pub focus: u32,
    #[serde(rename = "Self")]
    pub self_attr: u32,
    #[serde(rename = "Stamina")]
    pub stamina: u32,
    #[serde(rename = "StaminaMax")]
    pub stamina_max: u32,
    #[serde(rename = "Mana")]
    pub mana: u32,
    #[serde(rename = "ManaMax")]
    pub mana_max: u32,
}

impl CreatureProfile {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            health: reader.read_u32()?,
            health_max: reader.read_u32()?,
            strength: reader.read_u32()?,
            endurance: reader.read_u32()?,
            quickness: reader.read_u32()?,
            coordination: reader.read_u32()?,
            focus: reader.read_u32()?,
            self_attr: reader.read_u32()?,
            stamina: reader.read_u32()?,
            stamina_max: reader.read_u32()?,
            mana: reader.read_u32()?,
            mana_max: reader.read_u32()?,
        })
    }
}

/// Read a PackableHashTable<u32, i32> (property int)
pub fn read_int_properties(reader: &mut BinaryReader) -> Result<HashMap<String, i32>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_i32()?;
        map.insert(property_int_name(key), value);
    }
    Ok(map)
}

/// Read a PackableHashTable<u32, i64> (property int64)
pub fn read_int64_properties(reader: &mut BinaryReader) -> Result<HashMap<String, i64>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_i64()?;
        map.insert(property_int64_name(key), value);
    }
    Ok(map)
}

/// Read a PackableHashTable<u32, bool> (property bool) - bools stored as i32
pub fn read_bool_properties(reader: &mut BinaryReader) -> Result<HashMap<String, bool>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_i32()? != 0;
        map.insert(property_bool_name(key), value);
    }
    Ok(map)
}

/// Read a PackableHashTable<u32, f64> (property float) - f64 in appraisal
pub fn read_float_properties(reader: &mut BinaryReader) -> Result<HashMap<String, f64>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_f64()?;
        map.insert(property_float_name(key), value);
    }
    Ok(map)
}

/// Read a PackableHashTable<u32, String> (property string)
pub fn read_string_properties(reader: &mut BinaryReader) -> Result<HashMap<String, String>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_string16l_ex()?;
        map.insert(property_string_name(key), value);
    }
    Ok(map)
}

/// Read a PackableHashTable<u32, u32> (property dataid)
pub fn read_dataid_properties(reader: &mut BinaryReader) -> Result<HashMap<String, u32>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_u32()?;
        map.insert(property_dataid_name(key), value);
    }
    Ok(map)
}

/// LayeredSpellId for spellbook entries
#[derive(Debug, Clone, Serialize)]
pub struct LayeredSpellId {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "Layer")]
    pub layer: u16,
}

/// Read a PackableList<LayeredSpellId> - u32 count, then N × u32 spell_id
pub fn read_spell_book(reader: &mut BinaryReader) -> Result<Vec<LayeredSpellId>> {
    use anyhow::Context;
    let count = reader.read_u32().context(format!("spell count at pos {}", reader.position()))? as usize;
    if count > 1000 {
        anyhow::bail!("Suspicious spell count: {} at pos {}", count, reader.position());
    }
    let mut spells = Vec::with_capacity(count);
    for i in 0..count {
        let id = reader.read_u32().context(format!("spell {} at pos {}", i, reader.position()))?;
        // Layer is not stored in the binary - reference shows it as separate field
        // For now, output just the ID
        spells.push(LayeredSpellId { id, layer: 0 });
    }
    Ok(spells)
}

// Property name lookup functions - values from ACProtocol/protocol.xml PropertyInt enum
pub fn property_int_name(key: u32) -> String {
    match key {
        1 => "ItemType", 2 => "CreatureType", 3 => "PaletteTemplate", 4 => "ClothingPriority",
        5 => "EncumbranceVal", 6 => "ItemsCapacity", 7 => "ContainersCapacity", 8 => "Mass",
        9 => "ValidLocations", 10 => "CurrentWieldedLocation", 11 => "MaxStackSize", 12 => "StackSize",
        13 => "StackUnitEncumbrance", 14 => "StackUnitMass", 15 => "StackUnitValue", 16 => "ItemUseable",
        17 => "RareId", 18 => "UiEffects", 19 => "Value", 20 => "CoinValue",
        21 => "TotalExperience", 22 => "AvailableCharacter", 23 => "TotalSkillCredits", 24 => "AvailableSkillCredits",
        25 => "Level", 26 => "AccountRequirements", 27 => "ArmorType", 28 => "ArmorLevel",
        29 => "AllegianceCpPool", 30 => "AllegianceRank", 31 => "ChannelsAllowed", 32 => "ChannelsActive",
        33 => "Bonded", 34 => "MonarchsRank", 35 => "AllegianceFollowers", 36 => "ResistMagic",
        37 => "ResistItemAppraisal", 38 => "ResistLockpick", 39 => "DeprecatedResistRepair", 40 => "CombatMode",
        41 => "CurrentAttackHeight", 42 => "CombatCollisions", 43 => "NumDeaths",
        44 => "Damage", 45 => "DamageType", 46 => "DefaultCombatStyle", 47 => "AttackType",
        48 => "WeaponSkill", 49 => "WeaponTime", 50 => "AmmoType", 51 => "CombatUse",
        52 => "ParentLocation", 53 => "PlacementPosition", 54 => "WeaponEncumbrance", 55 => "WeaponMass",
        56 => "ShieldValue", 57 => "ShieldEncumbrance", 58 => "MissileInventoryLocation",
        59 => "FullDamageType", 60 => "WeaponRange", 61 => "AttackersSkill", 62 => "DefendersSkill",
        63 => "AttackersSkillValue", 64 => "AttackersClass", 65 => "Placement",
        66 => "CheckpointStatus", 67 => "Tolerance", 68 => "TargetingTactic", 69 => "CombatTactic",
        70 => "HomesickTargetingTactic", 71 => "NumFollowFailures", 72 => "FriendType", 73 => "FoeType",
        74 => "MerchandiseItemTypes", 75 => "MerchandiseMinValue", 76 => "MerchandiseMaxValue",
        77 => "NumItemsSold", 78 => "NumItemsBought", 79 => "MoneyIncome", 80 => "MoneyOutflow",
        81 => "MaxGeneratedObjects", 82 => "InitGeneratedObjects", 83 => "ActivationResponse",
        84 => "OriginalValue", 85 => "NumMoveFailures", 86 => "MinLevel", 87 => "MaxLevel",
        88 => "LockpickMod", 89 => "BoosterEnum", 90 => "BoostValue",
        91 => "MaxStructure", 92 => "Structure", 93 => "PhysicsState", 94 => "TargetType",
        95 => "RadarBlipColor", 96 => "EncumbranceCapacity", 97 => "LoginTimestamp", 98 => "CreationTimestamp",
        99 => "PkLevelModifier", 100 => "GeneratorType", 101 => "AiAllowedCombatStyle",
        102 => "LogoffTimestamp", 103 => "GeneratorDestructionType", 104 => "ActivationCreateClass",
        105 => "ItemWorkmanship", 106 => "ItemSpellcraft", 107 => "ItemCurMana", 108 => "ItemMaxMana",
        109 => "ItemDifficulty", 110 => "ItemAllegianceRankLimit", 111 => "PortalBitmask",
        112 => "AdvocateLevel", 113 => "Gender", 114 => "Attuned", 115 => "ItemSkillLevelLimit",
        116 => "GateLogic", 117 => "ItemManaCost", 118 => "Logoff", 119 => "Active",
        120 => "AttackHeight", 121 => "NumAttackFailures", 122 => "AiCpThreshold",
        123 => "AiAdvancementStrategy", 124 => "Version", 125 => "Age",
        126 => "VendorHappyMean", 127 => "VendorHappyVariance", 128 => "CloakStatus",
        129 => "VitaeCpPool", 130 => "NumServicesSold", 131 => "MaterialType",
        132 => "NumAllegianceBreaks", 133 => "ShowableOnRadar", 134 => "PlayerKillerStatus",
        135 => "VendorHappyMaxItems", 136 => "ScorePageNum", 137 => "ScoreConfigNum",
        138 => "ScoreNumScores", 139 => "DeathLevel", 140 => "AiOptions",
        141 => "OpenToEveryone", 142 => "GeneratorTimeType", 143 => "GeneratorStartTime",
        144 => "GeneratorEndTime", 145 => "GeneratorEndDestructionType", 146 => "XpOverride",
        147 => "NumCrashAndTurns", 148 => "ComponentWarningThreshold", 149 => "HouseStatus",
        150 => "HookPlacement", 151 => "HookType", 152 => "HookItemType",
        153 => "AiPpThreshold", 154 => "GeneratorVersion", 155 => "HouseType",
        156 => "PickupEmoteOffset", 157 => "WeenieIteration",
        158 => "WieldRequirements", 159 => "WieldSkillType", 160 => "WieldDifficulty",
        161 => "HouseMaxHooksUsable", 162 => "HouseCurrentHooksUsable",
        163 => "AllegianceMinLevel", 164 => "AllegianceMaxLevel", 165 => "HouseRelinkHookCount",
        166 => "SlayerCreatureType", 167 => "ConfirmationInProgress", 168 => "ConfirmationTypeInProgress",
        169 => "TsysMutationData", 170 => "NumItemsInMaterial",
        171 => "NumTimesTinkered", 172 => "AppraisalLongDescDecoration", 173 => "AppraisalLockpickSuccessPercent",
        174 => "AppraisalPages", 175 => "AppraisalMaxPages", 176 => "AppraisalItemSkill",
        177 => "GemCount", 178 => "GemType", 179 => "ImbuedEffect",
        180 => "AttackersRawSkillValue", 181 => "ChessRank", 182 => "ChessTotalGames",
        183 => "ChessGamesWon", 184 => "ChessGamesLost",
        185 => "TypeOfAlteration", 186 => "SkillToBeAltered", 187 => "SkillAlterationCount",
        188 => "HeritageGroup", 189 => "TransferFromAttribute", 190 => "TransferToAttribute",
        191 => "AttributeTransferCount", 192 => "FakeFishingSkill", 193 => "NumKeys",
        194 => "DeathTimestamp", 195 => "PkTimestamp", 196 => "VictimTimestamp",
        197 => "HookGroup", 198 => "AllegianceSwearTimestamp", 199 => "HousePurchaseTimestamp",
        200 => "RedirectableEquippedArmorCount",
        265 => "EquipmentSetId", 267 => "Lifespan", 268 => "RemainingLifespan",
        270 => "WieldRequirements2", 271 => "WieldSkillType2", 272 => "WieldDifficulty2",
        273 => "WieldRequirements3", 274 => "WieldSkillType3", 275 => "WieldDifficulty3",
        276 => "WieldRequirements4", 277 => "WieldSkillType4", 278 => "WieldDifficulty4",
        279 => "Unique", 280 => "SharedCooldown",
        // Ratings (300+)
        307 => "DamageRating", 308 => "DamageResistRating", 309 => "AugmentationDamageBonus",
        310 => "AugmentationDamageReduction", 311 => "ImbueStackingBits",
        312 => "HealOverTime", 313 => "CritRating", 314 => "CritDamageRating",
        315 => "CritResistRating", 316 => "CritDamageResistRating", 317 => "HealingResistRating",
        318 => "DamageOverTime", 319 => "ItemMaxLevel", 320 => "ItemXpStyle",
        321 => "EquipmentSetExtra", 322 => "AetheriaBitfield", 323 => "HealingBoostRating",
        352 => "CloakWeaveProc", 353 => "WeaponType",
        // Gear ratings (370+)
        370 => "GearDamage", 371 => "GearDamageResist", 372 => "GearCrit", 373 => "GearCritResist",
        374 => "GearCritDamage", 375 => "GearCritDamageResist", 376 => "GearHealingBoost",
        377 => "GearNetherResist", 378 => "GearLifeResist", 379 => "GearMaxHealth",
        381 => "PKDamageRating", 382 => "PKDamageResistRating",
        383 => "GearPKDamageRating", 384 => "GearPKDamageResistRating",
        386 => "Overpower", 387 => "OverpowerResist",
        388 => "GearOverpower", 389 => "GearOverpowerResist", 390 => "Enlightenment",
        _ => return format!("PropertyInt_{}", key),
    }.to_string()
}

pub fn property_int64_name(key: u32) -> String {
    match key {
        1 => "TotalExperience",
        2 => "AvailableExperience",
        3 => "AugmentationCost",
        4 => "ItemTotalXp",
        5 => "ItemBaseXp",
        6 => "AvailableLuminance",
        7 => "MaximumLuminance",
        8 => "InteractionReqs",
        _ => return format!("Int64_{}", key),
    }.to_string()
}

pub fn property_bool_name(key: u32) -> String {
    // Values from ACProtocol/protocol.xml PropertyBool enum
    match key {
        0 => "Undef",
        1 => "Stuck",
        2 => "Open",
        3 => "Locked",
        4 => "RotProof",
        5 => "AllegianceUpdateRequest",
        6 => "AiUsesMana",
        7 => "AiUseHumanMagicAnimations",
        8 => "AllowGive",
        9 => "CurrentlyAttacking",
        10 => "AttackerAi",
        11 => "IgnoreCollisions",
        12 => "ReportCollisions",
        13 => "Ethereal",
        14 => "GravityStatus",
        15 => "LightsStatus",
        16 => "ScriptedCollision",
        17 => "Inelastic",
        18 => "Visibility",
        19 => "Attackable",
        20 => "SafeSpellComponents",
        21 => "AdvocateState",
        22 => "Inscribable",
        23 => "DestroyOnSell",
        24 => "UiHidden",
        25 => "IgnoreHouseBarriers",
        26 => "HiddenAdmin",
        27 => "PkWounder",
        28 => "PkKiller",
        29 => "NoCorpse",
        30 => "UnderLifestoneProtection",
        31 => "ItemManaUpdatePending",
        32 => "GeneratorStatus",
        33 => "ResetMessagePending",
        34 => "DefaultOpen",
        35 => "DefaultLocked",
        36 => "DefaultOn",
        37 => "OpenForBusiness",
        38 => "IsFrozen",
        39 => "DealMagicalItems",
        40 => "LogoffImDead",
        41 => "ReportCollisionsAsEnvironment",
        42 => "AllowEdgeSlide",
        43 => "AdvocateQuest",
        44 => "IsAdmin",
        45 => "IsArch",
        46 => "IsSentinel",
        47 => "IsAdvocate",
        48 => "CurrentlyPoweringUp",
        49 => "GeneratorEnteredWorld",
        50 => "NeverFailCasting",
        51 => "VendorService",
        52 => "AiImmobile",
        53 => "DamagedByCollisions",
        54 => "IsDynamic",
        55 => "IsHot",
        56 => "IsAffecting",
        57 => "AffectsAis",
        58 => "SpellQueueActive",
        59 => "GeneratorDisabled",
        60 => "IsAcceptingTells",
        61 => "LoggingChannel",
        62 => "OpensAnyLock",
        63 => "UnlimitedUse",
        64 => "GeneratedTreasureItem",
        65 => "IgnoreMagicResist",
        66 => "IgnoreMagicArmor",
        67 => "AiAllowTrade",
        68 => "SpellComponentsRequired",
        69 => "IsSellable",
        70 => "IgnoreShieldsBySkill",
        71 => "NoDraw",
        72 => "ActivationUntargeted",
        73 => "HouseHasGottenPriorityBootPos",
        74 => "GeneratorAutomaticDestruction",
        75 => "HouseHooksVisible",
        76 => "HouseRequiresMonarch",
        77 => "HouseHooksEnabled",
        78 => "HouseNotifiedHudOfHookCount",
        79 => "AiAcceptEverything",
        80 => "IgnorePortalRestrictions",
        81 => "RequiresBackpackSlot",
        82 => "DontTurnOrMoveWhenGiving",
        83 => "NpcLooksLikeObject",
        84 => "IgnoreCloIcons",
        85 => "AppraisalHasAllowedWielder",
        86 => "ChestRegenOnClose",
        87 => "LogoffInMinigame",
        88 => "PortalShowDestination",
        89 => "PortalIgnoresPkAttackTimer",
        90 => "NpcInteractsSilently",
        91 => "Retained",
        92 => "IgnoreAuthor",
        93 => "Limbo",
        94 => "AppraisalHasAllowedActivator",
        95 => "ExistedBeforeAllegianceXpChanges",
        96 => "IsDeaf",
        97 => "IsPsr",
        98 => "Invincible",
        99 => "Ivoryable",
        100 => "Dyable",
        101 => "CanGenerateRare",
        102 => "CorpseGeneratedRare",
        103 => "NonProjectileMagicImmune",
        104 => "ActdReceivedItems",
        105 => "Unknown105",
        106 => "FirstEnterWorldDone",
        107 => "RecallsDisabled",
        108 => "RareUsesTimer",
        109 => "ActdPreorderReceivedItems",
        110 => "Afk",
        111 => "IsGagged",
        112 => "ProcSpellSelfTargeted",
        113 => "IsAllegianceGagged",
        114 => "EquipmentSetTriggerPiece",
        115 => "Uninscribe",
        116 => "WieldOnUse",
        117 => "ChestClearedWhenClosed",
        118 => "NeverAttack",
        119 => "SuppressGenerateEffect",
        120 => "TreasureCorpse",
        121 => "EquipmentSetAddLevel",
        122 => "BarberActive",
        123 => "TopLayerPriority",
        124 => "NoHeldItemShown",
        125 => "LoginAtLifestone",
        126 => "OlthoiPk",
        127 => "Account15Days",
        128 => "HadNoVitae",
        129 => "NoOlthoiTalk",
        130 => "AutowieldLeft",
        _ => return format!("Bool_{}", key),
    }.to_string()
}

pub fn property_float_name(key: u32) -> String {
    match key {
        1 => "HeartbeatInterval",
        2 => "HeartbeatTimestamp",
        3 => "HealthRate",
        4 => "StaminaRate",
        5 => "ManaRate",
        6 => "HealthUponResurrection",
        7 => "ManaUponResurrection",
        8 => "StaminaUponResurrection",
        9 => "StartTime",
        10 => "StopTime",
        11 => "ResetInterval",
        12 => "Shade",
        13 => "ArmorModVsSlash",
        14 => "ArmorModVsPierce",
        15 => "ArmorModVsBludgeon",
        16 => "ArmorModVsCold",
        17 => "ArmorModVsFire",
        18 => "ArmorModVsAcid",
        19 => "ArmorModVsElectric",
        20 => "ArmorModVsNether",
        21 => "CombatSpeed",
        22 => "WeaponLength",
        23 => "DamageVariance",
        24 => "CurrentPowerMod",
        25 => "AccuracyMod",
        26 => "StrengthMod",
        27 => "MaximumVelocity",
        28 => "RotationSpeed",
        29 => "MotionTimestamp",
        30 => "WeaponDefense",
        31 => "WimpyLevel",
        32 => "VisualAwarenessRange",
        33 => "AuralAwarenessRange",
        34 => "PerceptionLevel",
        35 => "PowerUptime",
        36 => "MaxCarryWeight",
        37 => "RegenerationInterval",
        38 => "WeaponOffense",
        39 => "LifestoneProtectionTimestamp",
        40 => "PkTimestamp",
        41 => "ObjScale",
        42 => "BulkMod",
        43 => "SizeMod",
        44 => "GagTimestamp",
        45 => "GeneratorRadius",
        46 => "TimeToRot",
        47 => "DeathTimestamp",
        48 => "PkTimerDuration",
        57 => "Friction",
        58 => "Elasticity",
        59 => "Translucency",
        60 => "VelocityX",
        61 => "VelocityY",
        62 => "VelocityZ",
        63 => "OmegaX",
        64 => "OmegaY",
        65 => "OmegaZ",
        66 => "DefaultScale",
        67 => "StolenTimestamp",
        68 => "LoginTimestamp",
        69 => "CreationTimestamp",
        70 => "PkTimerResetTimestamp",
        71 => "CastingDelay",
        72 => "StartMissileAttackTimestamp",
        73 => "IgnoreShield",
        74 => "ElementalDamageMod",
        77 => "ManaCost",
        78 => "ModificationTimestamp",
        79 => "AnchorTimestamp",
        80 => "MeleeDefenseMod",
        81 => "MissileDefenseMod",
        82 => "LifeResistMod",
        83 => "CriticalChance",
        84 => "CriticalMultiplier",
        85 => "PkDamageResistMod",
        86 => "PkDamageMod",
        91 => "ManaConversionMod",
        92 => "HealOverTime",
        93 => "ManaStoneChance",
        94 => "SpecializationSkillUsage",
        95 => "CreationModifier",
        96 => "MagicResistance",
        97 => "CloakMod",
        98 => "FellowshipMod",
        99 => "ResistCold",
        100 => "ResistFire",
        101 => "ResistAcid",
        102 => "ResistElectric",
        103 => "ResistNether",
        134 => "WeaponMissileDefense",
        135 => "WeaponMagicDefense",
        136 => "ManaConversionTarget",
        137 => "EnchantmentTimestamp",
        138 => "RatingManaRegen",
        139 => "PCAPRecordedVelocityX",
        140 => "PCAPRecordedVelocityY",
        141 => "PCAPRecordedVelocityZ",
        150 => "SneakAttackMod",
        151 => "EnchantmentTarget",
        152 => "Unknown152",
        153 => "Unknown153",
        154 => "Unknown154",
        _ => return format!("Float_{}", key),
    }.to_string()
}

pub fn property_string_name(key: u32) -> String {
    // Values from ACProtocol/protocol.xml PropertyString enum
    match key {
        1 => "Name",
        2 => "Title",
        3 => "Sex",
        4 => "HeritageGroup",
        5 => "Template",
        6 => "AttackersName",
        7 => "Inscription",
        8 => "ScribeName",
        9 => "VendorsName",
        10 => "Fellowship",
        11 => "MonarchsName",
        12 => "LockCode",
        13 => "KeyCode",
        14 => "Use",
        15 => "ShortDesc",
        16 => "LongDesc",
        17 => "ActivationTalk",
        18 => "UseMessage",
        19 => "ItemHeritageGroupRestriction",
        20 => "PluralName",
        21 => "MonarchsTitle",
        22 => "ActivationFailure",
        23 => "ScribeAccount",
        24 => "TownName",
        25 => "CraftsmanName",
        26 => "UsePkServerError",
        27 => "ScoreCachedText",
        28 => "ScoreDefaultEntryFormat",
        29 => "ScoreFirstEntryFormat",
        30 => "ScoreLastEntryFormat",
        31 => "ScoreOnlyEntryFormat",
        32 => "ScoreNoEntry",
        33 => "Quest",
        34 => "GeneratorEvent",
        35 => "PatronsTitle",
        36 => "HouseOwnerName",
        37 => "QuestRestriction",
        38 => "AppraisalPortalDestination",
        39 => "TinkerName",
        40 => "ImbuerName",
        41 => "HouseOwnerAccount",
        42 => "DisplayName",
        43 => "DateOfBirth",
        44 => "ThirdPartyApi",
        45 => "KillQuest",
        46 => "Afk",
        47 => "AllegianceName",
        48 => "AugmentationAddQuest",
        49 => "KillQuest2",
        50 => "KillQuest3",
        51 => "UseSendsSignal",
        52 => "GearPlatingName",
        _ => return format!("String_{}", key),
    }.to_string()
}

pub fn property_dataid_name(key: u32) -> String {
    match key {
        1 => "Setup",
        2 => "MotionTable",
        3 => "SoundTable",
        4 => "CombatTable",
        5 => "QualityFilter",
        6 => "PaletteBase",
        7 => "ClothingBase",
        8 => "Icon",
        9 => "EyesTexture",
        10 => "NoseTexture",
        11 => "MouthTexture",
        12 => "DefaultEyesTexture",
        13 => "DefaultNoseTexture",
        14 => "DefaultMouthTexture",
        15 => "HairTexture",
        16 => "DefaultHairTexture",
        17 => "HeadObject",
        18 => "ActivationAnimation",
        19 => "InitMotion",
        20 => "ActivationSound",
        21 => "PhysicsEffectTable",
        22 => "UseSound",
        23 => "UseTargetAnimation",
        24 => "UseTargetSuccessAnimation",
        25 => "UseTargetFailureAnimation",
        26 => "UseUserAnimation",
        27 => "Spell",
        28 => "SpellComponent",
        29 => "PhysicsScript",
        30 => "EquippedPhysicsScript",
        31 => "RingCode",
        32 => "LinkedPortalOne",
        33 => "LinkedPortalTwo",
        34 => "PCAPRecordedWeenieHeader",
        35 => "PCAPRecordedWeenieHeader2",
        36 => "PCAPRecordedObjectDesc",
        37 => "PCAPRecordedPhysicsDesc",
        38 => "PCAPRecordedParentLocation",
        39 => "PCAPRecordedDefaultScript",
        40 => "PCAPRecordedDefaultScriptIntensity",
        41 => "InventoryHeiroglyphic",
        _ => return format!("DataId_{}", key),
    }.to_string()
}

/// Sound type names from ACProtocol/protocol.xml Sound enum
pub fn sound_type_name(key: u32) -> String {
    match key {
        0x00 => "Invalid",
        0x01 => "Speak1",
        0x02 => "Random",
        0x03 => "Attack1",
        0x04 => "Attack2",
        0x05 => "Attack3",
        0x06 => "SpecialAttack1",
        0x07 => "SpecialAttack2",
        0x08 => "SpecialAttack3",
        0x09 => "Damage1",
        0x0A => "Damage2",
        0x0B => "Damage3",
        0x0C => "Wound1",
        0x0D => "Wound2",
        0x0E => "Wound3",
        0x0F => "Death1",
        0x10 => "Death2",
        0x11 => "Death3",
        0x12 => "Grunt1",
        0x13 => "Grunt2",
        0x14 => "Grunt3",
        0x15 => "Oh1",
        0x16 => "Oh2",
        0x17 => "Oh3",
        0x18 => "Heave1",
        0x19 => "Heave2",
        0x1A => "Heave3",
        0x1B => "Knockdown1",
        0x1C => "Knockdown2",
        0x1D => "Knockdown3",
        0x1E => "Swoosh1",
        0x1F => "Swoosh2",
        0x20 => "Swoosh3",
        0x21 => "Thump1",
        0x22 => "Smash1",
        0x23 => "Scratch1",
        0x24 => "Spear",
        0x25 => "Sling",
        0x26 => "Dagger",
        0x27 => "ArrowWhiz1",
        0x28 => "ArrowWhiz2",
        0x29 => "CrossbowPull",
        0x2A => "CrossbowRelease",
        0x2B => "BowPull",
        0x2C => "BowRelease",
        0x2D => "ThrownWeaponRelease1",
        0x2E => "ArrowLand",
        0x2F => "Collision",
        0x30 => "HitFlesh1",
        0x31 => "HitLeather1",
        0x32 => "HitChain1",
        0x33 => "HitPlate1",
        0x34 => "HitMissile1",
        0x35 => "HitMissile2",
        0x36 => "HitMissile3",
        0x37 => "Footstep1",
        0x38 => "Footstep2",
        0x39 => "Walk1",
        0x3A => "Dance1",
        0x3B => "Dance2",
        0x3C => "Dance3",
        0x3D => "Hidden1",
        0x3E => "Hidden2",
        0x3F => "Hidden3",
        0x40 => "Eat1",
        0x41 => "Drink1",
        0x42 => "Open",
        0x43 => "Close",
        0x44 => "OpenSlam",
        0x45 => "CloseSlam",
        0x46 => "Ambient1",
        0x47 => "Ambient2",
        0x48 => "Ambient3",
        0x49 => "Ambient4",
        0x4A => "Ambient5",
        0x4B => "Ambient6",
        0x4C => "Ambient7",
        0x4D => "Ambient8",
        0x4E => "Waterfall",
        0x4F => "LogOut",
        0x50 => "LogIn",
        0x51 => "LifestoneOn",
        0x52 => "AttribUp",
        0x53 => "AttribDown",
        0x54 => "SkillUp",
        0x55 => "SkillDown",
        0x56 => "HealthUp",
        0x57 => "HealthDown",
        0x58 => "ShieldUp",
        0x59 => "ShieldDown",
        0x5A => "EnchantUp",
        0x5B => "EnchantDown",
        0x5C => "VisionUp",
        0x5D => "VisionDown",
        0x5E => "Fizzle",
        0x5F => "Launch",
        0x60 => "Explode",
        0x61 => "TransUp",
        0x62 => "TransDown",
        0x63 => "BreatheFlaem",
        0x64 => "BreatheAcid",
        0x65 => "BreatheFrost",
        0x66 => "BreatheLightning",
        0x67 => "Create",
        0x68 => "Destroy",
        0x69 => "Lockpicking",
        0x6A => "UI_EnterPortal",
        0x6B => "UI_ExitPortal",
        0x6C => "UI_GeneralQuery",
        0x6D => "UI_GeneralError",
        0x6E => "UI_TransientMessage",
        0x6F => "UI_IconPickUp",
        0x70 => "UI_IconSuccessfulDrop",
        0x71 => "UI_IconInvalid_Drop",
        0x72 => "UI_ButtonPress",
        0x73 => "UI_GrabSlider",
        0x74 => "UI_ReleaseSlider",
        0x75 => "UI_NewTargetSelected",
        0x76 => "UI_Roar",
        0x77 => "UI_Bell",
        0x78 => "UI_Chant1",
        0x79 => "UI_Chant2",
        0x7A => "UI_DarkWhispers1",
        0x7B => "UI_DarkWhispers2",
        0x7C => "UI_DarkLaugh",
        0x7D => "UI_DarkWind",
        0x7E => "UI_DarkSpeech",
        0x7F => "UI_Drums",
        0x80 => "UI_GhostSpeak",
        0x81 => "UI_Breathing",
        0x82 => "UI_Howl",
        0x83 => "UI_LostSouls",
        0x84 => "UI_Squeal",
        0x85 => "UI_Thunder1",
        0x86 => "UI_Thunder2",
        0x87 => "UI_Thunder3",
        0x88 => "UI_Thunder4",
        0x89 => "UI_Thunder5",
        0x8A => "UI_Thunder6",
        0x8B => "RaiseTrait",
        0x8C => "WieldObject",
        0x8D => "UnwieldObject",
        0x8E => "ReceiveItem",
        0x8F => "PickUpItem",
        0x90 => "DropItem",
        0x91 => "ResistSpell",
        0x92 => "PicklockFail",
        0x93 => "LockSuccess",
        0x94 => "OpenFailDueToLock",
        0x95 => "TriggerActivated",
        0x96 => "SpellExpire",
        0x97 => "ItemManaDepleted",
        // 0x98-0xC9 = TriggerActivated[1-50] - handled below
        0xCA => "HealthDownVoid",
        0xCB => "RegenDownVoid",
        0xCC => "SkillDownVoid",
        _ => {
            // Handle TriggerActivated range (0x98-0xC9)
            if key >= 0x98 && key <= 0xC9 {
                return format!("TriggerActivated{}", key - 0x97);
            }
            return format!("Sound_{}", key);
        }
    }.to_string()
}
