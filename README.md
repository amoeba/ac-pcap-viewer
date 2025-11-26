# AC PCAP Parser

A parser and viewer for Asheron's Call network packet captures. Decodes PCAP files into human-readable JSON, available as a web app, desktop app, or CLI tool.

## About This Project

This entire repo is vibe coded using [Claude Code](https://github.com/anthropics/claude-code). The goal is to see how far I can get without writing any of the code myself.

None of this would be possible without [trevis](https://github.com/trevis) open sourcing his network parsing code and maintaining [protocol.xml](https://github.com/ACClientLib/ACProtocol). The real intellectual ownership of the parsing logic goes to him.

## Features

**Core**
- Parse PCAP files containing AC network traffic
- Reassemble fragmented UDP packets
- Decode Server-to-Client and Client-to-Server messages
- Full property dictionaries (int, float, bool, string, data IDs)
- Enchantment, movement, and object description parsing

**Web/Desktop UI**
- Drag-and-drop file loading
- Load PCAP from URL (query param or dialog)
- Interactive JSON tree viewer
- Hex editor for binary data
- Search and filter parsed packets
- Time scrubber with packet density visualization
- Dark/light mode toggle
- Responsive layout (mobile-friendly)
- Native file dialogs and menus (desktop)

**CLI**
- Multiple output formats (JSONL, JSON, table)
- Filter by message type and direction
- Sort by id, type, or direction
- Summary statistics
- Interactive TUI mode

## Usage

### Web

Try it at the hosted version, or run locally:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo xtask web --serve
```

Open http://localhost:8080. Drag and drop a PCAP file or load one via URL parameter (`?url=https://...`).

### Desktop

```bash
cargo xtask desktop --run
```

Supports native file dialogs and drag-and-drop.

### CLI

```bash
# Build
cargo build --release

# Parse and output messages as JSONL
ac-pcap-parser -f capture.pcap

# Summary statistics
ac-pcap-parser summary -f capture.pcap

# Filter and format
ac-pcap-parser messages -t Magic -o table
ac-pcap-parser messages -d recv -l 10

# Interactive TUI
ac-pcap-parser tui
```

CLI options:
- `-t, --filter-type <TYPE>` - Filter by message type
- `-d, --direction <DIR>` - Filter by direction: `send` or `recv`
- `-s, --sort <FIELD>` - Sort by: `id`, `type`, or `direction`
- `-o, --output <FORMAT>` - Output: `jsonl`, `json`, or `table`
- `-l, --limit <N>` - Limit results

## Development Setup

Requirements: Rust (stable)

```bash
git clone https://github.com/amoeba/ac-pcap-parser
cd ac-pcap-parser
cargo build
```

For web development:
```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo xtask web --serve
```

For desktop:
```bash
cargo xtask desktop --run
```

### Project Structure

```
├── src/                  # Core parser library
├── crates/
│   ├── web/              # GUI (desktop + web, shared codebase)
│   └── xtask/            # Build tasks
```

### Build Tasks

Uses the [xtask pattern](https://github.com/matklad/cargo-xtask) for build automation:

```bash
cargo xtask web --serve      # Build and serve web app
cargo xtask desktop --run    # Build and run desktop app
cargo xtask --help           # List all tasks
```

## Contributing

1. Fork the repo
2. Create a branch
3. Make changes
4. Open a PR

Since this is a vibe coding experiment, feel free to submit issues describing what you want and I'll see if Claude Code can implement it.

## License

MIT
