// Protocol-level modules
pub mod fragment;
pub mod packet;
pub mod reader;

// Re-export commonly used types
pub use fragment::{Fragment, FragmentGroup, FragmentHeader};
pub use packet::{PacketHeader, PacketHeaderFlags};
pub use reader::BinaryReader;
