//! Extract weenie (object) information from parsed messages
//!
//! This module analyzes parsed messages and extracts any object IDs and properties,
//! creating WeenieUpdate structures that can be used to populate the WeenieDatabase.

use crate::messages::ParsedMessage;
use crate::weenie::WeenieUpdate;

/// Extract weenie updates from a parsed message
pub fn extract_weenie_updates(message: &ParsedMessage) -> Vec<WeenieUpdate> {
    let mut updates = Vec::new();

    // Route based on message type (using underscore names as they appear in ParsedMessage)
    match message.message_type.as_str() {
        // ===== Direct S2C Messages (Pattern B) =====
        // Quality updates with ObjectId
        "Qualities_UpdateInt" => {
            if let Some(update) = extract_s2c_direct(message, "QualitiesUpdateInt", extract_int_property) {
                updates.push(update);
            }
        }
        "Qualities_UpdateInstanceId" => {
            if let Some(update) = extract_s2c_direct(message, "QualitiesUpdateInstanceId", extract_instance_id_property) {
                updates.push(update);
            }
        }
        "Qualities_UpdateBool" => {
            if let Some(update) = extract_s2c_direct(message, "QualitiesUpdateBool", extract_bool_property) {
                updates.push(update);
            }
        }
        "Qualities_UpdateFloat" => {
            if let Some(update) = extract_s2c_direct(message, "QualitiesUpdateFloat", extract_float_property) {
                updates.push(update);
            }
        }
        "Qualities_UpdateString" => {
            if let Some(update) = extract_s2c_direct(message, "QualitiesUpdateString", extract_string_property) {
                updates.push(update);
            }
        }
        "Qualities_UpdateInt64" => {
            if let Some(update) = extract_s2c_direct(message, "QualitiesUpdateInt64", extract_int64_property) {
                updates.push(update);
            }
        }
        "Qualities_UpdateDataId" => {
            if let Some(update) = extract_s2c_direct(message, "QualitiesUpdateDataId", extract_data_id_property) {
                updates.push(update);
            }
        }

        // Effects
        "Effects_PlayScriptType" => {
            if let Some(update) = extract_s2c_simple(message, "EffectsPlayScriptType") {
                updates.push(update);
            }
        }
        "Effects_SoundEvent" => {
            if let Some(update) = extract_s2c_simple(message, "EffectsSoundEvent") {
                updates.push(update);
            }
        }

        // Inventory
        "Inventory_PickupEvent" => {
            if let Some(update) = extract_s2c_simple(message, "InventoryPickupEvent") {
                updates.push(update);
            }
        }

        // Movement
        "Movement_SetObjectMovement" => {
            if let Some(update) = extract_s2c_simple(message, "MovementSetObjectMovement") {
                updates.push(update);
            }
        }

        // Item/Object
        "Item_ObjDescEvent" => {
            if let Some(update) = extract_s2c_simple(message, "ItemObjDescEvent") {
                updates.push(update);
            }
        }

        // Messages without ObjectId - skip these
        "Qualities_PrivateUpdateAttribute2ndLevel" |
        "Qualities_PrivateUpdateInt" |
        "Communication_TextboxString" => {
            // These don't have ObjectId, they're player-specific or text-only
        }

        // ===== OrderedGameEvent Messages (Pattern A) =====
        "Item_SetAppraiseInfo" => {
            if let Some(update) = extract_appraise_info(message) {
                updates.push(update);
            }
        }
        "Item_ServerSaysContainId" => {
            if let Some(update) = extract_ordered_event(message, "ItemServerSaysContainId", extract_contain_id_data) {
                updates.push(update);
            }
        }
        "Item_WearItem" => {
            if let Some(update) = extract_ordered_event(message, "ItemWearItem", extract_wear_item_data) {
                updates.push(update);
            }
        }
        "Magic_UpdateEnchantment" => {
            if let Some(update) = extract_ordered_event(message, "MagicUpdateEnchantment", extract_enchantment_data) {
                updates.push(update);
            }
        }
        "Magic_DispelEnchantment" => {
            if let Some(update) = extract_ordered_event_simple(message, "MagicDispelEnchantment") {
                updates.push(update);
            }
        }

        // ===== C2S Messages (OrderedGameAction) =====
        "Item_Appraise" => {
            if let Some(update) = extract_c2s_action(message, "ItemAppraise") {
                updates.push(update);
            }
        }
        "Inventory_PutItemInContainer" => {
            if let Some(update) = extract_c2s_action(message, "InventoryPutItemInContainer") {
                updates.push(update);
            }
        }
        "Inventory_GetAndWieldItem" => {
            if let Some(update) = extract_c2s_action(message, "InventoryGetAndWieldItem") {
                updates.push(update);
            }
        }
        "Character_CharacterOptionsEvent" => {
            // This is a character settings event, no weenie data
        }

        // Unknown message types - handled gracefully
        _ => {
            // Silently skip unknown message types
        }
    }

    updates
}

// ===== Generic Extraction Helpers =====

/// Extract from direct S2C message with property extraction (Pattern B)
fn extract_s2c_direct<F>(
    message: &ParsedMessage,
    msg_type: &str,
    property_extractor: F,
) -> Option<WeenieUpdate>
where
    F: FnOnce(&serde_json::Value, &mut WeenieUpdate),
{
    let s2c = message.data.get("S2C")?;
    let msg_data = s2c.get(msg_type)?;

    let object_id = msg_data.get("ObjectId")?.as_u64()? as u32;
    let mut update = WeenieUpdate::new(object_id, message.timestamp, message.id);

    property_extractor(msg_data, &mut update);

    Some(update)
}

/// Extract from direct S2C message with just ObjectId (Pattern B)
fn extract_s2c_simple(message: &ParsedMessage, msg_type: &str) -> Option<WeenieUpdate> {
    let s2c = message.data.get("S2C")?;
    let msg_data = s2c.get(msg_type)?;

    let object_id = msg_data.get("ObjectId")?.as_u64()? as u32;
    let update = WeenieUpdate::new(object_id, message.timestamp, message.id);

    Some(update)
}

/// Extract from OrderedGameEvent with data extraction (Pattern A)
fn extract_ordered_event<F>(
    message: &ParsedMessage,
    event_type: &str,
    data_extractor: F,
) -> Option<WeenieUpdate>
where
    F: FnOnce(&serde_json::Value, &mut WeenieUpdate) -> Option<()>,
{
    let s2c = message.data.get("S2C")?;
    let ordered_event = s2c.get("OrderedGameEvent")?;
    let event = ordered_event.get("event")?;
    let event_data = event.get(event_type)?;

    let mut update = WeenieUpdate::new(0, message.timestamp, message.id);
    data_extractor(event_data, &mut update)?;

    Some(update)
}

/// Extract from OrderedGameEvent with just object_id at OrderedGameEvent level (Pattern A)
fn extract_ordered_event_simple(message: &ParsedMessage, event_type: &str) -> Option<WeenieUpdate> {
    let s2c = message.data.get("S2C")?;
    let ordered_event = s2c.get("OrderedGameEvent")?;
    let event = ordered_event.get("event")?;
    let _event_data = event.get(event_type)?;

    // object_id is at OrderedGameEvent level, not in the event data
    let object_id = ordered_event.get("object_id")?.as_u64()? as u32;
    let update = WeenieUpdate::new(object_id, message.timestamp, message.id);

    Some(update)
}

/// Extract from C2S OrderedGameAction
fn extract_c2s_action(message: &ParsedMessage, action_type: &str) -> Option<WeenieUpdate> {
    let c2s = message.data.get("C2S")?;
    let ordered_action = c2s.get("OrderedGameAction")?;
    let action = ordered_action.get("action")?;
    let action_data = action.get(action_type)?;

    let object_id = action_data.get("ObjectId")?.as_u64()? as u32;
    let update = WeenieUpdate::new(object_id, message.timestamp, message.id);

    Some(update)
}

// ===== Property Extractors =====

fn extract_int_property(msg_data: &serde_json::Value, update: &mut WeenieUpdate) {
    if let (Some(key), Some(value)) = (
        msg_data.get("Key").and_then(|v| v.as_str()),
        msg_data.get("Value").and_then(|v| v.as_i64()),
    ) {
        update.int_properties.insert(key.to_string(), value as i32);
    }
}

fn extract_instance_id_property(msg_data: &serde_json::Value, update: &mut WeenieUpdate) {
    if let (Some(key), Some(value)) = (
        msg_data.get("Key").and_then(|v| v.as_str()),
        msg_data.get("Value").and_then(|v| v.as_u64()),
    ) {
        update.instance_id_properties.insert(key.to_string(), value as u32);
    }
}

fn extract_bool_property(msg_data: &serde_json::Value, update: &mut WeenieUpdate) {
    if let (Some(key), Some(value)) = (
        msg_data.get("Key").and_then(|v| v.as_str()),
        msg_data.get("Value").and_then(|v| v.as_bool()),
    ) {
        update.bool_properties.insert(key.to_string(), value);
    }
}

fn extract_float_property(msg_data: &serde_json::Value, update: &mut WeenieUpdate) {
    if let (Some(key), Some(value)) = (
        msg_data.get("Key").and_then(|v| v.as_str()),
        msg_data.get("Value").and_then(|v| v.as_f64()),
    ) {
        update.float_properties.insert(key.to_string(), value);
    }
}

fn extract_string_property(msg_data: &serde_json::Value, update: &mut WeenieUpdate) {
    if let (Some(key), Some(value)) = (
        msg_data.get("Key").and_then(|v| v.as_str()),
        msg_data.get("Value").and_then(|v| v.as_str()),
    ) {
        update.string_properties.insert(key.to_string(), value.to_string());
    }
}

fn extract_int64_property(msg_data: &serde_json::Value, update: &mut WeenieUpdate) {
    if let (Some(key), Some(value)) = (
        msg_data.get("Key").and_then(|v| v.as_str()),
        msg_data.get("Value").and_then(|v| v.as_i64()),
    ) {
        update.int64_properties.insert(key.to_string(), value);
    }
}

fn extract_data_id_property(msg_data: &serde_json::Value, update: &mut WeenieUpdate) {
    if let (Some(key), Some(value)) = (
        msg_data.get("Key").and_then(|v| v.as_str()),
        msg_data.get("Value").and_then(|v| v.as_u64()),
    ) {
        update.data_id_properties.insert(key.to_string(), value as u32);
    }
}

// ===== Specialized Event Data Extractors =====

fn extract_appraise_info(message: &ParsedMessage) -> Option<WeenieUpdate> {
    // For Item_SetAppraiseInfo, the data is nested in S2C.OrderedGameEvent.event.ItemSetAppraiseInfo
    let s2c = message.data.get("S2C")?;
    let ordered_event = s2c.get("OrderedGameEvent")?;
    let event = ordered_event.get("event")?;
    let appraise_data = event.get("ItemSetAppraiseInfo")?;

    // object_id is in the ItemSetAppraiseInfo itself
    let object_id = appraise_data.get("ObjectId")?.as_u64()? as u32;

    let mut update = WeenieUpdate::new(object_id, message.timestamp, message.id);

    // Extract properties from all property dictionaries in appraise_info
    if let Some(int_props_obj) = appraise_data.get("IntProperties").and_then(|v| v.as_object()) {
        if let Some(table) = int_props_obj.get("Table").and_then(|v| v.as_object()) {
            for (key, value) in table {
                if let Some(v) = value.as_i64() {
                    update.int_properties.insert(key.clone(), v as i32);
                }
            }
        }
    }

    if let Some(int64_props_obj) = appraise_data.get("Int64Properties").and_then(|v| v.as_object()) {
        if let Some(table) = int64_props_obj.get("Table").and_then(|v| v.as_object()) {
            for (key, value) in table {
                if let Some(v) = value.as_i64() {
                    update.int64_properties.insert(key.clone(), v);
                }
            }
        }
    }

    if let Some(bool_props_obj) = appraise_data.get("BoolProperties").and_then(|v| v.as_object()) {
        if let Some(table) = bool_props_obj.get("Table").and_then(|v| v.as_object()) {
            for (key, value) in table {
                if let Some(v) = value.as_bool() {
                    update.bool_properties.insert(key.clone(), v);
                }
            }
        }
    }

    if let Some(float_props_obj) = appraise_data.get("FloatProperties").and_then(|v| v.as_object()) {
        if let Some(table) = float_props_obj.get("Table").and_then(|v| v.as_object()) {
            for (key, value) in table {
                if let Some(v) = value.as_f64() {
                    update.float_properties.insert(key.clone(), v);
                }
            }
        }
    }

    if let Some(string_props_obj) = appraise_data.get("StringProperties").and_then(|v| v.as_object()) {
        if let Some(table) = string_props_obj.get("Table").and_then(|v| v.as_object()) {
            for (key, value) in table {
                if let Some(v) = value.as_str() {
                    update.string_properties.insert(key.clone(), v.to_string());
                }
            }
        }
    }

    if let Some(did_props_obj) = appraise_data.get("DataIdProperties").and_then(|v| v.as_object()) {
        if let Some(table) = did_props_obj.get("Table").and_then(|v| v.as_object()) {
            for (key, value) in table {
                if let Some(v) = value.as_u64() {
                    update.data_id_properties.insert(key.clone(), v as u32);
                }
            }
        }
    }

    // Extract name if present (try Name first, then LongDesc)
    if let Some(name) = update.string_properties.get("Name").cloned() {
        update.name = Some(name);
    } else if let Some(name) = update.string_properties.get("LongDesc").cloned() {
        update.name = Some(name);
    }

    Some(update)
}

fn extract_contain_id_data(event_data: &serde_json::Value, update: &mut WeenieUpdate) -> Option<()> {
    let object_id = event_data.get("ObjectId")?.as_u64()? as u32;
    let container_id = event_data.get("ContainerId")?.as_u64()? as u32;

    update.object_id = object_id;
    update.instance_id_properties.insert("Container".to_string(), container_id);
    Some(())
}

fn extract_wear_item_data(event_data: &serde_json::Value, update: &mut WeenieUpdate) -> Option<()> {
    let object_id = event_data.get("ObjectId")?.as_u64()? as u32;
    update.object_id = object_id;
    Some(())
}

fn extract_enchantment_data(event_data: &serde_json::Value, update: &mut WeenieUpdate) -> Option<()> {
    let enchantment = event_data.get("Enchantment")?;
    let caster_id = enchantment.get("CasterId")?.as_u64()? as u32;
    update.object_id = caster_id;
    Some(())
}
