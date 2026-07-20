use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = Path::new("proto");
    let narrative_proto = proto_dir.join("narrative/v1/narrative.proto");

    let protoc_path = protoc_bin_vendored::protoc_bin_path()
        .expect("failed to locate vendored protoc binary; set PROTOC env var to override");
    unsafe { std::env::set_var("PROTOC", &protoc_path) };

    println!("cargo:rerun-if-env-changed=PROTOC");
    walk_protos(proto_dir);

    tonic_build::configure()
        .build_client(false)
        .build_server(false)
        .compile_protos(&[&narrative_proto], &[proto_dir])?;

    Ok(())
}

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
