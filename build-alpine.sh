#!/bin/sh
# build-alpine.sh - Build UnEgg for Alpine Linux (musl, static)
# Run this on an Alpine system or inside an Alpine container

set -e

echo "=== UnEgg Alpine/musl static build ==="

# Install build deps if needed
if [ -f /etc/alpine-release ]; then
    apk add --no-cache g++ make 2>/dev/null || true
fi

cd "$(dirname "$0")"

# Clean and build static
make clean
make -j$(nproc 2>/dev/null || echo 2) \
    CXX=g++ \
    CC=gcc \
    CXXFLAGS="-O2 -Wall -Wno-unused -Wno-multichar -std=c++11 -D_7ZIP_ST" \
    CFLAGS="-O2 -Wall -Wno-unused -Wno-multichar -D_7ZIP_ST" \
    LDFLAGS="-static"

echo ""
echo "=== Build complete ==="
ls -lh unegg
echo ""
echo "Binary type:"
file unegg 2>/dev/null || true
echo ""
echo "Dependencies:"
ldd unegg 2>/dev/null || echo "Static binary - no dynamic dependencies"
echo ""
echo "Test:"
./unegg -h || true
