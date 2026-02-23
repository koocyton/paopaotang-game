#!/bin/bash
set -e

echo "=== Building Paopaotang ==="

echo "[1/3] Building WASM client..."
cd client
wasm-pack build --target web --out-dir www/pkg --no-typescript
cd ..

echo "[2/3] Building server..."
cargo build --release -p server

echo "[3/3] Done!"
echo ""
echo "To run the game:"
echo "  cargo run --release -p server"
echo ""
echo "Then open http://localhost:3000 in two browser tabs"
