#!/usr/bin/sh

set -e

cd "$(dirname "$0")"

TEMP_DIR="$(mktemp -d)"
echo "TEMP_DIR: $TEMP_DIR"

echo "Building..."
wasm-pack build --release --target web

echo "Copying files..."
cp -r pkg/* "$TEMP_DIR"
cp -r assets/* "$TEMP_DIR"

ls -l "$TEMP_DIR"

cd "$TEMP_DIR"
echo "Starting server..."
python3 -m http.server 8080