# AC PCAP Parser - Codebase Documentation

## Project Overview

**ac-pcap-parser** is a Rust application that parses PCAP (packet capture) files containing Asheron's Call (AC) game network traffic. It extracts, reassembles fragmented UDP packets, and decodes binary messages from the AC network protocol into JSON format.

The parser handles:
- PCAP file reading and UDP packet extraction
- Packet fragmentation and reassembly
- AC protocol packet structure parsing
- Game message decoding (Server-to-Client and Client-to-Server)
- JSON serialization of parsed data

**Language:** Rust (2021 edition)
**Version:** 0.1.0
**Total Lines of Code:** ~2,587 lines

---

## Project Goals & Current Status

### Primary Objectives

We are building a parser that matches the output of a reference implementation. The goal is to parse `pkt_2025-11-18_1763490291_log.pcap` and produce output matching the reference files.

### Reference Files (Target Output)

| File | Lines | Description |
|------|-------|-------------|
| `messages.json` | 2,087 | Parsed messages in JSONL format - **primary target** |
| `fragments.json` | 631 | Packet-level output with headers, fragments, and messages |

### Current Parser Output

| Metric | Reference | Our Output | Status |
|--------|-----------|------------|--------|
| Message count | 2,087 | 2,047 | **CLOSE** (96% match) |

### Goal #1: Match Message Count - COMPLETED

Message count now closely matches! The fix was handling `PcapError::Incomplete` correctly in the PCAP reading loop.

### Goal #2: Match Message Data Quality - IN PROGRESS

**COMPLETED:**

1. **`Item_SetAppraiseInfo`** - ✅ Full property dictionaries
   - Now has: `IntProperties`, `Int64Properties`, `BoolProperties`, `FloatProperties`, `StringProperties`, `DataIdProperties`, `SpellBook`, `ArmorProfile`
   - Property names now match reference (e.g., "Dyable", "ImbuerName", "GearDamage")

2. **`Movement_SetObjectMovement`** - ✅ Basic MovementData fixed
   - Now has: `MovementType: "InterpertedMotionState"` (was "Invalid")
   - Added: `OptionFlags`, `Stance` fields

3. **Property name mappings** - ✅ Implemented in `properties.rs`
   - Added: `property_int_name`, `property_int64_name`, `property_bool_name`, `property_float_name`, `property_string_name`, `property_dataid_name`
   - Values aligned with ACProtocol/protocol.xml definitions

**REMAINING:**

1. **`Movement_SetObjectMovement`** - Need full `State` parsing
   - Reference has: `State` with `Flags`, `CurrentStyle`, `ForwardCommand`, `Commands`
   - We have: `State` is `null` (parsing attempted but may fail)

2. **`Item_ObjDescEvent`** - Missing ObjectDescription
   - Reference has: Full `ObjectDescription` with `Palette`, `Subpalettes`, `TMChanges`, `APChanges`
   - We have: Only basic `ObjectId` and sequences

3. **`Magic_UpdateEnchantment`** - Missing enchantment details
   - Reference has: Full `Enchantment` with `Id`, `SpellCategory`, `Duration`, `CasterId`, `StatMod`, `EquipmentSet`
   - We have: Only basic ordered event fields

4. **`Effects_SoundEvent`** - Need enum names
   - Reference has: `"SoundType": "UnwieldObject"`
   - We have: `"SoundType": 39` (raw number)

### Priority Work Items

1. ~~**Fix multiple messages per packet** - DONE~~
2. ~~**Implement full `Item_SetAppraiseInfo` parsing** - DONE~~
3. ~~**Fix `MovementData` parsing** - DONE (basic, needs State improvement)~~
4. ~~**Implement property name mappings** - DONE~~
5. **Implement `ObjectDescription` parsing** - For `Item_ObjDescEvent`
6. **Implement full enchantment parsing** - For `Magic_UpdateEnchantment`
7. **Add enum name mappings** - Convert numeric values to string names (SoundType, etc.)

### How to Compare Output

```bash
# Generate our output
cargo run 2>/dev/null > our_output.jsonl

# Count messages
wc -l our_output.jsonl messages.json

# Compare first few messages (use jq for formatting)
head -5 our_output.jsonl | python3 -m json.tool
head -5 messages.json | python3 -m json.tool
```

---

## Project Structure

```
ac-pcap-parser/
├── Cargo.toml                 # Rust project manifest and dependencies
├── Cargo.lock                 # Locked dependency versions
├── .gitignore                 # Git ignore rules (ignores /target)
├── CLAUDE.md                  # This documentation
├── src/                       # Source code directory
│   ├── main.rs               # Entry point and main packet parser logic (~340 lines)
│   ├── packet.rs             # Packet header parsing and flag definitions (341 lines)
│   ├── fragment.rs           # Fragment reassembly logic (123 lines)
│   ├── message.rs            # Message wrapper structure (28 lines)
│   ├── reader.rs             # Binary reader utility (~170 lines)
│   ├── enums.rs              # Message opcode enumerations (227 lines)
│   ├── properties.rs         # AC property enums and parsing (613 lines) - NEW
│   └── messages/             # Message parsing module
│       ├── mod.rs            # Message routing and dispatcher (181 lines)
│       ├── s2c.rs            # Server-to-Client message parsing (1,049 lines)
│       └── c2s.rs            # Client-to-Server message parsing (171 lines)
├── pkt_2025-11-18_1763490291_log.pcap  # Sample PCAP file for testing
├── expected.json              # Expected parser output (test data)
├── our_output.json           # Our parser output (test data)
├── fragments.json            # Raw fragments output (large test file, 1.5MB)
├── messages.json             # Parsed messages output (927KB)
├── exp.jsonl                 # Expected JSONL format (test data)
├── our.jsonl                 # Our JSONL output (test data)
└── result.js                 # JavaScript output (large, 2.7MB)
```

---

## Build System & Configuration

### Cargo.toml

**Package Metadata:**
- Name: `ac-pcap-parser`
- Version: `0.1.0`
- Edition: `2021` (Rust 2021 edition)

**Dependencies:**

| Crate | Version | Purpose |
|-------|---------|---------|
| `pcap-parser` | 0.15 | PCAP file format parsing and block reading |
| `anyhow` | 1.0 | Error handling with context (Result/Error types) |
| `thiserror` | 2.0 | Error derive macros (currently not heavily used) |
| `bitflags` | 2.4 | Bitfield flag definitions for packet headers |
| `etherparse` | 0.16 | Ethernet/IP/UDP header parsing (imported but minimal use) |
| `serde` | 1.0 | Serialization framework (with `derive` feature) |
| `serde_json` | 1.0 | JSON serialization/deserialization |
| `hex` | 0.4 | Hexadecimal encoding utilities |
| `base64` | 0.22 | Base64 encoding for fragment data |

**No dev-dependencies or test configuration currently defined.**

---

## Source Files & Their Roles

### 1. `src/main.rs` (337 lines)

**Primary Entry Point & Core Logic**

Main responsibilities:
- PCAP file reading and parsing
- UDP packet extraction (skips 42-byte Ethernet/IP/UDP header)
- Direction determination based on port (AC server ports: 9000-9013)
- Fragment management and reassembly coordination
- Message parsing orchestration
- JSON output generation

**Key Structures:**
- `Direction` enum: `Send` (Client→Server), `Recv` (Server→Client)
- `FragmentInfo`: Fragment metadata with base64-encoded data
- `ParsedPacket`: Complete packet data with header, direction, and messages
- `PacketParser`: Main state machine managing fragment reassembly

**Main Functions:**
- `parse_pcap_file()`: Opens and processes PCAP file, returns packets and messages
- `parse_packet()`: Processes individual UDP payload
- `parse_fragment()`: Handles fragment parsing and reassembly logic
- `main()`: Reads PCAP, outputs messages as JSONL to stdout

**Important Details:**
- Uses `LegacyPcapReader` from pcap-parser crate
- Direction detection: source port >= 9000 && <= 9013 = Server, else Client
- Maintains HashMap of pending fragments by sequence number
- Implements fragment completion detection and automatic cleanup

---

### 2. `src/packet.rs` (341 lines)

**Packet Header Parsing**

Defines the AC network protocol packet header structure and parsing.

**Key Structures:**

**PacketHeaderFlags** (bitflags - 21 flags):
```rust
const RETRANSMISSION = 0x00000001;      // Retransmitted packet
const ENCRYPTED_CHECKSUM = 0x00000002;  // Checksum encrypted
const BLOB_FRAGMENTS = 0x00000004;      // Contains fragmented data
const SERVER_SWITCH = 0x00000100;       // Server switching
const LOGON_SERVER_ADDR = 0x00000200;   // Logon server address
const EMPTY_HEADER1 = 0x00000400;       // Empty header slot
const REFERRAL = 0x00000800;            // Referral data
const REQUEST_RETRANSMIT = 0x00001000;  // Request retransmission
const REJECT_RETRANSMIT = 0x00002000;   // Reject retransmission
const ACK_SEQUENCE = 0x00004000;        // Acknowledgment for sequence
const DISCONNECT = 0x00008000;          // Disconnection signal
const LOGIN_REQUEST = 0x00010000;       // Login request
const WORLD_LOGIN_REQUEST = 0x00020000; // World login request
const CONNECT_REQUEST = 0x00040000;     // Connection request
const CONNECT_RESPONSE = 0x00080000;    // Connection response
const NET_ERROR = 0x00100000;           // Network error
const NET_ERROR_DISCONNECT = 0x00200000;// Network error with disconnect
const CICMD_COMMAND = 0x00400000;       // CI command
const TIME_SYNC = 0x01000000;           // Time synchronization
const ECHO_REQUEST = 0x02000000;        // Ping request
const ECHO_RESPONSE = 0x04000000;       // Ping response
const FLOW = 0x08000000;                // Flow control
```

**PacketHeader** (main structure):
```rust
pub struct PacketHeader {
    pub sequence: u32,
    pub flags: PacketHeaderFlags,
    pub checksum: u32,
    pub id: u16,
    pub time: u16,
    pub size: u16,
    pub iteration: u16,
    // Optional headers based on flags...
}
```

Base size: 20 bytes (5x u32 or u16 combinations)

**Implemented Optional Headers:**
- `AckSequenceHeader`: Sequence acknowledgment (u32)
- `TimeSyncHeader`: Time value (u64)
- `EchoRequestHeader`: Local time (f32)
- `EchoResponseHeader`: Local time + holding time (f32 × 2)
- `FlowHeader`: Data received (u32) + interval (u16)
- `RequestRetransmitHeader`: List of sequence IDs to retransmit

**Not Yet Implemented Headers** (bail with error):
- `ServerSwitchHeader`
- `LogonServerAddrHeader`
- `ReferralHeader`
- `LoginRequestHeader`
- `WorldLoginRequestHeader`
- `ConnectRequestHeader`
- `ConnectResponseHeader`
- `NetErrorHeader`
- `NetErrorDisconnectHeader`
- `CICMDCommandHeader`

---

### 3. `src/fragment.rs` (123 lines)

**Fragment Management & Reassembly**

Handles splitting of large messages across multiple UDP packets.

**Key Structures:**

**FragmentGroup** (enum):
- `Event` (5)
- `Private` (9)
- `Object` (10)

**FragmentHeader** (16-byte structure):
```rust
pub struct FragmentHeader {
    pub sequence: u32,   // Fragment sequence ID
    pub id: u32,         // Fragment ID
    pub count: u16,      // Total fragments
    pub size: u16,       // Fragment size
    pub index: u16,      // This fragment's index
    pub group: Option<FragmentGroup>,
}
```

Constants:
- `CHUNK_SIZE`: 448 bytes (max payload per fragment)
- `SIZE`: 16 bytes (header size)

**Fragment** (reassembly buffer):
```rust
pub struct Fragment {
    pub header: FragmentHeader,
    pub sequence: u32,
    pub data: Vec<u8>,      // Reassembled data
    pub count: usize,       // Total expected fragments
    pub received: usize,    // Fragments received so far
    pub length: usize,      // Total bytes received
    chunks: Vec<bool>,      // Tracking which chunks arrived
}
```

**Key Methods:**
- `new()`: Create new fragment buffer for given count
- `add_chunk()`: Add fragment piece, track received count
- `is_complete()`: Check if all fragments received
- Custom hex serialization for raw data

---

### 4. `src/reader.rs` (130 lines)

**Binary Reader Utility**

Provides convenient methods for reading little-endian binary data from AC protocol messages.

**BinaryReader** Structure:
- Wraps `Cursor<&[u8]>` for position tracking
- All reads are little-endian (matching AC protocol)

**Reading Methods:**
- `read_u8()`, `read_u16()`, `read_u32()`, `read_u64()`
- `read_i32()`, `read_i64()`
- `read_f32()`, `read_f64()`
- `read_bool()`: Reads u32 and checks != 0
- `read_bool_byte()`: Reads u8 and checks != 0
- `read_bytes(len)`: Read variable-length byte buffer
- `read_string16l()`: Read length-prefixed string with 4-byte alignment
- `read_compressed_uint()`: Variable-length compression (1-4 bytes)

**Position Tracking:**
- `position()`: Get current cursor position
- `set_position(pos)`: Jump to position
- `remaining()`: Bytes left to read

**Important Note:** All multi-byte reads use little-endian byte order (`.from_le_bytes()`).

---

### 5. `src/enums.rs` (227 lines)

**Message Type Definitions**

Defines opcode enumerations for both S2C and C2S message types.

**S2CMessageType** (56 variants):
Server-to-Client message opcodes including:
- Item operations: `ItemServerSaysRemove` (0x0024), `ItemUpdateStackSize` (0x0197), `ItemObjDescEvent` (0xF625), etc.
- Qualities updates: `QualitiesUpdateInt` (0x02CE), `QualitiesUpdateBool` (0x02D2), etc.
- Movement: `MovementPositionEvent` (0xF748), `MovementSetObjectMovement` (0xF74C)
- Communication: `CommunicationHearSpeech` (0x02BB), `CommunicationTextboxString` (0xF7E0)
- Combat: `CombatHandlePlayerDeathEvent` (0x019E)
- Effects: `EffectsSoundEvent` (0xF750), `EffectsPlayerTeleport` (0xF751)
- Login: `LoginCreatePlayer` (0xF746), `LoginWorldInfo` (0xF7E1)
- DDD (Data Distribution): `DDDDataMessage`, `DDDBeginDDDMessage`, etc.
- `OrderedGameEvent` (0xF7B0): Container for game events

**C2SMessageType** (3 variants):
Client-to-Server message opcodes:
- `LoginSendEnterWorld` (0xF657)
- `OrderedGameAction` (0xF7B1)
- `Unknown` (0xFFFFFFFF)

**Implementation:**
- Each type has `from_u32()` method for opcode mapping
- Used to dispatch messages to appropriate handlers

---

### 6. `src/message.rs` (28 lines)

**Message Wrapper**

Simple wrapper for raw message data.

**Message** Structure:
```rust
pub struct Message {
    pub data: Vec<u8>,  // Raw message bytes
}
```

Features:
- `parse()`: Creates Message from bytes
- Custom hex serialization

This is a placeholder; actual message parsing happens in `messages/` module.

---

### 7. `src/messages/mod.rs` (181 lines)

**Message Dispatcher & Router**

Routes messages to correct handlers based on opcode and direction.

**ParsedMessage** Structure:
```rust
pub struct ParsedMessage {
    pub id: usize,
    pub message_type: String,
    pub data: serde_json::Value,  // Actual parsed message content
    pub direction: String,         // "Send" or "Recv"
    pub opcode: String,            // Hex format: "F7B0"
}
```

**Key Functions:**

`parse_message(data: &[u8], id: usize) -> Result<ParsedMessage>`:
1. Reads opcode (first u32)
2. Checks if S2C or C2S type
3. Routes to appropriate handler
4. Returns parsed message or unknown handler

**Handlers:**
- `parse_s2c_message()`: Routes S2C opcodes (0xF7B0 game events, various update messages)
- `parse_c2s_message()`: Routes C2S opcodes (0xF7B1 game actions)

**Implemented S2C Message Handlers (10):**
- `OrderedGameEvent` (0xF7B0)
- `QualitiesPrivateUpdateInt` (0x02CD)
- `QualitiesPrivateUpdateAttribute2ndLevel` (0x02E9)
- `QualitiesUpdateInt` (0x02CE)
- `QualitiesUpdateInstanceId` (0x02DA)
- `MovementSetObjectMovement` (0xF74C)
- `InventoryPickupEvent` (0xF74A)
- `EffectsSoundEvent` (0xF750)
- `EffectsPlayScriptType` (0xF755)
- `CommunicationTextboxString` (0xF7E0)
- `ItemObjDescEvent` (0xF625)

---

### 8. `src/messages/s2c.rs` (1,049 lines)

**Server-to-Client Message Parsing**

Comprehensive message handlers for S2C communication.

**GameEventType Enum (75 variants):**
Game event opcodes for ordered events (0xF7B0), organized by category:
- Allegiance: `AllegianceUpdate`, `AllegianceUpdateAborted`, `AllegianceLoginNotificationEvent`
- Communication: `PopUpString`, `ChannelBroadcast`, `HearDirectSpeech`, `TransientString`
- Social: `FriendsUpdate`, `CharacterTitleTable`, `SendClientContractTracker`
- Item: `ServerSaysContainId`, `WearItem`, `SetAppraiseInfo`, `UseDone`, `AppraiseDone`
- Fellowship: `Quit`, `Dismiss`, `FullUpdate`, `Disband`, `UpdateFellow`
- Writing: `BookOpen`, `BookAddPageResponse`, `BookDeletePageResponse`
- Trade: `RegisterTrade`, `OpenTrade`, `CloseTrade`, `AcceptTrade`, etc.
- House: `HouseProfile`, `HouseData`, `HouseStatus`, `AvailableHouses`
- Combat: `HandleAttackDoneEvent`, `QueryHealthResponse`, `HandleVictimNotificationEvent`
- Magic: `UpdateSpell`, `UpdateEnchantment`, `RemoveEnchantment`, `DispelEnchantment`
- Game (chess): `JoinGameResponse`, `StartGame`, `MoveResponse`, `GameOver`
- Misc: `PortalStormBrewing`, `PortalStormImminent`, `PortalStorm`

**Implemented Game Event Handlers (6):**
- `Character_CharacterOptionsEvent` (0x00F7)
- `Item_SetAppraiseInfo` (0x00C9)
- `Item_ServerSaysContainId` (0x0022)
- `Item_WearItem` (0x0023)
- `Magic_UpdateEnchantment` (0x02C2)
- `Magic_DispelEnchantment` (0x02C7)

**S2C Message Structures:**
1. `QualitiesPrivateUpdateInt`: Property int update (sequence, key, value)
2. `QualitiesPrivateUpdateAttribute2ndLevel`: Vital level update (Health/Stamina/Mana)
3. `QualitiesUpdateInt`: Object property update with object_id
4. `QualitiesUpdateInstanceId`: Object instance property update
5. `MovementSetObjectMovement`: Object movement data with MovementData
6. `InventoryPickupEvent`: Item pickup (object_id, sequences)
7. `EffectsSoundEvent`: Sound playback (type, volume)
8. `EffectsPlayScriptType`: Animation/script playback
9. `CommunicationTextboxString`: Chat message (text, type)
10. `ItemObjDescEvent`: Object description update

**MovementData Structure:**
```rust
pub struct MovementData {
    pub object_movement_sequence: u16,
    pub object_server_control_sequence: u16,
    pub autonomous: u8,
    pub movement_type: String,  // Invalid, General, RawCommand, etc.
}
```

Movement Types: Invalid (0), General (1), RawCommand (2), InterpertedMotionState (3), StopCompletely (4), MoveToObject (5), MoveToPosition (6), TurnToObject (7), TurnToHeading (8), Jump (9)

**Helper Functions:**
- `property_int_name(key: u32) -> String`: Maps property IDs to names
- `vital_name(key: u32) -> String`: Maps vital IDs (1=Health, 2=Stamina, 3=Mana)
- `property_instance_id_name(key: u32) -> String`: Maps instance ID property names

---

### 9. `src/messages/c2s.rs` (171 lines)

**Client-to-Server Message Parsing**

Handles client action messages.

**GameActionType Enum (4 variants):**
- `ItemAppraise` (0x00C8): Appraise item
- `InventoryPutItemInContainer` (0x0019): Store item in container
- `InventoryWieldItem` (0x001A): Equip item
- `Unknown` (0xFFFFFFFF): Unrecognized action

**C2S Action Structures:**

1. `ItemAppraise`:
   ```rust
   pub struct ItemAppraise {
       pub object_id: u32,
       pub ordered_sequence: u32,
       pub action_type: String,
       pub opcode: u32,       // 0xF7B1
       pub message_type: String,
       pub message_direction: String,
   }
   ```

2. `InventoryPutItemInContainer`:
   ```rust
   pub struct InventoryPutItemInContainer {
       pub object_id: u32,
       pub container_id: u32,
       pub slot_index: u32,
       // ... metadata
   }
   ```

3. `InventoryWieldItem`:
   ```rust
   pub struct InventoryWieldItem {
       pub object_id: u32,
       pub wield_location: u32,
       // ... metadata
   }
   ```

---

## Entry Points & Execution Flow

### Main Function Flow

1. **Parse PCAP File**
   ```
   File → BinaryBuffer → LegacyPcapReader
   ```

2. **Extract UDP Packets**
   - Read all PCAP blocks
   - Skip non-legacy packets
   - Filter by payload size (>42 bytes for Ethernet/IP/UDP headers)
   - Detect direction from source port

3. **Packet Processing**
   - Parse packet header (20 bytes)
   - Check `BLOB_FRAGMENTS` flag
   - If present, parse fragments within packet boundary

4. **Fragment Handling**
   - Read fragment header (16 bytes)
   - Add chunk to pending fragment
   - Check if complete
   - If complete: reassemble and parse message

5. **Message Parsing**
   - Read opcode (u32)
   - Route to S2C or C2S handler
   - Parse message-specific data
   - Output as JSON

6. **Output**
   - Each complete message serialized to stdout (JSONL format)
   - One JSON object per line
   - Stderr logs packet/message counts

### Output Format

**ParsedMessage JSON:**
```json
{
  "Id": 0,
  "Type": "Message_Type_Name",
  "Data": { /* message-specific data */ },
  "Direction": "Send|Recv",
  "OpCode": "F7B1"
}
```

---

## Code Patterns & Conventions

### 1. **Error Handling**
- Uses `anyhow::Result<T>` throughout
- `.context()` for error messages
- `bail!()` macro for immediate errors
- `.unwrap_or()` for fallback values

### 2. **Binary Parsing**
- Little-endian reads via `BinaryReader`
- Position tracking for packet boundaries
- Validation of data length before reading
- Graceful handling of incomplete data

### 3. **Serialization**
- `#[derive(Serialize)]` for all output structures
- `#[serde(rename = "FieldName")]` for uppercase JSON keys
- Custom serializers for special types (hex, base64)
- `serde_json::json!()` for dynamic objects

### 4. **Enums & Matching**
- `#[repr(u32)]` for opcode enums
- `from_u32()` conversion methods
- Exhaustive pattern matching with `Unknown` fallback
- Named getter methods (e.g., `.name()` for GameEventType)

### 5. **Data Structures**
- Most structures use `Option<T>` for optional fields
- HashMap for stateful collections (pending fragments)
- Vec<u8> for binary data buffers
- Custom numeric types for semantics (e.g., u16 for sequences)

### 6. **Type Safety**
- Strong typing for different IDs (object_id, sequence, etc.)
- Wrapper types for semantic meaning
- No type aliases (raw types used)

### 7. **Naming Conventions**
- PascalCase for types, constants
- snake_case for variables and methods
- Prefix/suffix convention: `read_*` for readers, `parse_*` for parsers
- Enum variants match game protocol terminology

### 8. **Module Organization**
- One major concept per file
- Submodules in `messages/` for specialization
- Clear public/private boundaries
- No circular dependencies

### 9. **Comments & Documentation**
- Line comments (`//`) for explanation
- Doc comments (`///`) minimal (mainly derive docs)
- Clear variable naming reduces need for comments
- Some TODOs for unimplemented message types

---

## Testing & Test Data

### Test Artifacts

**PCAP Test File:**
- `pkt_2025-11-18_1763490291_log.pcap` (~196 KB)
- Contains real AC game protocol traffic
- Used for parser validation

**Expected Output:**
- `expected.json` (~5 KB): Reference parsed messages
- Contains 5 example messages with full detail
- Used to validate parser correctness

**Generated Output:**
- `our_output.json`: Parser output for comparison
- `our.jsonl`: JSONL format output
- `messages.json` (~927 KB): Full message dump

**Large Outputs:**
- `fragments.json` (~1.5 MB): Raw fragment data
- `result.js` (~2.7 MB): JavaScript format dump

### Current Testing Status

**No automated test suite configured:**
- No `#[cfg(test)]` modules
- No cargo test infrastructure
- Manual testing via PCAP file parsing
- Comparison against expected.json

**How to Test:**
```bash
cargo run > our_output.json 2>&1
# Compare with expected.json
```

---

## Key Dependencies Deep Dive

### pcap-parser 0.15
- **Purpose**: Read and parse PCAP file format
- **Key Types**: `LegacyPcapReader`, `PcapBlockOwned`, `PcapError`
- **Usage**: Opens PCAP file, iterates blocks, extracts packet data
- **Note**: Only uses legacy PCAP format (not PCAP-NG)

### serde + serde_json 1.0
- **Purpose**: Serialization to JSON format
- **Features Used**: `#[derive(Serialize)]`, custom serializers
- **Output**: Human-readable JSON objects with custom field names
- **Note**: Only serialization used (not deserialization)

### anyhow 1.0
- **Purpose**: Error handling with context
- **Key Types**: `Result<T>`, `Context` trait
- **Usage**: `.context()` for error messages, `bail!()` for early returns
- **Advantage**: Clear error chains when parsing fails

### bitflags 2.4
- **Purpose**: Efficient bitfield flag definitions
- **Key Usage**: `PacketHeaderFlags` for packet header bits
- **Features**: Bitwise operations, string representations
- **Implementation**: `bitflags!` macro for safe flag manipulation

### base64 0.22
- **Purpose**: Base64 encoding of fragment data
- **Usage**: Fragment payloads encoded to base64 strings in JSON
- **Code**: `BASE64.encode(&bytes)` where BASE64 is STANDARD engine

### hex 0.4
- **Purpose**: Hexadecimal encoding
- **Usage**: Unknown/unimplemented message data shown as hex
- **Code**: `hex::encode(&data)`

### etherparse 0.16
- **Purpose**: Ethernet/IP/UDP header parsing
- **Current Usage**: Minimal (headers manually parsed in main.rs)
- **Note**: Not actually used - headers skipped by offset calculation

---

## Known Limitations & TODO Items

### Not Yet Implemented

1. **Packet Header Parsing** (10 types bail with error)
   - `ServerSwitch` header parsing
   - `LogonServerAddr` header parsing
   - `Referral` header parsing
   - `LoginRequest` header parsing
   - `WorldLoginRequest` header parsing
   - `ConnectRequest` header parsing
   - `ConnectResponse` header parsing
   - `NetError` header parsing
   - `NetErrorDisconnect` header parsing
   - `CICMDCommand` header parsing

2. **S2C Message Parsing**
   - Most S2C message types fallback to raw hex data
   - Only ~10 specific S2C message types fully parsed
   - ~69 game event types return raw data
   - Complex property dictionaries not fully implemented
   - Movement state full parsing incomplete
   - Object description parsing incomplete

3. **C2S Message Parsing**
   - Only 3 action types implemented (of many possible)
   - Most game actions return raw hex

### Partial Implementations
- `ItemSetAppraiseInfo`: Basic fields only, property dictionaries skipped
- `MovementSetObjectMovement`: Basic movement data only, movement params skipped
- `ItemObjDescEvent`: Simplified, skips complex descriptor
- `CharacterCharacterOptionsEvent`: Skips actual options data
- `MagicUpdateEnchantment`: Skips enchantment details

### Protocol Gaps
- PCAP-NG format not supported (only legacy PCAP)
- Non-UDP packets fully ignored
- No packet reassembly for IP-level fragmentation
- Assumes AC server on ports 9000-9013

---

## Development Workflow

### Build
```bash
cargo build              # Debug build
cargo build --release   # Optimized build
```

### Run
```bash
cargo run                              # Uses default PCAP file
cargo run > output.json 2>&1          # Capture all output
RUST_BACKTRACE=1 cargo run            # Full backtrace on panic
```

### Check
```bash
cargo check             # Quick compilation check
cargo clippy            # Lint suggestions
cargo fmt               # Format code
```

---

## Common Tasks for AI Assistants

### Adding a New S2C Message Type

1. Add variant to `S2CMessageType` enum in `enums.rs`
2. Add case to `S2CMessageType::from_u32()` match
3. Create struct in `messages/s2c.rs` with fields
4. Implement `read()` method using `BinaryReader`
5. Add case to `parse_s2c_message()` in `messages/mod.rs`

**Example:**
```rust
// In enums.rs - add variant
ItemNewMessage = 0x1234,

// In from_u32() - add match arm
0x1234 => S2CMessageType::ItemNewMessage,

// In s2c.rs - add struct
#[derive(Debug, Clone, Serialize)]
pub struct ItemNewMessage {
    #[serde(rename = "ObjectId")]
    pub object_id: u32,
    // ... fields
}

impl ItemNewMessage {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        let object_id = reader.read_u32()?;
        Ok(Self { object_id, ... })
    }
}

// In mod.rs parse_s2c_message() - add handler
S2CMessageType::ItemNewMessage => {
    let msg = s2c::ItemNewMessage::read(reader)?;
    ("Item_NewMessage".to_string(), serde_json::to_value(&msg)?)
}
```

### Adding a New C2S Action Type

1. Add variant to `GameActionType` enum in `messages/c2s.rs`
2. Add case to `GameActionType::from_u32()` match
3. Create struct in `messages/c2s.rs`
4. Implement `read()` method
5. Add case to `parse_game_action()` dispatcher

### Adding a New Game Event Type

1. Add variant to `GameEventType` enum in `messages/s2c.rs`
2. Add case to `GameEventType::from_u32()` match
3. Add case to `GameEventType::name()` method
4. Create struct for the event data
5. Implement `read()` method
6. Add case to `parse_game_event()` function

### Implementing a Packet Header Type

1. Fill in struct fields in `packet.rs` (currently empty `// TODO: fill in fields`)
2. Remove the `bail!()` call in `PacketHeader::parse()`
3. Implement actual parsing logic
4. Test with PCAP data that contains this header type

### Debugging Fragment Reassembly

- Fragment HashMap keyed by `sequence` number
- Check `Fragment::is_complete()` before message parsing
- Verify `add_chunk()` correctly handles index mapping
- Monitor `CHUNK_SIZE` (448 bytes) alignment

### Handling Protocol Variations

- Direction detection is port-based (may need other heuristics)
- Message routing via opcode lookup (add unknown handlers)
- Field sizes documented in header constants
- Alignment padding documented in `read_string16l()`

---

## Architecture Notes

### Design Philosophy
- **Stateful**: Maintains fragment reassembly state
- **Streaming**: Processes packets sequentially
- **Type-Safe**: Strong typing for protocol constants
- **Lossy**: Unknown message types don't block processing
- **Fail-Fast**: Unimplemented headers bail immediately

### Performance Characteristics
- Single-threaded processing
- O(1) fragment lookup via HashMap
- Binary data processed in single pass
- No allocations for completed fragments

### Extension Points
1. Message parsing: Add more message types in s2c.rs/c2s.rs
2. Output format: Modify `ParsedMessage` serialization
3. Direction detection: Enhance port/opcode logic
4. Fragment handling: Add compression support if needed

---

## References & Related Info

**Protocol Basis:**
- This parser targets Asheron's Call (AC) game network protocol
- Protocol is proprietary but well-documented by community
- Message format: opcode-based binary protocol with variable-length fields
- Transport: UDP over IP with custom fragmentation

**Game Context:**
- AC uses ServerToClient and ClientToServer message patterns
- Fragment groups: Event, Private, Object
- Common message types: Movement, Items, Combat, Magic, Social
- State managed via qualities (properties) on server

**Output Examples:**
See expected.json for sample parsed messages with full field detail

---

## Quick Reference

### File Summary
| File | Lines | Purpose |
|------|-------|---------|
| `main.rs` | 337 | Entry point, PCAP parsing, fragment management |
| `packet.rs` | 341 | Packet header structure (21 flags) |
| `fragment.rs` | 123 | Fragment reassembly (448-byte chunks) |
| `reader.rs` | 130 | Binary utilities (12 read methods) |
| `enums.rs` | 227 | Opcode definitions (S2C/C2S) |
| `message.rs` | 28 | Message wrapper (minimal) |
| `messages/mod.rs` | 181 | Message dispatcher/router |
| `messages/s2c.rs` | 1,049 | S2C parsing (75 event types, 10 handlers) |
| `messages/c2s.rs` | 171 | C2S parsing (3 action handlers) |
| **Total** | **2,587** | |

### Implementation Status
| Category | Defined | Implemented |
|----------|---------|-------------|
| Packet Header Flags | 21 | 6 fully, 10 bail |
| S2C Message Types | 56 | ~10 |
| Game Event Types | 75 | 6 |
| C2S Action Types | 3 | 3 |
