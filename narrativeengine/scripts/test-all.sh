#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"$ROOT_DIR/scripts/build-all.sh"
PYTHON="${PYTHON:-$ROOT_DIR/.venv-python/bin/python}" "$ROOT_DIR/scripts/test-parity.sh"
