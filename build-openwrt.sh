#!/bin/bash
set -e

TARGET="aarch64-unknown-linux-musl"
BINARY_NAME="server"

echo "=== Building Paopaotang for OpenWrt ARM64 (MT7981BA) ==="
echo "Target: ${TARGET}"
echo ""

echo "[1/3] Building WASM client..."
cd client
wasm-pack build --target web --out-dir www/pkg --no-typescript
cd ..

echo "[2/3] Cross-compiling server for ${TARGET}..."
RUSTFLAGS="-C target-feature=+crt-static" \
  cargo build --release -p server --target "${TARGET}"

BINARY="target/${TARGET}/release/${BINARY_NAME}"
if [ ! -f "${BINARY}" ]; then
    echo "ERROR: Build failed, binary not found at ${BINARY}"
    exit 1
fi

echo "[3/3] Packaging..."
DIST_DIR="dist/openwrt-arm64"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}/client/www/pkg"

cp "${BINARY}" "${DIST_DIR}/"
aarch64-unknown-linux-musl-strip "${DIST_DIR}/${BINARY_NAME}"
cp client/www/index.html "${DIST_DIR}/client/www/"
cp client/www/style.css "${DIST_DIR}/client/www/"
cp client/www/pkg/client_bg.wasm "${DIST_DIR}/client/www/pkg/"
cp client/www/pkg/client.js "${DIST_DIR}/client/www/pkg/"

BINARY_SIZE=$(du -h "${DIST_DIR}/${BINARY_NAME}" | cut -f1)
TOTAL_SIZE=$(du -sh "${DIST_DIR}" | cut -f1)

echo ""
echo "=== Build Complete ==="
echo "Binary: ${DIST_DIR}/${BINARY_NAME} (${BINARY_SIZE})"
echo "Package: ${DIST_DIR}/ (${TOTAL_SIZE})"
echo ""
echo "Deploy to OpenWrt:"
echo "  scp -r ${DIST_DIR}/* root@<router-ip>:/opt/paopaotang/"
echo "  ssh root@<router-ip> 'cd /opt/paopaotang && ./${BINARY_NAME}'"
echo ""
echo "Then open http://<router-ip>:3000 in your browser"
