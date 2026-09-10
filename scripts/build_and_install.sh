#!/usr/bin/env bash
set -e

LOCAL_INSTALL_DIR="$HOME/.local/bin"
existing="$(command -v px 2>/dev/null || true)"
if [[ -n "$existing" || -e "$LOCAL_INSTALL_DIR/px" ]]; then
    echo "Refusing to overwrite existing px executable (${existing:-$LOCAL_INSTALL_DIR/px})." >&2
    echo "Remove it explicitly or choose a different installation directory." >&2
    exit 1
fi

# Build and install the PX CLI.
echo "Building portalshq-px-cli..."
cargo build --release -p portalshq-px-cli

echo "Installing px to cargo bin..."
cargo install --path crates/px-cli

echo "Copying to ~/.local/bin..."
mkdir -p "$LOCAL_INSTALL_DIR"
cp target/release/px "$LOCAL_INSTALL_DIR/px"
chmod +x "$LOCAL_INSTALL_DIR/px"

echo "Done."
