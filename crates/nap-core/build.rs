//! Build script that compiles the lore-server RevisionsService .proto
//! definitions into generated Rust code (via tonic-build + prost-build).
//!
//! The .proto files are symlinked into proto/ from the lore-server source tree.
//! Only the client stubs are generated (no server code).

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = Path::new("proto");
    let revision_proto = proto_dir.join("lore/revision/v1/revision.proto");

    // ── rebuild-on-change tracking ────────────────────────────────────
    println!("cargo:rerun-if-env-changed=PROTOC");
    walk_protos(proto_dir);

    // ── compile the revision proto (transitively compiles model.proto) ─
    //
    // NOTE: `bytes(["."])` must match the lore-server's own prost config so
    // that all `bytes` protobuf fields are generated as `bytes::Bytes`
    // (zero-copy) instead of `Vec<u8>`.  This keeps the wire representation
    // consistent across the ecosystem.
    let mut prost_config = prost_build::Config::new();
    prost_config.bytes(["."]);

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos_with_config(prost_config, &[&revision_proto], &[proto_dir])?;

    Ok(())
}

/// Recursively emit `cargo:rerun-if-changed` for every `.proto` file under
/// `dir` so that Cargo rebuilds when any upstream proto definition changes.
fn walk_protos(dir: &Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_protos(&path);
            } else if path.extension().is_some_and(|e| e == "proto") {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
