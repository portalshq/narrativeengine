#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"$ROOT_DIR/scripts/generate-types.sh"
"$ROOT_DIR/scripts/build-rust.sh"
"$ROOT_DIR/scripts/build-python.sh"
"$ROOT_DIR/scripts/build-typescript.sh"
