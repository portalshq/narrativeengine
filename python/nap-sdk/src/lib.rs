#![allow(clippy::useless_conversion, unsafe_op_in_unsafe_fn)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::path::Path;

fn map_error(error: nap_core::NapError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

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

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(parse_uri, module)?)?;
    module.add_function(wrap_pyfunction!(parse_manifest, module)?)?;
    module.add_function(wrap_pyfunction!(resolve, module)?)?;
    module.add_function(wrap_pyfunction!(version, module)?)?;
    Ok(())
}
