#!/bin/bash
# Legacy build script - use cargo xtask instead

echo "This build script is deprecated."
echo ""
echo "Use cargo xtask instead:"
echo "  cargo xtask web          # Build WASM"
echo "  cargo xtask web --serve  # Build and serve"
echo "  cargo xtask web --small  # Smaller WASM (opt-level z)"
echo ""
echo "Running: cargo xtask web $@"
cd "$(dirname "$0")/../.."
exec cargo xtask web "$@"
