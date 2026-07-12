#!/usr/bin/env bash
set -e

# Build and install nap-cli
echo "Building nap-cli..."
cargo build --release -p nap-cli

echo "Installing nap-cli to cargo bin..."
cargo install --path crates/nap-cli --force

echo "Copying to ~/.local/bin..."
mkdir -p ~/.local/bin
cp target/release/nap ~/.local/bin/nap
chmod +x ~/.local/bin/nap

echo "Done."
