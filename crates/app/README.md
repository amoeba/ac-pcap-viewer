# AC PCAP Web

> **Note:** This project was entirely vibe coded based on [Trevis's](https://github.com/trevis) work. All credit goes to Trevis and the AI researchers who made this possible.

WebAssembly-based web interface for the AC PCAP parser.

## Prerequisites

Before building, install the required toolchain and tools:

```bash
# Add the WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen CLI
cargo install wasm-bindgen-cli
```

## Building

```bash
./build.sh
```

## Development Server

To build and start a local development server:

```bash
./build.sh --serve
```

This will serve the app at http://localhost:8080.

## Build Output

The build outputs to `pkg/`:
- `ac_pcap_web.wasm` - The compiled WebAssembly module
- `ac_pcap_web.js` - JavaScript bindings
- `index.html` - The web interface
