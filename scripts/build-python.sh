#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV_DIR="${VENV_DIR:-$ROOT_DIR/.venv-python}"
PYTHON_BIN="${PYTHON:-python3}"

echo "Using ${PYTHON_BIN} at: $(which "$PYTHON_BIN")"

if [ ! -x "$VENV_DIR/bin/python" ]; then
  "$PYTHON_BIN" -m venv "$VENV_DIR"
fi

PYTHON_BIN="$VENV_DIR/bin/python"

export PYO3_PYTHON="$PYTHON_BIN"

cd "$ROOT_DIR/python"

"$PYTHON_BIN" -m pip install --upgrade pip
"$PYTHON_BIN" -m pip install -e ".[dev]"
"$PYTHON_BIN" -m maturin develop --manifest-path ../crates/narrativeengine-py/Cargo.toml --extras pydantic
"$PYTHON_BIN" -m pytest
"$PYTHON_BIN" -m ruff check --fix .
"$PYTHON_BIN" -m mypy narrativeengine
