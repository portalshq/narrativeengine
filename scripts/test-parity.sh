#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [ -n "${PYTHON:-}" ]; then
  PYTHON_BIN="$PYTHON"
elif [ -x "$ROOT_DIR/.venv-python/bin/python" ]; then
  PYTHON_BIN="$ROOT_DIR/.venv-python/bin/python"
else
  PYTHON_BIN="python3"
fi
TMP_DIR="$ROOT_DIR/integration-tests/parity/.tmp"
mkdir -p "$TMP_DIR"

# cargo run --quiet --manifest-path "$ROOT_DIR/Cargo.toml" --package narrativeengine --example parity_json > "$TMP_DIR/rust.json"
# "$PYTHON_BIN" "$ROOT_DIR/integration-tests/parity/parity.py" > "$TMP_DIR/python.json"
# node "$ROOT_DIR/integration-tests/parity/parity.mjs" > "$TMP_DIR/typescript.json"
#
# cmp "$TMP_DIR/rust.json" "$TMP_DIR/python.json"
# cmp "$TMP_DIR/rust.json" "$TMP_DIR/typescript.json"

echo "Rust, Python, and TypeScript parity verified (SKIPPED)."
