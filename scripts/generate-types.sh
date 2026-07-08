#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cargo run --quiet --manifest-path "$ROOT_DIR/Cargo.toml" --package portalshq-narrativeengine-codegen -- --language python --out "$ROOT_DIR/python/narrativeengine/narrativeengine/models.py"
cargo run --quiet --manifest-path "$ROOT_DIR/Cargo.toml" --package portalshq-narrativeengine-codegen -- --language typescript --out "$ROOT_DIR/typescript/narrativeengine/src/models.ts"
cargo run --quiet --manifest-path "$ROOT_DIR/Cargo.toml" --package portalshq-narrativeengine-codegen -- --language go --out "$ROOT_DIR/generated/go/models.go"
cargo run --quiet --manifest-path "$ROOT_DIR/Cargo.toml" --package portalshq-narrativeengine-codegen -- --language java --out "$ROOT_DIR/generated/java/NarrativeModels.java"
cargo run --quiet --manifest-path "$ROOT_DIR/Cargo.toml" --package portalshq-narrativeengine-codegen -- --language csharp --out "$ROOT_DIR/generated/csharp/NarrativeModels.cs"
cargo run --quiet --manifest-path "$ROOT_DIR/Cargo.toml" --package portalshq-narrativeengine-codegen -- --language swift --out "$ROOT_DIR/generated/swift/NarrativeModels.swift"
cargo run --quiet --manifest-path "$ROOT_DIR/Cargo.toml" --package portalshq-narrativeengine-codegen -- --language kotlin --out "$ROOT_DIR/generated/kotlin/NarrativeModels.kt"
