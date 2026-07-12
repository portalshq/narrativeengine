#!/usr/bin/env bash
set -euo pipefail

# Run local lore server integration tests
# Requires: Local lore server running at lore://localhost:41337

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Running local lore server integration tests..."
echo "Ensure lore server is running at lore://localhost:41337"
echo ""

cd "$ROOT_DIR"
cargo test -p nap-cli --test local_lore_suite --features lore-e2e -- --test-threads=1 "$@"
