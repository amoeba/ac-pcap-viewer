//! Shared types and utilities for AC PCAP parser crates

use serde::{Serialize, Serializer};

/// Direction of packet flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Send, // Client to Server
    Recv, // Server to Client
}

impl Serialize for Direction {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Direction::Send => serializer.serialize_str("Send"),
            Direction::Recv => serializer.serialize_str("Recv"),
        }
    }
}

/// UI tab selection
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Tab {
    #[default]
    Messages,
    Fragments,
}

/// UI view mode
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    #[default]
    Tree,
    Binary,
}

/// Sort field options
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum SortField {
    #[default]
    Id,
    Type,
    Direction,
}
