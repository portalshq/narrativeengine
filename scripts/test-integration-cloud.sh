#!/usr/bin/env bash
set -euo pipefail

# Run Portals Cloud lore server integration tests
# Requires: Environment variables set for cloud authentication

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Check required environment variables
export NAP_LORE_URL_BASE="${NAP_LORE_URL_BASE:-grpcs://lore.portals.works}"

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

# Allow PORTALS_CLOUD_API_KEY to be a file path (e.g. CI secret mount) — read it if so
if [ -f "${PORTALS_CLOUD_API_KEY}" ]; then
    PORTALS_CLOUD_API_KEY="$(cat "${PORTALS_CLOUD_API_KEY}")"
    export PORTALS_CLOUD_API_KEY
fi

# --- Artifact emit: explicit directory, created if missing (best modern practice) ---
# Keep artifacts out of docs/security — use a dedicated, git-tracked directory.
# Simple, robust, extensible: one timestamped run dir per invocation with
# human log + machine JSON + env metadata. Extensible to JUnit via cargo2junit/nextest later.
ARTIFACT_ROOT="${ARTIFACT_ROOT:-$ROOT_DIR/artifacts/e2e}"
RUN_ID="$(date -u +%Y-%m-%d_%H%M%S)_$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || echo "nogit")"
ARTIFACT_DIR="$ARTIFACT_ROOT/$RUN_ID"
mkdir -p "$ARTIFACT_DIR"

# Record run metadata (never secrets) for traceability
cat > "$ARTIFACT_DIR/env.json" <<EOF
{
  "timestamp_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "nap_lore_url_base": "$NAP_LORE_URL_BASE",
  "nap_workspace_id": "$NAP_WORKSPACE_ID",
  "git_sha": "$(git -C "$ROOT_DIR" rev-parse HEAD 2>/dev/null || echo "unknown")",
  "git_branch": "$(git -C "$ROOT_DIR" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")",
  "lore_client_version": "$(grep -E '^version' "$ROOT_DIR/Cargo.toml" 2>/dev/null | head -1 || echo "unknown")",
  "artifact_dir": "$ARTIFACT_DIR"
}
EOF

echo "Running Portals Cloud integration tests..."
echo "NAP_LORE_URL_BASE: $NAP_LORE_URL_BASE"
echo "NAP_WORKSPACE_ID: $NAP_WORKSPACE_ID"
echo "Artifact dir: $ARTIFACT_DIR"
echo ""

cd "$ROOT_DIR"
# Authenticate via service-account API key (functional integration: exchanges HMAC key for JWT via 8086 internal API)
if ! cargo run -p nap-cli -- auth login --api-key 2>&1 | tee "$ARTIFACT_DIR/auth-login.log"; then
    echo "ERROR: nap auth login --api-key failed — key not exchanged for JWT (check KMS/JWKS/pepper)" | tee -a "$ARTIFACT_DIR/auth-login.log"
    exit 1
fi
trap 'cargo run -p nap-cli -- auth logout >/dev/null 2>&1 || true; echo "Artifacts: $ARTIFACT_DIR" ' EXIT

# Run suite: human log + machine JSON (extensible to JUnit via cargo2junit/nextest)
# --format json is unstable; we tee human output and also try json for future parsing
set +e
cargo test -p nap-cli --test cloud_lore_suite --features lore-e2e -- --test-threads=1 --nocapture "$@" 2>&1 | tee "$ARTIFACT_DIR/nap-cloud-e2e.log"
TEST_RC=${PIPESTATUS[0]}
# Also emit JSON for machine parsing if cargo supports it (non-fatal if not)
cargo test -p nap-cli --test cloud_lore_suite --features lore-e2e -- --test-threads=1 --format=json 2>"$ARTIFACT_DIR/nap-cloud-e2e.json" || true
# If cargo2junit or nextest is available, produce JUnit (non-fatal, extensible)
if command -v cargo2junit >/dev/null 2>&1; then
    cargo2junit < "$ARTIFACT_DIR/nap-cloud-e2e.log" > "$ARTIFACT_DIR/junit.xml" 2>/dev/null || true
fi
echo "Artifacts written to $ARTIFACT_DIR (log, json, env.json, junit.xml if available)"
exit $TEST_RC
