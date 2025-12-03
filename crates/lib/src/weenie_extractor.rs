//! Extract weenie (object) information from parsed messages
//!
//! This module analyzes parsed messages and extracts any object IDs and properties,
//! creating WeenieUpdate structures that can be used to populate the WeenieDatabase.

use crate::messages::ParsedMessage;
use crate::weenie::WeenieUpdate;
use std::collections::HashMap;

/// Extract weenie updates from a parsed message
pub fn extract_weenie_updates(message: &ParsedMessage) -> Vec<WeenieUpdate> {
    let mut updates = Vec::new();

    // Check the message type and extract object data accordingly
    match message.message_type.as_str() {
        // Quality updates with ObjectId
        "Qualities_UpdateInt" => {
            if let Some(update) = extract_qualities_update_int(message) {
                updates.push(update);
            }
        }
        "Qualities_UpdateInstanceId" => {
            if let Some(update) = extract_qualities_update_instance_id(message) {
                updates.push(update);
            }
        }
        "Qualities_UpdateBool" => {
            if let Some(update) = extract_qualities_update_bool(message) {
                updates.push(update);
            }
        }
        "Qualities_UpdateFloat" => {
            if let Some(update) = extract_qualities_update_float(message) {
                updates.push(update);
            }
        }
        "Qualities_UpdateString" => {
            if let Some(update) = extract_qualities_update_string(message) {
                updates.push(update);
            }
        }
        "Qualities_UpdateInt64" => {
            if let Some(update) = extract_qualities_update_int64(message) {
                updates.push(update);
            }
        }
        "Qualities_UpdateDataId" => {
            if let Some(update) = extract_qualities_update_data_id(message) {
                updates.push(update);
            }
        }

        // Game events with object data
        "Ordered_GameEvent" => {
            if let Some(event_type) = message.data.get("Type").and_then(|v| v.as_str()) {
                match event_type {
                    "Item_SetAppraiseInfo" => {
                        if let Some(update) = extract_appraise_info(message) {
                            updates.push(update);
                        }
                    }
                    "Item_ServerSaysContainId" => {
                        if let Some(update) = extract_contain_id(message) {
                            updates.push(update);
                        }
                    }
                    "Item_WearItem" => {
                        if let Some(update) = extract_wear_item(message) {
                            updates.push(update);
                        }
                    }
                    "Magic_UpdateEnchantment" => {
                        if let Some(update) = extract_enchantment(message) {
                            updates.push(update);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Movement events
        "Movement_SetObjectMovement" => {
            if let Some(update) = extract_movement_object(message) {
                updates.push(update);
            }
        }

        // Inventory events
        "Inventory_PickupEvent" => {
            if let Some(update) = extract_pickup_event(message) {
                updates.push(update);
            }
        }

        // Object description
        "Item_ObjDescEvent" => {
            if let Some(update) = extract_obj_desc_event(message) {
                updates.push(update);
            }
        }

        // Communication events may reference objects
        "Communication_HearSpeech" => {
            if let Some(update) = extract_hear_speech(message) {
                updates.push(update);
            }
        }

        _ => {}
    }

    updates
}

fn extract_qualities_update_int(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let object_id = message.data.get("ObjectId")?.as_u64()? as u32;
    let key = message.data.get("Key")?.as_str()?;
    let value = message.data.get("Value")?.as_i64()? as i32;

    let mut update = WeenieUpdate::new(object_id, message.timestamp);
    update.int_properties.insert(key.to_string(), value);
    Some(update)
}

fn extract_qualities_update_instance_id(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let object_id = message.data.get("ObjectId")?.as_u64()? as u32;
    let key = message.data.get("Key")?.as_str()?;
    let value = message.data.get("Value")?.as_u64()? as u32;

    let mut update = WeenieUpdate::new(object_id, message.timestamp);
    update.instance_id_properties.insert(key.to_string(), value);
    Some(update)
}

fn extract_qualities_update_bool(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let object_id = message.data.get("ObjectId")?.as_u64()? as u32;
    let key = message.data.get("Key")?.as_str()?;
    let value = message.data.get("Value")?.as_bool()?;

    let mut update = WeenieUpdate::new(object_id, message.timestamp);
    update.bool_properties.insert(key.to_string(), value);
    Some(update)
}

fn extract_qualities_update_float(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let object_id = message.data.get("ObjectId")?.as_u64()? as u32;
    let key = message.data.get("Key")?.as_str()?;
    let value = message.data.get("Value")?.as_f64()?;

    let mut update = WeenieUpdate::new(object_id, message.timestamp);
    update.float_properties.insert(key.to_string(), value);
    Some(update)
}

fn extract_qualities_update_string(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let object_id = message.data.get("ObjectId")?.as_u64()? as u32;
    let key = message.data.get("Key")?.as_str()?;
    let value = message.data.get("Value")?.as_str()?;

    let mut update = WeenieUpdate::new(object_id, message.timestamp);
    update
        .string_properties
        .insert(key.to_string(), value.to_string());
    Some(update)
}

fn extract_qualities_update_int64(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let object_id = message.data.get("ObjectId")?.as_u64()? as u32;
    let key = message.data.get("Key")?.as_str()?;
    let value = message.data.get("Value")?.as_i64()?;

    let mut update = WeenieUpdate::new(object_id, message.timestamp);
    update.int64_properties.insert(key.to_string(), value);
    Some(update)
}

fn extract_qualities_update_data_id(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let object_id = message.data.get("ObjectId")?.as_u64()? as u32;
    let key = message.data.get("Key")?.as_str()?;
    let value = message.data.get("Value")?.as_u64()? as u32;

    let mut update = WeenieUpdate::new(object_id, message.timestamp);
    update.data_id_properties.insert(key.to_string(), value);
    Some(update)
}

fn extract_appraise_info(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let data = message.data.get("Data")?;
    let object_id = data.get("ObjectId")?.as_u64()? as u32;

    let mut update = WeenieUpdate::new(object_id, message.timestamp);

    // Extract properties from all property dictionaries
    if let Some(int_props) = data.get("IntProperties").and_then(|v| v.as_object()) {
        for (key, value) in int_props {
            if let Some(v) = value.as_i64() {
                update.int_properties.insert(key.clone(), v as i32);
            }
        }
    }

    if let Some(int64_props) = data.get("Int64Properties").and_then(|v| v.as_object()) {
        for (key, value) in int64_props {
            if let Some(v) = value.as_i64() {
                update.int64_properties.insert(key.clone(), v);
            }
        }
    }

    if let Some(bool_props) = data.get("BoolProperties").and_then(|v| v.as_object()) {
        for (key, value) in bool_props {
            if let Some(v) = value.as_bool() {
                update.bool_properties.insert(key.clone(), v);
            }
        }
    }

    if let Some(float_props) = data.get("FloatProperties").and_then(|v| v.as_object()) {
        for (key, value) in float_props {
            if let Some(v) = value.as_f64() {
                update.float_properties.insert(key.clone(), v);
            }
        }
    }

    if let Some(string_props) = data.get("StringProperties").and_then(|v| v.as_object()) {
        for (key, value) in string_props {
            if let Some(v) = value.as_str() {
                update.string_properties.insert(key.clone(), v.to_string());
            }
        }
    }

    if let Some(did_props) = data.get("DataIdProperties").and_then(|v| v.as_object()) {
        for (key, value) in did_props {
            if let Some(v) = value.as_u64() {
                update.data_id_properties.insert(key.clone(), v as u32);
            }
        }
    }

    // Extract name if present
    if let Some(name) = string_props_get(&update.string_properties, "Name") {
        update.name = Some(name);
    }

    Some(update)
}

fn extract_contain_id(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let data = message.data.get("Data")?;
    let object_id = data.get("ObjectId")?.as_u64()? as u32;
    let container_id = data.get("ContainerId")?.as_u64()? as u32;

    let mut update = WeenieUpdate::new(object_id, message.timestamp);
    update
        .instance_id_properties
        .insert("Container".to_string(), container_id);
    Some(update)
}

fn extract_wear_item(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let data = message.data.get("Data")?;
    let object_id = data.get("ObjectId")?.as_u64()? as u32;

    let update = WeenieUpdate::new(object_id, message.timestamp);
    Some(update)
}

fn extract_enchantment(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let data = message.data.get("Data")?;
    let caster_id = data.get("CasterId")?.as_u64()? as u32;

    let update = WeenieUpdate::new(caster_id, message.timestamp);
    Some(update)
}

fn extract_movement_object(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let object_id = message.data.get("ObjectId")?.as_u64()? as u32;
    let update = WeenieUpdate::new(object_id, message.timestamp);
    Some(update)
}

fn extract_pickup_event(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let object_id = message.data.get("ObjectId")?.as_u64()? as u32;
    let update = WeenieUpdate::new(object_id, message.timestamp);
    Some(update)
}

fn extract_obj_desc_event(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let object_id = message.data.get("ObjectId")?.as_u64()? as u32;
    let update = WeenieUpdate::new(object_id, message.timestamp);
    Some(update)
}

fn extract_hear_speech(message: &ParsedMessage) -> Option<WeenieUpdate> {
    let sender_id = message.data.get("SenderId")?.as_u64()? as u32;
    let update = WeenieUpdate::new(sender_id, message.timestamp);
    Some(update)
}

// Helper to get string from string properties
fn string_props_get(props: &HashMap<String, String>, key: &str) -> Option<String> {
    props.get(key).cloned()
}
