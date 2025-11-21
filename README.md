# AC PCAP Parser

A Rust application that parses PCAP (packet capture) files containing Asheron's Call (AC) game network traffic. It extracts, reassembles fragmented UDP packets, and decodes binary messages from the AC network protocol into JSON format.

## Features

- Parse PCAP files with AC network traffic
- Reassemble fragmented UDP packets
- Decode Server-to-Client (S2C) and Client-to-Server (C2S) messages
- CLI with filtering, sorting, and multiple output formats
- Interactive TUI with tabs for messages and fragments

## Installation

```bash
cargo build --release
```

## Usage

### Basic Usage (Default: Output Messages as JSONL)

```bash
# Parse default PCAP file and output messages as JSONL
ac-pcap-parser

# Specify a different PCAP file
ac-pcap-parser -f my_capture.pcap
```

### Commands

#### Summary

Show statistics about the PCAP file:

```bash
ac-pcap-parser summary
```

Example output:
```
=== PCAP Summary ===

Packets: 632
Messages: 2088

Packets by Direction:
  Send (C→S): 137
  Recv (S→C): 495

Messages by Direction:
  Send (C→S): 103
  Recv (S→C): 1985

Message Types (top 20):
  Effects_PlayScriptType                     515
  Magic_DispelEnchantment                    323
  Magic_UpdateEnchantment                    313
  ...
```

#### Messages

Output parsed messages with filtering and sorting:

```bash
# Show all messages as JSONL (default)
ac-pcap-parser messages

# Filter by message type (substring match)
ac-pcap-parser messages -t Magic

# Filter by direction
ac-pcap-parser messages -d recv

# Sort by type instead of ID
ac-pcap-parser messages -s type

# Reverse sort order
ac-pcap-parser messages -s type -r

# Limit output
ac-pcap-parser messages -l 10

# Output as formatted JSON array
ac-pcap-parser messages -o json

# Output as ASCII table
ac-pcap-parser messages -o table
```

Options:
- `-t, --filter-type <TYPE>` - Filter by message type (substring match)
- `-d, --direction <DIR>` - Filter by direction: `send` or `recv`
- `-s, --sort <FIELD>` - Sort by: `id`, `type`, or `direction` (default: `id`)
- `-r, --reverse` - Reverse sort order
- `-l, --limit <N>` - Limit number of results
- `-o, --output <FORMAT>` - Output format: `jsonl`, `json`, or `table` (default: `jsonl`)

#### Fragments

Output raw packet/fragment data:

```bash
# Show all fragments as JSONL
ac-pcap-parser fragments

# Filter by direction
ac-pcap-parser fragments -d send

# Sort by sequence number
ac-pcap-parser fragments -s sequence

# Output as table
ac-pcap-parser fragments -o table
```

Options:
- `-d, --direction <DIR>` - Filter by direction: `send` or `recv`
- `-s, --sort <FIELD>` - Sort by: `id`, `sequence`, or `direction` (default: `id`)
- `-r, --reverse` - Reverse sort order
- `-l, --limit <N>` - Limit number of results
- `-o, --output <FORMAT>` - Output format: `jsonl`, `json`, or `table` (default: `jsonl`)

#### TUI (Interactive Mode)

Launch the interactive terminal user interface:

```bash
ac-pcap-parser tui
```

TUI Features:
- **Tab** - Switch between Messages and Fragments tabs
- **↑/↓** or **j/k** - Navigate up/down
- **PgUp/PgDn** or **u/d** - Page up/down
- **Home/End** - Jump to first/last item
- **s** - Cycle sort field (ID → Type/Sequence → Direction)
- **r** - Toggle sort order (ascending/descending)
- **/** - Start search (filter by type)
- **Enter** - Show/hide detail view (full JSON)
- **Esc** - Clear search / close detail
- **q** - Quit

## Output Formats

### JSONL (Default)

One JSON object per line, suitable for streaming and processing with tools like `jq`:

```json
{"Id":0,"Type":"Magic_UpdateEnchantment","Data":{...},"Direction":"Recv","OpCode":"F7B0"}
{"Id":1,"Type":"Effects_PlayScriptType","Data":{...},"Direction":"Recv","OpCode":"F755"}
```

### JSON

Formatted JSON array, suitable for human reading:

```json
[
  {
    "Id": 0,
    "Type": "Magic_UpdateEnchantment",
    "Data": {...},
    "Direction": "Recv",
    "OpCode": "F7B0"
  }
]
```

### Table

ASCII table for quick viewing:

```
    ID  Type                                      Dir        OpCode
----------------------------------------------------------------------
     0  Magic_UpdateEnchantment                   Recv       F7B0
     1  Effects_PlayScriptType                    Recv       F755
```

## Message Types

The parser supports various AC protocol message types including:

### Server-to-Client (S2C)
- `Magic_UpdateEnchantment` - Enchantment applied to object
- `Magic_DispelEnchantment` - Enchantment removed
- `Item_SetAppraiseInfo` - Item appraisal results
- `Item_ObjDescEvent` - Object description update
- `Movement_SetObjectMovement` - Object movement state
- `Effects_PlayScriptType` - Visual effects
- `Effects_SoundEvent` - Sound effects
- `Communication_TextboxString` - Chat messages
- `Qualities_*` - Property updates

### Client-to-Server (C2S)
- `Item_Appraise` - Request item appraisal
- `Inventory_PutItemInContainer` - Store item
- `Inventory_GetAndWieldItem` - Equip item
- `Character_CharacterOptionsEvent` - Player options

## Examples

### Filter enchantment messages and output as JSON
```bash
ac-pcap-parser messages -t enchantment -o json > enchantments.json
```

### Get top 10 message types by count
```bash
ac-pcap-parser messages -o table | tail -n +3 | cut -c9-48 | sort | uniq -c | sort -rn | head -10
```

### Export all server messages
```bash
ac-pcap-parser messages -d recv > server_messages.jsonl
```

### Process with jq
```bash
ac-pcap-parser messages | jq 'select(.Type | contains("Magic"))'
```

## Building from Source

```bash
git clone <repo>
cd ac-pcap-parser
cargo build --release
```

## Dependencies

- `pcap-parser` - PCAP file parsing
- `clap` - Command-line argument parsing
- `ratatui` + `crossterm` - Terminal UI
- `serde` + `serde_json` - JSON serialization
- `anyhow` - Error handling

## License

MIT
