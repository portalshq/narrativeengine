#!/usr/bin/env bash
set -euo pipefail

# Generate all documentation from source.
# This script is the canonical way to regenerate documentation.
# It should be run before commits and releases.

echo "Generating documentation..."
cargo run -p nap-docgen

echo "Done."
