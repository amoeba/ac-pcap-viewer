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
        201 => "MeleeDefenseImbuedEffectTypeCache", 202 => "MissileDefenseImbuedEffectTypeCache",
        203 => "MagicDefenseImbuedEffectTypeCache", 204 => "ElementalDamageBonus",
        205 => "ImbueAttempts", 206 => "ImbueSuccesses", 207 => "CreatureKills",
        208 => "PlayerKillsPk", 209 => "PlayerKillsPkl",
        210 => "RaresTierOne", 211 => "RaresTierTwo", 212 => "RaresTierThree",
        213 => "RaresTierFour", 214 => "RaresTierFive",
        215 => "AugmentationStat", 216 => "AugmentationFamilyStat", 217 => "AugmentationInnateFamily",
        218 => "AugmentationInnateStrength", 219 => "AugmentationInnateEndurance",
        220 => "AugmentationInnateCoordination", 221 => "AugmentationInnateQuickness",
        222 => "AugmentationInnateFocus", 223 => "AugmentationInnateSelf",
        224 => "AugmentationSpecializeSalvaging", 225 => "AugmentationSpecializeItemTinkering",
        226 => "AugmentationSpecializeArmorTinkering", 227 => "AugmentationSpecializeMagicItemTinkering",
        228 => "AugmentationSpecializeWeaponTinkering", 229 => "AugmentationExtraPackSlot",
        230 => "AugmentationIncreasedCarryingCapacity", 231 => "AugmentationLessDeathItemLoss",
        232 => "AugmentationSpellsRemainPastDeath", 233 => "AugmentationCriticalDefense",
        234 => "AugmentationBonusXp", 235 => "AugmentationBonusSalvage",
        236 => "AugmentationBonusImbueChance", 237 => "AugmentationFasterRegen",
        238 => "AugmentationIncreasedSpellDuration", 239 => "AugmentationResistanceFamily",
        240 => "AugmentationResistanceSlash", 241 => "AugmentationResistancePierce",
        242 => "AugmentationResistanceBlunt", 243 => "AugmentationResistanceAcid",
        244 => "AugmentationResistanceFire", 245 => "AugmentationResistanceFrost",
        246 => "AugmentationResistanceLightning",
        247 => "RaresTierOneLogin", 248 => "RaresTierTwoLogin", 249 => "RaresTierThreeLogin",
        250 => "RaresTierFourLogin", 251 => "RaresTierFiveLogin", 252 => "RaresLoginTimestamp",
        253 => "RaresTierSix", 254 => "RaresTierSeven",
        255 => "RaresTierSixLogin", 256 => "RaresTierSevenLogin",
        257 => "ItemAttributeLimit", 258 => "ItemAttributeLevelLimit",
        259 => "ItemAttribute2ndLimit", 260 => "ItemAttribute2ndLevelLimit",
        261 => "CharacterTitleId", 262 => "NumCharacterTitles",
        263 => "ResistanceModifierType", 264 => "FreeTinkersBitfield",
        265 => "EquipmentSetId", 266 => "PetClass", 267 => "Lifespan", 268 => "RemainingLifespan",
        269 => "UseCreateQuantity",
        270 => "WieldRequirements2", 271 => "WieldSkillType2", 272 => "WieldDifficulty2",
        273 => "WieldRequirements3", 274 => "WieldSkillType3", 275 => "WieldDifficulty3",
        276 => "WieldRequirements4", 277 => "WieldSkillType4", 278 => "WieldDifficulty4",
        279 => "Unique", 280 => "SharedCooldown",
        281 => "Faction1Bits", 282 => "Faction2Bits", 283 => "Faction3Bits",
        284 => "Hatred1Bits", 285 => "Hatred2Bits", 286 => "Hatred3Bits",
        287 => "SocietyRankCelhan", 288 => "SocietyRankEldweb", 289 => "SocietyRankRadblo",
        290 => "HearLocalSignals", 291 => "HearLocalSignalsRadius", 292 => "Cleaving",
        293 => "AugmentationSpecializeGearcraft", 294 => "AugmentationInfusedCreatureMagic",
        295 => "AugmentationInfusedItemMagic", 296 => "AugmentationInfusedLifeMagic",
        297 => "AugmentationInfusedWarMagic", 298 => "AugmentationCriticalExpertise",
        299 => "AugmentationCriticalPower", 300 => "AugmentationSkilledMelee",
        301 => "AugmentationSkilledMissile", 302 => "AugmentationSkilledMagic",
        303 => "ImbuedEffect2", 304 => "ImbuedEffect3", 305 => "ImbuedEffect4", 306 => "ImbuedEffect5",
        307 => "DamageRating", 308 => "DamageResistRating",
        309 => "AugmentationDamageBonus", 310 => "AugmentationDamageReduction",
        311 => "ImbueStackingBits", 312 => "HealOverTime",
        313 => "CritRating", 314 => "CritDamageRating",
        315 => "CritResistRating", 316 => "CritDamageResistRating",
        317 => "HealingResistRating", 318 => "DamageOverTime",
        319 => "ItemMaxLevel", 320 => "ItemXpStyle",
        321 => "EquipmentSetExtra", 322 => "AetheriaBitfield", 323 => "HealingBoostRating",
        324 => "HeritageSpecificArmor", 325 => "AlternateRacialSkills",
        326 => "AugmentationJackOfAllTrades", 327 => "AugmentationResistanceNether",
        328 => "AugmentationInfusedVoidMagic", 329 => "WeaknessRating",
        330 => "NetherOverTime", 331 => "NetherResistRating", 332 => "LuminanceAward",
        333 => "LumAugDamageRating", 334 => "LumAugDamageReductionRating",
        335 => "LumAugCritDamageRating", 336 => "LumAugCritReductionRating",
        337 => "LumAugSurgeEffectRating", 338 => "LumAugSurgeChanceRating",
        339 => "LumAugItemManaUsage", 340 => "LumAugItemManaGain",
        341 => "LumAugVitality", 342 => "LumAugHealingRating",
        343 => "LumAugSkilledCraft", 344 => "LumAugSkilledSpec", 345 => "LumAugNoDestroyCraft",
        346 => "RestrictInteraction", 347 => "OlthoiLootTimestamp", 348 => "OlthoiLootStep",
        349 => "UseCreatesContractId", 350 => "DotResistRating", 351 => "LifeResistRating",
        352 => "CloakWeaveProc", 353 => "WeaponType",
        354 => "MeleeMastery", 355 => "RangedMastery",
        356 => "SneakAttackRating", 357 => "RecklessnessRating", 358 => "DeceptionRating",
        359 => "CombatPetRange", 360 => "WeaponAuraDamage", 361 => "WeaponAuraSpeed",
        362 => "SummoningMastery", 363 => "HeartbeatLifespan", 364 => "UseLevelRequirement",
        365 => "LumAugAllSkills", 366 => "UseRequiresSkill", 367 => "UseRequiresSkillLevel",
        368 => "UseRequiresSkillSpec", 369 => "UseRequiresLevel",
        370 => "GearDamage", 371 => "GearDamageResist", 372 => "GearCrit", 373 => "GearCritResist",
        374 => "GearCritDamage", 375 => "GearCritDamageResist", 376 => "GearHealingBoost",
        377 => "GearNetherResist", 378 => "GearLifeResist", 379 => "GearMaxHealth",
        380 => "Unknown380",
        381 => "PKDamageRating", 382 => "PKDamageResistRating",
        383 => "GearPKDamageRating", 384 => "GearPKDamageResistRating",
        385 => "Unknown385",
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
    // Values from ACProtocol/protocol.xml PropertyFloat enum
    match key {
        0 => "Undef",
        1 => "HeartbeatInterval", 2 => "HeartbeatTimestamp",
        3 => "HealthRate", 4 => "StaminaRate", 5 => "ManaRate",
        6 => "HealthUponResurrection", 7 => "StaminaUponResurrection", 8 => "ManaUponResurrection",
        9 => "StartTime", 10 => "StopTime", 11 => "ResetInterval", 12 => "Shade",
        13 => "ArmorModVsSlash", 14 => "ArmorModVsPierce", 15 => "ArmorModVsBludgeon",
        16 => "ArmorModVsCold", 17 => "ArmorModVsFire", 18 => "ArmorModVsAcid",
        19 => "ArmorModVsElectric", 20 => "CombatSpeed",
        21 => "WeaponLength", 22 => "DamageVariance",
        23 => "CurrentPowerMod", 24 => "AccuracyMod", 25 => "StrengthMod",
        26 => "MaximumVelocity", 27 => "RotationSpeed", 28 => "MotionTimestamp",
        29 => "WeaponDefense", 30 => "WimpyLevel",
        31 => "VisualAwarenessRange", 32 => "AuralAwarenessRange", 33 => "PerceptionLevel",
        34 => "PowerupTime", 35 => "MaxChargeDistance", 36 => "ChargeSpeed",
        37 => "BuyPrice", 38 => "SellPrice", 39 => "DefaultScale", 40 => "LockpickMod",
        41 => "RegenerationInterval", 42 => "RegenerationTimestamp",
        43 => "GeneratorRadius", 44 => "TimeToRot", 45 => "DeathTimestamp",
        46 => "PkTimestamp", 47 => "VictimTimestamp",
        48 => "LoginTimestamp", 49 => "CreationTimestamp", 50 => "MinimumTimeSincePk",
        51 => "DeprecatedHousekeepingPriority", 52 => "AbuseLoggingTimestamp",
        53 => "LastPortalTeleportTimestamp", 54 => "UseRadius", 55 => "HomeRadius",
        56 => "ReleasedTimestamp", 57 => "MinHomeRadius", 58 => "Facing",
        59 => "ResetTimestamp", 60 => "LogoffTimestamp", 61 => "EconRecoveryInterval",
        62 => "WeaponOffense", 63 => "DamageMod",
        64 => "ResistSlash", 65 => "ResistPierce", 66 => "ResistBludgeon",
        67 => "ResistFire", 68 => "ResistCold", 69 => "ResistAcid", 70 => "ResistElectric",
        71 => "ResistHealthBoost", 72 => "ResistStaminaDrain", 73 => "ResistStaminaBoost",
        74 => "ResistManaDrain", 75 => "ResistManaBoost", 76 => "Translucency",
        77 => "PhysicsScriptIntensity", 78 => "Friction", 79 => "Elasticity",
        80 => "AiUseMagicDelay", 81 => "ItemMinSpellcraftMod", 82 => "ItemMaxSpellcraftMod",
        83 => "ItemRankProbability", 84 => "Shade2", 85 => "Shade3", 86 => "Shade4",
        87 => "ItemEfficiency", 88 => "ItemManaUpdateTimestamp",
        89 => "SpellGestureSpeedMod", 90 => "SpellStanceSpeedMod",
        91 => "AllegianceAppraisalTimestamp", 92 => "PowerLevel", 93 => "AccuracyLevel",
        94 => "AttackAngle", 95 => "AttackTimestamp", 96 => "CheckpointTimestamp",
        97 => "SoldTimestamp", 98 => "UseTimestamp", 99 => "UseLockTimestamp",
        100 => "HealkitMod", 101 => "FrozenTimestamp", 102 => "HealthRateMod",
        103 => "AllegianceSwearTimestamp", 104 => "ObviousRadarRange",
        105 => "HotspotCycleTime", 106 => "HotspotCycleTimeVariance",
        107 => "SpamTimestamp", 108 => "SpamRate", 109 => "BondWieldedTreasure",
        110 => "BulkMod", 111 => "SizeMod", 112 => "GagTimestamp",
        113 => "GeneratorUpdateTimestamp", 114 => "DeathSpamTimestamp", 115 => "DeathSpamRate",
        116 => "WildAttackProbability", 117 => "FocusedProbability",
        118 => "CrashAndTurnProbability", 119 => "CrashAndTurnRadius", 120 => "CrashAndTurnBias",
        121 => "GeneratorInitialDelay", 122 => "AiAcquireHealth",
        123 => "AiAcquireStamina", 124 => "AiAcquireMana", 125 => "ResistHealthDrain",
        126 => "LifestoneProtectionTimestamp", 127 => "AiCounteractEnchantment",
        128 => "AiDispelEnchantment", 129 => "TradeTimestamp",
        130 => "AiTargetedDetectionRadius", 131 => "EmotePriority",
        132 => "LastTeleportStartTimestamp", 133 => "EventSpamTimestamp", 134 => "EventSpamRate",
        135 => "InventoryOffset", 136 => "CriticalMultiplier", 137 => "ManaStoneDestroyChance",
        138 => "SlayerDamageBonus", 139 => "AllegianceInfoSpamTimestamp",
        140 => "AllegianceInfoSpamRate", 141 => "NextSpellcastTimestamp",
        142 => "AppraisalRequestedTimestamp", 143 => "AppraisalHeartbeatDueTimestamp",
        144 => "ManaConversionMod", 145 => "LastPkAttackTimestamp",
        146 => "FellowshipUpdateTimestamp", 147 => "CriticalFrequency",
        148 => "LimboStartTimestamp", 149 => "WeaponMissileDefense", 150 => "WeaponMagicDefense",
        151 => "IgnoreShield", 152 => "ElementalDamageMod", 153 => "StartMissileAttackTimestamp",
        154 => "LastRareUsedTimestamp", 155 => "IgnoreArmor", 156 => "ProcSpellRate",
        157 => "ResistanceModifier", 158 => "AllegianceGagTimestamp",
        159 => "AbsorbMagicDamage", 160 => "CachedMaxAbsorbMagicDamage",
        161 => "GagDuration", 162 => "AllegianceGagDuration", 163 => "GlobalXpMod",
        164 => "HealingModifier", 165 => "ArmorModVsNether", 166 => "ResistNether",
        167 => "CooldownDuration", 168 => "WeaponAuraOffense", 169 => "WeaponAuraDefense",
        170 => "WeaponAuraElemental", 171 => "WeaponAuraManaConv",
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
    // Values from ACProtocol/protocol.xml PropertyDataId enum
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
        15 => "HairPalette",
        16 => "EyesPalette",
        17 => "SkinPalette",
        18 => "HeadObject",
        19 => "ActivationAnimation",
        20 => "InitMotion",
        21 => "ActivationSound",
        22 => "PhysicsEffectTable",
        23 => "UseSound",
        24 => "UseTargetAnimation",
        25 => "UseTargetSuccessAnimation",
        26 => "UseTargetFailureAnimation",
        27 => "UseUserAnimation",
        28 => "Spell",
        29 => "SpellComponent",
        30 => "PhysicsScript",
        31 => "LinkedPortalOne",
        32 => "WieldedTreasureType",
        33 => "InventoryTreasureType",
        34 => "ShopTreasureType",
        35 => "DeathTreasureType",
        36 => "MutateFilter",
        37 => "ItemSkillLimit",
        38 => "UseCreateItem",
        39 => "DeathSpell",
        40 => "VendorsClassId",
        41 => "ItemSpecializedOnly",
        42 => "HouseId",
        43 => "AccountHouseId",
        44 => "RestrictionEffect",
        45 => "CreationMutationFilter",
        46 => "TsysMutationFilter",
        47 => "LastPortal",
        48 => "LinkedPortalTwo",
        49 => "OriginalPortal",
        50 => "IconOverlay",
        51 => "IconOverlaySecondary",
        52 => "IconUnderlay",
        53 => "AugmentationMutationFilter",
        54 => "AugmentationEffect",
        55 => "ProcSpell",
        56 => "AugmentationCreateItem",
        57 => "AlternateCurrency",
        58 => "BlueSurgeSpell",
        59 => "YellowSurgeSpell",
        60 => "RedSurgeSpell",
        61 => "OlthoiDeathTreasureType",
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

// ============================================================================
// CHILD ENUM MAPPING FUNCTIONS
// These are enums referenced by PropertyInt values via enum= attribute
// Generated from protocol.xml enum definitions
// ============================================================================

/// ItemType enum (mask values)
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
        _ => return format!("ItemType_{}", key),
    }.to_string()
}

/// CreatureType enum
pub fn creature_type_name(key: u32) -> String {
    match key {
        1 => "Olthoi", 2 => "Banderling", 3 => "Drudge", 4 => "Mosswart",
        5 => "Lugian", 6 => "Tumerok", 7 => "Mite", 8 => "Tusker",
        9 => "PhyntosWasp", 10 => "Rat", 11 => "Auroch", 12 => "Cow",
        13 => "Golem", 14 => "Undead", 15 => "Gromnie", 16 => "Reedshark",
        17 => "Armoredillo", 18 => "Fae", 19 => "Virindi", 20 => "Wisp",
        21 => "Knathtead", 22 => "Shadow", 23 => "Mattekar", 24 => "Mumiyah",
        25 => "Rabbit", 26 => "Sclavus", 27 => "ShallowsShark", 28 => "Monouga",
        29 => "Zefir", 30 => "Skeleton", 31 => "Human", 32 => "Shreth",
        33 => "Chittick", 34 => "Moarsman", 35 => "OlthoiLarvae", 36 => "Slithis",
        37 => "Deru", 38 => "FireElemental", 39 => "Snowman", 40 => "Unknown",
        41 => "Bunny", 42 => "LightningElemental", 43 => "Rockslide", 44 => "Grievver",
        45 => "Niffis", 46 => "Ursuin", 47 => "Crystal", 48 => "HollowMinion",
        49 => "Scarecrow", 50 => "Idol", 51 => "Empyrean", 52 => "Hopeslayer",
        53 => "Doll", 54 => "Marionette", 55 => "Carenzi", 56 => "Siraluun",
        57 => "AunTumerok", 58 => "HeaTumerok", 59 => "Simulacrum", 60 => "AcidElemental",
        61 => "FrostElemental", 62 => "Elemental", 63 => "Statue", 64 => "Wall",
        65 => "AlteredHuman", 66 => "Device", 67 => "Harbinger", 68 => "DarkSarcophagus",
        69 => "Chicken", 70 => "GotrokLugian", 71 => "Margul", 72 => "BleachedRabbit",
        73 => "NastyRabbit", 74 => "GrimacingRabbit", 75 => "Burun", 76 => "Target",
        77 => "Ghost", 78 => "Fiun", 79 => "Eater", 80 => "Penguin",
        81 => "Ruschk", 82 => "Thrungus", 83 => "ViamontianKnight", 84 => "Remoran",
        85 => "Swarm", 86 => "Moar", 87 => "EnchantedArms", 88 => "Sleech",
        89 => "Mukkir", 90 => "Merwart", 91 => "Food", 92 => "ParadoxOlthoi",
        93 => "Harvest", 94 => "Energy", 95 => "Apparition", 96 => "Aerbax",
        97 => "Touched", 98 => "BlightedMoarsman", 99 => "GearKnight", 100 => "Gurog",
        101 => "Anekshay",
        _ => return format!("CreatureType_{}", key),
    }.to_string()
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
        _ => return format!("DamageType_{}", key),
    }.to_string()
}

/// SkillId enum
pub fn skill_id_name(key: u32) -> String {
    match key {
        0x01 => "Axe", 0x02 => "Bow", 0x03 => "Crossbow", 0x04 => "Dagger",
        0x05 => "Mace", 0x06 => "MeleeDefense", 0x07 => "MissileDefense", 0x08 => "Sling",
        0x09 => "Spear", 0x0A => "Staff", 0x0B => "Sword", 0x0C => "ThrownWeapons",
        0x0D => "UnarmedCombat", 0x0E => "ArcaneLore", 0x0F => "MagicDefense",
        0x10 => "ManaConversion", 0x11 => "Spellcraft", 0x12 => "ItemTinkering",
        0x13 => "AssessPerson", 0x14 => "Deception", 0x15 => "Healing", 0x16 => "Jump",
        0x17 => "Lockpick", 0x18 => "Run", 0x19 => "Awareness", 0x1A => "ArmorRepair",
        0x1B => "AssessCreature", 0x1C => "WeaponTinkering", 0x1D => "ArmorTinkering",
        0x1E => "MagicItemTinkering", 0x1F => "CreatureEnchantment", 0x20 => "ItemEnchantment",
        0x21 => "LifeMagic", 0x22 => "WarMagic", 0x23 => "Leadership", 0x24 => "Loyalty",
        0x25 => "Fletching", 0x26 => "Alchemy", 0x27 => "Cooking", 0x28 => "Salvaging",
        0x29 => "TwoHandedCombat", 0x2A => "Gearcraft", 0x2B => "VoidMagic",
        0x2C => "HeavyWeapons", 0x2D => "LightWeapons", 0x2E => "FinesseWeapons",
        0x2F => "MissileWeapons", 0x31 => "DualWield", 0x32 => "Recklessness",
        0x33 => "SneakAttack", 0x34 => "DirtyFighting", 0x35 => "Challenge", 0x36 => "Summoning",
        _ => return format!("SkillId_{}", key),
    }.to_string()
}

/// EquipmentSet enum
pub fn equipment_set_name(key: u32) -> String {
    match key {
        0 => "None", 1 => "Test", 2 => "Test2", 3 => "Unknown3",
        4 => "CarraidasBenediction", 5 => "NobleRelic", 6 => "AncientRelic", 7 => "AlduressaRelic",
        8 => "Ninja", 9 => "EmpyreanRings", 10 => "ArmMindHeart", 11 => "ArmorPerfectLight",
        12 => "ArmorPerfectLight2", 13 => "Soldiers", 14 => "Adepts", 15 => "Archers",
        16 => "Defenders", 17 => "Tinkers", 18 => "Crafters", 19 => "Hearty",
        20 => "Dexterous", 21 => "Wise", 22 => "Swift", 23 => "Hardened",
        24 => "Reinforced", 25 => "Interlocking", 26 => "Flameproof", 27 => "Acidproof",
        28 => "Coldproof", 29 => "Lightningproof", 30 => "SocietyArmor", 31 => "ColosseumClothing",
        32 => "GraveyardClothing", 33 => "OlthoiClothing", 34 => "NoobieArmor",
        35 => "AetheriaDefense", 36 => "AetheriaDestruction", 37 => "AetheriaFury",
        38 => "AetheriaGrowth", 39 => "AetheriaVigor", 40 => "RareDamageResistance",
        41 => "RareDamageBoost", 42..=48 => "OlthoiArmor_Variant",
        49..=90 => "CloakSkill_Variant", 91..=129 => "ShroudedSoul_Variant",
        130 => "ShimmeringShadowsSet", 131..=136 => "SocietyAccessory_Variant",
        137 => "GauntletGarb", 138 => "ParagonMissile", 139 => "ParagonCaster", 140 => "ParagonMelee",
        _ => return format!("EquipmentSet_{}", key),
    }.to_string()
}

/// EquipMask enum (mask values)
pub fn equip_mask_name(key: u32) -> String {
    match key {
        0x00000001 => "Head", 0x00000002 => "ChestUnderwear", 0x00000004 => "AbdomenUnderwear",
        0x00000008 => "UpperArmsUnderwear", 0x00000010 => "LowerArmsUnderwear", 0x00000020 => "Hands",
        0x00000040 => "UpperLegsUnderwear", 0x00000080 => "LowerLegsUnderwear", 0x00000100 => "Feet",
        0x00000200 => "Chest", 0x00000400 => "Abdomen", 0x00000800 => "UpperArms",
        0x00001000 => "LowerArms", 0x00002000 => "UpperLegs", 0x00004000 => "LowerLegs",
        0x00008000 => "Necklace", 0x00010000 => "RightBracelet", 0x00020000 => "LeftBracelet",
        0x00040000 => "RightRing", 0x00080000 => "LeftRing", 0x00100000 => "MeleeWeapon",
        0x00200000 => "Shield", 0x00400000 => "MissileWeapon", 0x00800000 => "Ammunition",
        0x01000000 => "Wand",
        _ => return format!("EquipMask_{}", key),
    }.to_string()
}

/// CombatMode enum
pub fn combat_mode_name(key: u32) -> String {
    match key {
        0x1 => "NonCombat", 0x2 => "Melee", 0x4 => "Missile", 0x8 => "Magic",
        _ => return format!("CombatMode_{}", key),
    }.to_string()
}

/// HeritageGroup enum
pub fn heritage_group_name(key: u32) -> String {
    match key {
        0 => "Invalid", 1 => "Aluvian", 2 => "Gharundim", 3 => "Sho",
        4 => "Viamontian", 5 => "Shadowbound", 6 => "Gearknight", 7 => "Tumerok",
        8 => "Lugian", 9 => "Empyrean", 10 => "Penumbraen", 11 => "Undead",
        12 => "Olthoi", 13 => "OlthoiAcid",
        _ => return format!("HeritageGroup_{}", key),
    }.to_string()
}

/// WeaponType enum
pub fn weapon_type_name(key: u32) -> String {
    match key {
        0 => "Undef", 1 => "Unarmed", 2 => "Sword", 3 => "Axe",
        4 => "Mace", 5 => "Spear", 6 => "Dagger", 7 => "Staff",
        8 => "Bow", 9 => "Crossbow", 10 => "Thrown", 11 => "TwoHanded", 12 => "Magic",
        _ => return format!("WeaponType_{}", key),
    }.to_string()
}

/// MaterialType enum
pub fn material_type_name(key: u32) -> String {
    match key {
        1 => "Ceramic", 2 => "Porcelain", 4 => "Linen", 5 => "Satin", 6 => "Silk",
        7 => "Velvet", 8 => "Wool", 10 => "Agate", 11 => "Amber", 12 => "Amethyst",
        13 => "Aquamarine", 14 => "Azurite", 15 => "BlackGarnet", 16 => "BlackOpal",
        17 => "Bloodstone", 18 => "Carnelian", 19 => "Citrine", 20 => "Diamond",
        21 => "Emerald", 22 => "FireOpal", 23 => "GreenGarnet", 24 => "GreenJade",
        25 => "Hematite", 26 => "ImperialTopaz", 27 => "Jet", 28 => "LapisLazuli",
        29 => "LavenderJade", 30 => "Malachite", 31 => "Moonstone", 32 => "Onyx",
        33 => "Opal", 34 => "Peridot", 35 => "RedGarnet", 36 => "RedJade",
        37 => "RoseQuartz", 38 => "Ruby", 39 => "Sapphire", 40 => "SmokeyQuartz",
        41 => "Sunstone", 42 => "TigerEye", 43 => "Tourmaline", 44 => "Turquoise",
        45 => "WhiteJade", 46 => "WhiteQuartz", 47 => "WhiteSapphire", 48 => "YellowGarnet",
        49 => "YellowTopaz", 50 => "Zircon", 51 => "Ivory", 52 => "Leather",
        53 => "ArmoredilloHide", 54 => "GromnieHide", 55 => "ReedSharkHide",
        57 => "Brass", 58 => "Bronze", 59 => "Copper", 60 => "Gold",
        61 => "Iron", 62 => "Pyreal", 63 => "Silver", 64 => "Steel",
        66 => "Alabaster", 67 => "Granite", 68 => "Marble", 69 => "Obsidian",
        70 => "Sandstone", 71 => "Serpentine", 73 => "Ebony", 74 => "Mahogany",
        75 => "Oak", 76 => "Pine", 77 => "Teak",
        _ => return format!("MaterialType_{}", key),
    }.to_string()
}

/// Gender enum
pub fn gender_name(key: u32) -> String {
    match key {
        0 => "Invalid", 1 => "Male", 2 => "Female",
        _ => return format!("Gender_{}", key),
    }.to_string()
}

/// AttackType enum (mask values)
pub fn attack_type_name(key: u32) -> String {
    match key {
        0x0000 => "Undef", 0x0001 => "Punch", 0x0002 => "Thrust", 0x0004 => "Slash",
        0x0008 => "Kick", 0x0010 => "OffhandPunch", 0x0020 => "DoubleSlash",
        0x0040 => "TripleSlash", 0x0080 => "DoubleThrust", 0x0100 => "TripleThrust",
        0x0200 => "OffhandThrust", 0x0400 => "OffhandSlash", 0x0800 => "OffhandDoubleSlash",
        0x1000 => "OffhandTripleSlash", 0x2000 => "OffhandDoubleThrust", 0x4000 => "OffhandTripleThrust",
        _ => return format!("AttackType_{}", key),
    }.to_string()
}

/// AttackHeight enum
pub fn attack_height_name(key: u32) -> String {
    match key {
        0x01 => "High", 0x02 => "Medium", 0x03 => "Low",
        _ => return format!("AttackHeight_{}", key),
    }.to_string()
}

/// CombatStyle enum (mask values)
pub fn combat_style_name(key: u32) -> String {
    match key {
        0x00000 => "Undef", 0x00001 => "Unarmed", 0x00002 => "OneHanded",
        0x00004 => "OneHandedAndShield", 0x00008 => "TwoHanded", 0x00010 => "Bow",
        0x00020 => "Crossbow", 0x00040 => "Sling", 0x00080 => "ThrownWeapon",
        0x00100 => "DualWield", 0x00200 => "Magic", 0x00400 => "Atlatl",
        0x00800 => "ThrownShield", 0x10000 => "StubbornMagic", 0x20000 => "StubbornProjectile",
        0x40000 => "StubbornMelee", 0x80000 => "StubbornMissile",
        _ => return format!("CombatStyle_{}", key),
    }.to_string()
}

/// Placement enum
pub fn placement_name(key: u32) -> String {
    match key {
        0 => "Default", 1 => "RightHandCombat", 2 => "RightHandNonCombat", 3 => "LeftHand",
        4 => "Belt", 5 => "Quiver", 6 => "Shield", 7 => "LeftWeapon", 8 => "LeftUnarmed",
        0x33 => "SpecialCrossbowBolt", 0x34 => "MissileFlight", 0x65 => "Resting",
        0x66 => "Other", 0x67 => "Hook",
        _ => return format!("Placement_{}", key),
    }.to_string()
}

/// WieldRequirement enum
pub fn wield_requirement_name(key: u32) -> String {
    match key {
        0 => "Undef", 1 => "Skill", 2 => "RawSkill", 3 => "Attrib", 4 => "RawAttrib",
        5 => "SecondaryAttrib", 6 => "RawSecondaryAttrib", 7 => "Level", 8 => "Training",
        9 => "IntStat", 10 => "BoolStat", 11 => "CreatureType", 12 => "HeritageType",
        _ => return format!("WieldRequirement_{}", key),
    }.to_string()
}

/// CoverageMask enum (mask values)
pub fn coverage_mask_name(key: u32) -> String {
    match key {
        0x00000002 => "UpperLegsUnderwear", 0x00000004 => "LowerLegsUnderwear",
        0x00000008 => "ChestUnderwear", 0x00000010 => "AbdomenUnderwear",
        0x00000020 => "UpperArmsUnderwear", 0x00000040 => "LowerArmsUnderwear",
        0x00000100 => "UpperLegs", 0x00000200 => "LowerLegs", 0x00000400 => "Chest",
        0x00000800 => "Abdomen", 0x00001000 => "UpperArms", 0x00002000 => "LowerArms",
        0x00004000 => "Head", 0x00008000 => "Hands", 0x00010000 => "Feet",
        _ => return format!("CoverageMask_{}", key),
    }.to_string()
}

/// AmmoType enum
pub fn ammo_type_name(key: u32) -> String {
    match key {
        0 => "None", 1 => "Arrow", 2 => "Bolt", 4 => "Atlatl", 8 => "ThrownWeapon",
        _ => return format!("AmmoType_{}", key),
    }.to_string()
}

/// CombatUse enum
pub fn combat_use_name(key: u32) -> String {
    match key {
        0 => "None", 1 => "Melee", 2 => "Missile", 4 => "Ammo", 8 => "Shield", 16 => "TwoHanded",
        _ => return format!("CombatUse_{}", key),
    }.to_string()
}

/// ParentLocation enum
pub fn parent_location_name(key: u32) -> String {
    match key {
        0 => "None", 1 => "RightHand", 2 => "LeftHand", 3 => "Shield", 4 => "Belt",
        5 => "Quiver", 6 => "Hearldry", 7 => "Mouth", 8 => "LeftWeapon", 9 => "LeftUnarmed",
        _ => return format!("ParentLocation_{}", key),
    }.to_string()
}

/// RadarColor enum
pub fn radar_color_name(key: u32) -> String {
    match key {
        0 => "Default", 1 => "Blue", 2 => "Gold", 3 => "White", 4 => "Purple",
        5 => "Red", 6 => "Pink", 7 => "Green", 8 => "Yellow", 9 => "Cyan", 10 => "BrightGreen",
        _ => return format!("RadarColor_{}", key),
    }.to_string()
}

/// RadarBehavior enum
pub fn radar_behavior_name(key: u32) -> String {
    match key {
        0 => "Undefined", 1 => "ShowNever", 2 => "ShowMovement", 3 => "ShowAttacking", 4 => "ShowAlways",
        _ => return format!("RadarBehavior_{}", key),
    }.to_string()
}

/// ArmorType enum
pub fn armor_type_name(key: u32) -> String {
    match key {
        0 => "None", 1 => "Cloth", 2 => "Leather", 4 => "StuddedLeather",
        8 => "Scalemail", 16 => "Chainmail", 32 => "Metal",
        _ => return format!("ArmorType_{}", key),
    }.to_string()
}

/// ImbuedEffectType enum (mask values)
pub fn imbued_effect_type_name(key: u32) -> String {
    match key {
        0 => "Undef", 0x0001 => "CriticalStrike", 0x0002 => "CripplingBlow",
        0x0004 => "ArmorRending", 0x0008 => "SlashRending", 0x0010 => "PierceRending",
        0x0020 => "BludgeonRending", 0x0040 => "AcidRending", 0x0080 => "ColdRending",
        0x0100 => "ElectricRending", 0x0200 => "FireRending", 0x0400 => "MeleeDefense",
        0x0800 => "MissileDefense", 0x1000 => "MagicDefense", 0x2000 => "Spellbook",
        0x4000 => "NetherRending", 0x20000000 => "IgnoreSomeMagicProjectileDamage",
        0x40000000 => "AlwaysCritical", 0x80000000 => "IgnoreAllArmor",
        _ => return format!("ImbuedEffectType_{}", key),
    }.to_string()
}

/// UiEffects enum (mask values)
pub fn ui_effects_name(key: u32) -> String {
    match key {
        0x0000 => "Undef", 0x0001 => "Magical", 0x0002 => "Poisoned",
        0x0004 => "BoostHealth", 0x0008 => "BoostMana", 0x0010 => "BoostStamina",
        0x0020 => "Fire", 0x0040 => "Lightning", 0x0080 => "Frost",
        0x0100 => "Acid", 0x0200 => "Bludgeoning", 0x0400 => "Slashing",
        0x0800 => "Piercing", 0x1000 => "Nether",
        _ => return format!("UiEffects_{}", key),
    }.to_string()
}

/// Usable enum (mask values)
pub fn usable_name(key: u32) -> String {
    match key {
        0x00000001 => "No", 0x00000010 => "Self", 0x00000012 => "Wielded",
        0x00000014 => "Contained", 0x00000018 => "Viewed", 0x00000020 => "Remote",
        0x00000030 => "NeverWalk", 0x00000040 => "ObjSelf",
        0x00000800 => "ContainedViewedRemote", 0x00000810 => "ContainedViewedRemoteNeverWalk",
        0x00001000 => "ViewedRemote", 0x00001010 => "ViewedRemoteNeverWalk",
        0x00008000 => "SourceWieldedTargetWielded",
        0x00014000 => "SourceWieldedTargetContained",
        0x00018000 => "SourceWieldedTargetViewed",
        0x00030000 => "SourceWieldedTargetRemote",
        0x00040000 => "SourceWieldedTargetRemoteNeverWalk",
        0x00080000 => "SourceContainedTargetWielded",
        0x000C0000 => "SourceContainedTargetContained",
        0x000C8000 => "SourceContainedTargetSelfOrContained",
        _ => return format!("Usable_{}", key),
    }.to_string()
}

/// BondedStatus enum
pub fn bonded_status_name(key: u32) -> String {
    match key {
        0 => "Normal", 1 => "Bonded", 2 => "Sticky",
        0xFFFFFFFF => "Slippery", 0xFFFFFFFE => "Destroy",
        _ => return format!("BondedStatus_{}", key),
    }.to_string()
}

/// AttunedStatus enum
pub fn attuned_status_name(key: u32) -> String {
    match key {
        0 => "Normal", 1 => "Attuned", 2 => "Sticky",
        _ => return format!("AttunedStatus_{}", key),
    }.to_string()
}

/// HouseType enum
pub fn house_type_name(key: u32) -> String {
    match key {
        1 => "Cottage", 2 => "Villa", 3 => "Mansion", 4 => "Apartment",
        _ => return format!("HouseType_{}", key),
    }.to_string()
}

/// HookType enum (mask values)
pub fn hook_type_name(key: u32) -> String {
    match key {
        0x0001 => "Floor", 0x0002 => "Wall",
        _ => return format!("HookType_{}", key),
    }.to_string()
}

/// PaletteTemplate enum
pub fn palette_template_name(key: u32) -> String {
    match key {
        0 => "Undef", 1 => "AquaBlue", 2 => "Blue", 3 => "BluePurple", 4 => "Brown",
        5 => "DarkBlue", 6 => "DeepBrown", 7 => "DeepGreen", 8 => "Green", 9 => "Grey",
        10 => "LightBlue", 11 => "Maroon", 12 => "Navy", 13 => "Purple", 14 => "Red",
        15 => "RedPurple", 16 => "Rose", 17 => "Yellow", 18 => "YellowBrown",
        19 => "Copper", 20 => "Silver", 21 => "Gold", 22 => "Aqua",
        23..=38 => "MetalVariant", 39 => "Black", 40 => "Bronze",
        41..=88 => "ColorVariant",
        _ => return format!("PaletteTemplate_{}", key),
    }.to_string()
}

/// FactionBits enum (mask values)
pub fn faction_bits_name(key: u32) -> String {
    match key {
        0 => "None", 0x01 => "CelestialHand", 0x02 => "EldrytchWeb", 0x04 => "RadiantBlood",
        _ => return format!("FactionBits_{}", key),
    }.to_string()
}

/// PortalBitmask enum
pub fn portal_bitmask_name(key: u32) -> String {
    match key {
        0x00 => "Unrestricted", 0x01 => "NoPk", 0x02 => "NoPKLite",
        0x04 => "NoNPK", 0x08 => "NoSummon", 0x10 => "NoRecall",
        0x20 => "OnlyOlthoiPCs", 0x40 => "NoOlthoiPCs", 0x80 => "NoVitae",
        0x100 => "NoNewAccounts",
        _ => return format!("PortalBitmask_{}", key),
    }.to_string()
}

/// AetheriaBitfield enum
pub fn aetheria_bitfield_name(key: u32) -> String {
    match key {
        0 => "None", 0x1 => "Blue", 0x2 => "Yellow", 0x4 => "Red",
        _ => return format!("AetheriaBitfield_{}", key),
    }.to_string()
}

/// SummoningMastery enum
pub fn summoning_mastery_name(key: u32) -> String {
    match key {
        0 => "Undef", 1 => "Primalist", 2 => "Necromancer", 3 => "Naturalist",
        _ => return format!("SummoningMastery_{}", key),
    }.to_string()
}
