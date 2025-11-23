# AC PCAP Parser

A Rust application that parses PCAP (packet capture) files containing Asheron's Call (AC) game network traffic. It extracts, reassembles fragmented UDP packets, and decodes binary messages from the AC network protocol into JSON format.

**Runs on Desktop and Web** - The graphical interface works as both a native desktop application and in the browser via WebAssembly, sharing the same codebase.

## Quick Start - Desktop & Web GUI

The easiest way to use this tool is with the graphical interface, available as both a **native desktop app** and a **web app**.

### Desktop Application

```bash
# Build and run the desktop app
cargo xtask desktop --run

# Or build release version
cargo xtask desktop --release --run
```

Features:
- Native file dialogs ("Open File...")
- Drag-and-drop PCAP files
- Fast native performance
- Works offline

### Web Application

```bash
# Prerequisites (one-time setup)
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli

# Build and serve locally
cargo xtask web --serve
```

Then open http://localhost:8080 in your browser.

Features:
- No installation required
- Drag-and-drop PCAP files
- Load files from URL (`?url=https://...`)
- Runs entirely in the browser

### GUI Features (Both Platforms)

- Browse parsed messages and raw packet fragments
- Search and filter by message type
- Sort by ID, type, or direction
- JSON tree view for message details
- Dark/light theme toggle
- Responsive layout (mobile-friendly on web)

---

## Features

- Parse PCAP files with AC network traffic
- Reassemble fragmented UDP packets
- Decode Server-to-Client (S2C) and Client-to-Server (C2S) messages
- **Desktop GUI** with native file dialogs
- **Web GUI** via WebAssembly
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

## GUI (Desktop & Web)

The graphical interface is built with [egui](https://github.com/emilk/egui)/[eframe](https://docs.rs/eframe), which supports both native desktop and WebAssembly from the same codebase.

### Architecture

```
crates/web/
├── src/
│   ├── lib.rs              # Shared UI code (99% of the app)
│   └── bin/desktop.rs      # Desktop entry point
├── Cargo.toml              # Platform-specific dependencies via cfg()
└── index.html              # Web entry point
```

Platform-specific code uses `#[cfg(target_arch = "wasm32")]` / `#[cfg(not(target_arch = "wasm32"))]` for:
- File dialogs (desktop: `rfd` crate, web: drag-drop only)
- URL loading (web only)
- Logging setup

### Building Desktop

```bash
# Debug build
cargo xtask desktop

# Release build
cargo xtask desktop --release

# Build and run
cargo xtask desktop --run
cargo xtask desktop --release --run

# Or directly with cargo
cargo run -p web --bin ac-pcap-viewer --features desktop
```

### Building Web

Prerequisites:
```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

Build commands:
```bash
# Build the web UI
cargo xtask web

# Build and start a local dev server
cargo xtask web --serve

# Build with smaller WASM output (opt-level z)
cargo xtask web --small

# Custom port
cargo xtask web --serve --port=3000
```

Output files are placed in `crates/web/pkg/`.

## Build Tasks (xtask)

This project uses the [xtask pattern](https://github.com/matklad/cargo-xtask) for build automation. Instead of shell scripts or external tools like `make`, build tasks are implemented as a Rust binary in `crates/xtask/`.

### Why xtask?

- **No external dependencies** - works on any machine with Rust installed
- **Cross-platform** - no shell script compatibility issues
- **Type-safe** - build logic is checked by the compiler
- **IDE support** - full autocomplete and refactoring

### Available Tasks

```bash
cargo xtask --help           # List all tasks
cargo xtask web --help       # Help for a specific task
cargo xtask desktop --help   # Help for desktop task
```

| Task | Description |
|------|-------------|
| `cargo xtask web` | Build the WebAssembly UI |
| `cargo xtask desktop` | Build the native desktop app |

### Adding New Tasks

1. Add a new variant to the `Commands` enum in `crates/xtask/src/main.rs`
2. Implement the task as a function
3. Add the match arm in `main()`

Example:
```rust
#[derive(Subcommand)]
enum Commands {
    Web { /* ... */ },
    /// New task description
    NewTask {
        #[arg(long)]
        some_flag: bool,
    },
}

fn main() -> Result<()> {
    match cli.command {
        Commands::Web { .. } => build_web(..),
        Commands::NewTask { some_flag } => do_new_task(some_flag),
    }
}
```

## Deployment

The web UI can be deployed using Docker. A GitHub Actions workflow automatically builds and pushes images to GitHub Container Registry (GHCR) on every push to `main`.

### Docker Image

```bash
# Pull the latest image
docker pull ghcr.io/amoeba/ac-pcap-parser:latest

# Run locally
docker run -p 8080:80 ghcr.io/amoeba/ac-pcap-parser:latest
```

### Dokku Deployment

Deploy to [Dokku](https://dokku.com) using a pre-built image (no Rust compilation on your server):

```bash
# On your Dokku server
dokku apps:create ac-pcap-parser

# Deploy from GHCR image
dokku git:from-image ac-pcap-parser ghcr.io/amoeba/ac-pcap-parser:latest

# Optional: Set up domain
dokku domains:set ac-pcap-parser pcap.yourdomain.com

# Optional: Enable HTTPS with Let's Encrypt
dokku letsencrypt:enable ac-pcap-parser
```

To update to a new version:
```bash
dokku git:from-image ac-pcap-parser ghcr.io/amoeba/ac-pcap-parser:latest
```

### Building Docker Image Locally

```bash
docker build -t ac-pcap-parser .
docker run -p 8080:80 ac-pcap-parser
```

## Dependencies

- `pcap-parser` - PCAP file parsing
- `clap` - Command-line argument parsing
- `ratatui` + `crossterm` - Terminal UI
- `serde` + `serde_json` - JSON serialization
- `anyhow` - Error handling

## License

MIT
