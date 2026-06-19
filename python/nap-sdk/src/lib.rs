#![allow(clippy::useless_conversion, unsafe_op_in_unsafe_fn)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::path::Path;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

// ── Tokio Runtime (lazy global) ─────────────────────────────────────
//
// The Python API is synchronous, but the storage engine is async.
// We maintain a single Tokio runtime for the entire module and release
// the Python GIL before blocking on it — essential for preventing
// deadlocks in FastAPI / any multi-threaded Python application.

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        Runtime::new().expect("failed to create Tokio runtime for nap-sdk-py")
    })
}

// ── Error mapping ───────────────────────────────────────────────────

fn map_error(error: nap_core::NapError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn map_storage_error(error: nap_core::storage::StorageError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

// ── Functions ───────────────────────────────────────────────────────

#[pyfunction]
fn parse_uri(uri_str: String) -> PyResult<String> {
    let uri: nap_core::NapUri = uri_str
        .parse()
        .map_err(|e: nap_core::NapError| PyValueError::new_err(e.to_string()))?;
    serde_json::to_string(&uri).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn parse_manifest(yaml_str: String) -> PyResult<String> {
    let manifest: nap_core::Manifest =
        serde_yaml::from_str(&yaml_str).map_err(|e| PyValueError::new_err(e.to_string()))?;
    serde_json::to_string(&manifest).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn resolve(uri_str: String, repo_base_path: String) -> PyResult<String> {
    let resolver = nap_core::Resolver::new(Path::new(&repo_base_path));
    let manifest = resolver
        .resolve(&uri_str, &nap_core::ResolveOptions::default())
        .map_err(map_error)?;
    serde_json::to_string(&manifest).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Ingest raw media bytes into the content-addressed storage engine.
///
/// This function:
/// 1. Releases the Python GIL before entering the async runtime
///    (prevents FastAPI / multi-threaded deadlocks).
/// 2. Delegates all storage logic to the Rust engine — no Python I/O.
/// 3. Returns the `sha256:<hex>` content hash.
///
/// # Arguments
///
/// * `data` — Raw bytes of the media asset (image, audio, mesh, etc.).
/// * `format` — File extension without a leading dot (e.g. `"png"`,
///   `"jpg"`, `"wav"`, `"glb"`).
///
/// # Returns
///
/// The content-addressed hash string `sha256:<hex>`.
#[pyfunction]
fn ingest_media(py: Python<'_>, data: Vec<u8>, format: String) -> PyResult<String> {
    let engine = nap_core::storage::get_engine().map_err(map_storage_error)?;

    // CRITICAL: Release the Python GIL before blocking on Tokio I/O.
    // If we don't, concurrent Python threads (e.g. FastAPI workers) will
    // deadlock trying to acquire the GIL while we hold it in a blocking I/O
    // call.
    let result = py.allow_threads(|| get_runtime().block_on(engine.ingest_media(&data, &format)));

    result.map_err(map_storage_error)
}

// ── Module registration ────────────────────────────────────────────

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(parse_uri, module)?)?;
    module.add_function(wrap_pyfunction!(parse_manifest, module)?)?;
    module.add_function(wrap_pyfunction!(resolve, module)?)?;
    module.add_function(wrap_pyfunction!(version, module)?)?;
    module.add_function(wrap_pyfunction!(ingest_media, module)?)?;
    Ok(())
}
