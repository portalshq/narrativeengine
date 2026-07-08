#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --exclude narrativeengine-py --exclude nap-sdk-py --exclude narrativeengine-ts --exclude nap-sdk-ts -- -D warnings
# FFI crates are built by their respective language tooling
cargo build --workspace --exclude narrativeengine-py --exclude nap-sdk-py --exclude narrativeengine-ts --exclude nap-sdk-ts
cargo test --workspace --exclude narrativeengine-py --exclude nap-sdk-py --exclude narrativeengine-ts --exclude nap-sdk-ts
