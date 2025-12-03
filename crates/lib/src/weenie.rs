//! Weenie (game object) aggregation and tracking
//!
//! This module provides data structures and logic for aggregating weenie (game object)
//! information from PCAP messages. Since messages can contain partial or repeated
//! information about objects, we maintain a database that merges all information
//! seen about each object throughout the PCAP.

use serde::{Serialize, Serializer};
use std::collections::HashMap;

/// A weenie (game object) with all its accumulated properties
#[derive(Debug, Clone, Serialize)]
pub struct Weenie {
    /// The unique object ID
    #[serde(rename = "ObjectId")]
    pub object_id: u32,

    /// Name of the object (if known)
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Integer properties (e.g., ItemType, StackSize, Value)
    #[serde(rename = "IntProperties", skip_serializing_if = "HashMap::is_empty")]
    pub int_properties: HashMap<String, i32>,

    /// Int64 properties (e.g., Experience values)
    #[serde(rename = "Int64Properties", skip_serializing_if = "HashMap::is_empty")]
    pub int64_properties: HashMap<String, i64>,

    /// Boolean properties (e.g., Open, Locked, Attackable)
    #[serde(rename = "BoolProperties", skip_serializing_if = "HashMap::is_empty")]
    pub bool_properties: HashMap<String, bool>,

    /// Float properties (e.g., Scale, UseRadius)
    #[serde(rename = "FloatProperties", skip_serializing_if = "HashMap::is_empty")]
    pub float_properties: HashMap<String, f64>,

    /// String properties (e.g., Description, Inscription)
    #[serde(rename = "StringProperties", skip_serializing_if = "HashMap::is_empty")]
    pub string_properties: HashMap<String, String>,

    /// DataId properties (e.g., Icon, Sound IDs)
    #[serde(rename = "DataIdProperties", skip_serializing_if = "HashMap::is_empty")]
    pub data_id_properties: HashMap<String, u32>,

    /// Instance ID properties (references to other objects)
    #[serde(
        rename = "InstanceIdProperties",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub instance_id_properties: HashMap<String, u32>,

    /// Metadata: first seen timestamp
    #[serde(rename = "FirstSeen")]
    pub first_seen: f64,

    /// Metadata: last updated timestamp
    #[serde(rename = "LastUpdated")]
    pub last_updated: f64,

    /// Metadata: number of messages that referenced this object
    #[serde(rename = "MessageCount")]
    pub message_count: usize,
}

impl Weenie {
    /// Create a new weenie with the given object ID and timestamp
    pub fn new(object_id: u32, timestamp: f64) -> Self {
        Self {
            object_id,
            name: None,
            int_properties: HashMap::new(),
            int64_properties: HashMap::new(),
            bool_properties: HashMap::new(),
            float_properties: HashMap::new(),
            string_properties: HashMap::new(),
            data_id_properties: HashMap::new(),
            instance_id_properties: HashMap::new(),
            first_seen: timestamp,
            last_updated: timestamp,
            message_count: 1,
        }
    }

    /// Update the weenie with new data from a message
    pub fn update(&mut self, update: WeenieUpdate) {
        self.last_updated = update.timestamp;
        self.message_count += 1;

        if let Some(name) = update.name {
            self.name = Some(name);
        }

        for (key, value) in update.int_properties {
            self.int_properties.insert(key, value);
        }

        for (key, value) in update.int64_properties {
            self.int64_properties.insert(key, value);
        }

        for (key, value) in update.bool_properties {
            self.bool_properties.insert(key, value);
        }

        for (key, value) in update.float_properties {
            self.float_properties.insert(key, value);
        }

        for (key, value) in update.string_properties {
            self.string_properties.insert(key, value);
        }

        for (key, value) in update.data_id_properties {
            self.data_id_properties.insert(key, value);
        }

        for (key, value) in update.instance_id_properties {
            self.instance_id_properties.insert(key, value);
        }
    }
}

/// An update to a weenie from a message
#[derive(Debug, Clone, Default)]
pub struct WeenieUpdate {
    pub object_id: u32,
    pub timestamp: f64,
    pub name: Option<String>,
    pub int_properties: HashMap<String, i32>,
    pub int64_properties: HashMap<String, i64>,
    pub bool_properties: HashMap<String, bool>,
    pub float_properties: HashMap<String, f64>,
    pub string_properties: HashMap<String, String>,
    pub data_id_properties: HashMap<String, u32>,
    pub instance_id_properties: HashMap<String, u32>,
}

impl WeenieUpdate {
    /// Create a new update for the given object ID
    pub fn new(object_id: u32, timestamp: f64) -> Self {
        Self {
            object_id,
            timestamp,
            ..Default::default()
        }
    }
}

/// Database of all weenies seen in a PCAP
#[derive(Debug, Clone, Default)]
pub struct WeenieDatabase {
    weenies: HashMap<u32, Weenie>,
}

impl WeenieDatabase {
    /// Create a new empty weenie database
    pub fn new() -> Self {
        Self {
            weenies: HashMap::new(),
        }
    }

    /// Add or update a weenie with the given update
    pub fn add_or_update(&mut self, update: WeenieUpdate) {
        let object_id = update.object_id;
        let timestamp = update.timestamp;

        self.weenies
            .entry(object_id)
            .and_modify(|w| w.update(update.clone()))
            .or_insert_with(|| {
                let mut weenie = Weenie::new(object_id, timestamp);
                weenie.update(update);
                weenie
            });
    }

    /// Get a weenie by object ID
    pub fn get(&self, object_id: u32) -> Option<&Weenie> {
        self.weenies.get(&object_id)
    }

    /// Get all weenies
    pub fn weenies(&self) -> &HashMap<u32, Weenie> {
        &self.weenies
    }

    /// Get a sorted list of weenies by object ID
    pub fn sorted_weenies(&self) -> Vec<&Weenie> {
        let mut weenies: Vec<&Weenie> = self.weenies.values().collect();
        weenies.sort_by_key(|w| w.object_id);
        weenies
    }

    /// Get the number of weenies tracked
    pub fn count(&self) -> usize {
        self.weenies.len()
    }
}

impl Serialize for WeenieDatabase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.weenies.serialize(serializer)
    }
}
