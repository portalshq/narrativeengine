#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

: "${CARGO_REGISTRY_TOKEN:?CARGO_REGISTRY_TOKEN is required}"
: "${MATURIN_PYPI_TOKEN:?MATURIN_PYPI_TOKEN is required}"
: "${NPM_TOKEN:?NPM_TOKEN is required}"

"$ROOT_DIR/scripts/test-all.sh"

cargo publish --manifest-path "$ROOT_DIR/crates/core/Cargo.toml"
cargo publish --manifest-path "$ROOT_DIR/crates/codegen/Cargo.toml"

cd "$ROOT_DIR/python"
maturin publish --manifest-path ../crates/python-bindings/Cargo.toml --username __token__ --password "$MATURIN_PYPI_TOKEN"

cd "$ROOT_DIR/typescript"
printf '//registry.npmjs.org/:_authToken=%s\n' "$NPM_TOKEN" > .npmrc
npm publish --access public
rm .npmrc

