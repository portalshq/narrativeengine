#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --exclude narrativeengine-py -- -D warnings
# narrativeengine-py is built by maturin in build-python.sh (needs venv for pyo3)
cargo build --workspace --exclude narrativeengine-py
cargo test --workspace --exclude narrativeengine-py
