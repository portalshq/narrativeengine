#!/usr/bin/env bash
set -euo pipefail

# Run Portals Cloud lore server integration tests
# Requires: Environment variables set for cloud authentication

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Check required environment variables
export NAP_LORE_URL_BASE="${NAP_LORE_URL_BASE:-grpcs://lore.portals.sh}"

if [ -z "${NAP_WORKSPACE_ID:-}" ]; then
    echo "Error: NAP_WORKSPACE_ID environment variable not set"
    echo "Example: export NAP_WORKSPACE_ID='your-workspace-id'"
    exit 1
fi

if [ -z "${PORTALS_CLOUD_API_KEY:-}" ]; then
    echo "Error: PORTALS_CLOUD_API_KEY is not set"
    echo "Inject a revocable service-account API key from the CI secret store"
    exit 1
fi

echo "Running Portals Cloud integration tests..."
echo "NAP_LORE_URL_BASE: $NAP_LORE_URL_BASE"
echo "NAP_WORKSPACE_ID: $NAP_WORKSPACE_ID"
echo ""

cd "$ROOT_DIR"
cargo run -p nap-cli -- auth login --api-key
trap 'cargo run -p nap-cli -- auth logout >/dev/null 2>&1 || true' EXIT
cargo test -p nap-cli --test cloud_lore_suite --features lore-e2e -- --test-threads=1 "$@"
