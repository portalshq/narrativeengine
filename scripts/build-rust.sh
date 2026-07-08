#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --exclude portalshq-narrativeengine-py --exclude portalshq-nap-core-py --exclude portalshq-narrativeengine-ts --exclude portalshq-nap-core-ts -- -D warnings
# FFI crates are built by their respective language tooling
cargo build --workspace --exclude portalshq-narrativeengine-py --exclude portalshq-nap-core-py --exclude portalshq-narrativeengine-ts --exclude portalshq-nap-core-ts
cargo test --workspace --exclude portalshq-narrativeengine-py --exclude portalshq-nap-core-py --exclude portalshq-narrativeengine-ts --exclude portalshq-nap-core-ts
