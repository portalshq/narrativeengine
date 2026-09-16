#!/usr/bin/env bash
set -euo pipefail

FORCE=false
if [[ "${1:-}" == "--force" ]]; then
    FORCE=true
    shift
fi
if [[ $# -ne 0 ]]; then
    echo "Usage: $0 [--force]" >&2
    exit 2
fi

LOCAL_INSTALL_DIR="$HOME/.local/bin"
target="$LOCAL_INSTALL_DIR/px"
if [[ -e "$target" ]] && ! { [[ -x "$target" ]] && "$target" --version 2>/dev/null | grep -Eiq '^px([ -]|$)'; } && [[ "$FORCE" != true ]]; then
    echo "Refusing to overwrite unknown executable '$target'." >&2
    echo "Re-run with --force if this is intentional." >&2
    exit 1
fi

# Build and install the PX CLI.
echo "Building portalshq-px-cli..."
cargo build --release -p portalshq-px-cli

echo "Copying to ~/.local/bin..."
mkdir -p "$LOCAL_INSTALL_DIR"
cp target/release/px "$LOCAL_INSTALL_DIR/px"
chmod +x "$LOCAL_INSTALL_DIR/px"

echo "Done."
