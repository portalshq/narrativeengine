#!/usr/bin/env bash
set -euo pipefail

# Run Portals Cloud lore server integration tests
# Requires: Environment variables set for cloud authentication

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Check required environment variables
if [ -z "${NAP_LORE_URL_BASE:-}" ]; then
    echo "Error: NAP_LORE_URL_BASE environment variable not set"
    echo "Example: export NAP_LORE_URL_BASE='lore://cloud.portals.ai'"
    exit 1
fi

if [ -z "${NAP_WORKSPACE_ID:-}" ]; then
    echo "Error: NAP_WORKSPACE_ID environment variable not set"
    echo "Example: export NAP_WORKSPACE_ID='your-workspace-id'"
    exit 1
fi

echo "Running Portals Cloud integration tests..."
echo "NAP_LORE_URL_BASE: $NAP_LORE_URL_BASE"
echo "NAP_WORKSPACE_ID: $NAP_WORKSPACE_ID"
echo ""

cd "$ROOT_DIR"
cargo test -p nap-cli --test cloud_lore_suite --features lore-e2e -- --test-threads=1 "$@"
