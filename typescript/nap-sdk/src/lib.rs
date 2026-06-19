use napi::Error;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::path::Path;

// ── Synchronous functions ───────────────────────────────────────────

#[napi(js_name = "parseUri")]
pub fn parse_uri(uri_str: String) -> napi::Result<String> {
    let uri: nap_core::NapUri = uri_str
        .parse()
        .map_err(|e: nap_core::NapError| Error::from_reason(e.to_string()))?;
    serde_json::to_string(&uri).map_err(|e| Error::from_reason(e.to_string()))
}

#[napi(js_name = "parseManifest")]
pub fn parse_manifest(yaml_str: String) -> napi::Result<String> {
    let manifest: nap_core::Manifest =
        serde_yaml::from_str(&yaml_str).map_err(|e| Error::from_reason(e.to_string()))?;
    serde_json::to_string(&manifest).map_err(|e| Error::from_reason(e.to_string()))
}

#[napi(js_name = "resolve")]
pub fn resolve(uri_str: String, repo_base_path: String) -> napi::Result<String> {
    let resolver = nap_core::Resolver::new(Path::new(&repo_base_path));
    let manifest = resolver
        .resolve(&uri_str, &nap_core::ResolveOptions::default())
        .map_err(|e| Error::from_reason(e.to_string()))?;
    serde_json::to_string(&manifest).map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ── Async media ingestion ───────────────────────────────────────────

/// Ingest raw media bytes into the content-addressed storage engine.
///
/// This is an async function returning a JavaScript Promise.  The storage
/// backend is determined at runtime by the ``NAP_STORAGE_BACKEND``
/// environment variable.
///
/// ## Memory Safety (NAPI Buffer Ownership)
///
/// The incoming [`Buffer`] is **copied** to an owned `Vec<u8>` **before**
/// crossing the await boundary.  Holding a raw pointer to a V8 `ArrayBuffer`
/// across an await point is unsafe because Node's garbage collector may
/// invalidate or relocate the backing store.  By cloning eagerly we avoid
/// use-after-free and data races.
///
/// # Arguments
///
/// * `data` — Raw bytes of the media asset (image, audio, mesh, etc.),
///   passed as a Node.js `Buffer`.
/// * `format` — File extension without a leading dot (e.g. `"png"`,
///   `"jpg"`, `"wav"`, `"glb"`).
///
/// # Returns
///
/// A Promise resolving to the content-addressed hash `sha256:<hex>`.
#[napi]
pub async fn ingest_media(data: Buffer, format: String) -> napi::Result<String> {
    // CRITICAL (Gotcha 3): Copy the buffer to an owned Vec<u8> before
    // the first await point.  Node's GC can invalidate the underlying
    // ArrayBuffer once we yield control to the Tokio runtime.
    let owned_data = data.to_vec();

    let engine = nap_core::storage::get_engine().map_err(|e| Error::from_reason(e.to_string()))?;

    engine
        .ingest_media(&owned_data, &format)
        .await
        .map_err(|e| Error::from_reason(e.to_string()))
}
