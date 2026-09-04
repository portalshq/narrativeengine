#!/usr/bin/env bash
set -euo pipefail

# Run local Lore server integration tests. If no healthy Lore server is
# already listening on localhost, provision one through NAP in a temporary
# home and stop it when the suite exits.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LORE_HEALTH_URL="http://127.0.0.1:41339/health_check"
SERVER_HOME=""
SERVER_PID=""
STARTED_SERVER=0

cleanup() {
  local status=$?

  if [ "$STARTED_SERVER" -eq 1 ]; then
    if [ -z "$SERVER_PID" ] && [ -n "$SERVER_HOME" ] && [ -f "$SERVER_HOME/lore/pid" ]; then
      SERVER_PID="$(tr -d '[:space:]' < "$SERVER_HOME/lore/pid")"
    fi

    if [[ "$SERVER_PID" =~ ^[0-9]+$ ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
      echo "Stopping Lore server started for integration tests (PID $SERVER_PID)..."
      kill -TERM "$SERVER_PID" 2>/dev/null || true
      for _ in {1..20}; do
        kill -0 "$SERVER_PID" 2>/dev/null || break
        sleep 0.25
      done
      if kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "Lore server did not stop after SIGTERM; forcing shutdown..." >&2
        kill -KILL "$SERVER_PID" 2>/dev/null || true
        sleep 0.25
      fi
      if kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "Error: failed to stop the Lore server started for integration tests" >&2
        status=1
      fi
    fi
  fi

  if [ -n "$SERVER_HOME" ] && [ -d "$SERVER_HOME" ] && ! kill -0 "$SERVER_PID" 2>/dev/null; then
    rm -rf "$SERVER_HOME"
  fi

  trap - EXIT
  exit "$status"
}

trap cleanup EXIT

if curl --fail --silent --show-error --max-time 2 "$LORE_HEALTH_URL" >/dev/null; then
  echo "Using existing healthy Lore server at lore://localhost:41337"
else
  SERVER_HOME="$(mktemp -d "${TMPDIR:-/tmp}/nap-lore-integration.XXXXXX")"
  BOOTSTRAP_REPOSITORY="release-integration-bootstrap-$(date +%s)-$$"

  echo "Installing Lore through NAP..."
  cargo run -p nap-cli -- install lore

  echo "Starting a temporary local Lore server..."
  # Mark ownership before starting so the exit trap can clean up even if NAP
  # starts the daemon but reports a later initialization error.
  STARTED_SERVER=1
  cargo run -p nap-cli -- --base-dir "$SERVER_HOME" init "$BOOTSTRAP_REPOSITORY" --provider local

  if [ -f "$SERVER_HOME/lore/pid" ]; then
    SERVER_PID="$(tr -d '[:space:]' < "$SERVER_HOME/lore/pid")"
  fi
  if ! [[ "$SERVER_PID" =~ ^[0-9]+$ ]]; then
    echo "Error: NAP started Lore but did not record a valid server PID" >&2
    exit 1
  fi
fi

echo "Running local Lore server integration tests..."

cd "$ROOT_DIR"
cargo test -p nap-cli --test local_lore_suite --features local-e2e -- --test-threads=1 "$@"
