#!/bin/bash
set -e

cd "$(dirname "$0")"

# Parse arguments
SERVE=false
PORT=8080
for arg in "$@"; do
    case $arg in
        --serve)
            SERVE=true
            ;;
        --port=*)
            PORT="${arg#*=}"
            ;;
        -h|--help)
            echo "Usage: ./build.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --serve       Build and start a local web server"
            echo "  --port=PORT   Port for web server (default: 8080)"
            echo "  -h, --help    Show this help"
            exit 0
            ;;
    esac
done

echo "Building WASM..."
cargo build -p ac-pcap-web --release --target wasm32-unknown-unknown

echo "Generating JS bindings..."
wasm-bindgen \
    --target web \
    --out-dir pkg \
    --no-typescript \
    ../../target/wasm32-unknown-unknown/release/ac_pcap_web.wasm

echo "Copying index.html..."
cp index.html pkg/

echo "Copying example PCAP..."
cp ../../pkt_2025-11-18_1763490291_log.pcap pkg/example.pcap

echo ""
echo "Build complete! Files in crates/web/pkg/"
ls -lh pkg/

if [ "$SERVE" = true ]; then
    echo ""
    echo "Starting web server on http://localhost:$PORT"
    echo "Press Ctrl+C to stop"
    cd pkg && python3 -m http.server "$PORT"
else
    echo ""
    echo "To test locally:"
    echo "  ./build.sh --serve"
    echo "  # or manually:"
    echo "  cd pkg && python3 -m http.server 8080"
fi
