#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "=== narrativeengine (TypeScript) ==="
cd "$ROOT_DIR/typescript/narrativeengine"
npm install
npm run build
npm run lint
npm test

echo "=== nap-sdk (TypeScript) ==="
cd "$ROOT_DIR/typescript/nap-sdk"
npm install
npm run build
