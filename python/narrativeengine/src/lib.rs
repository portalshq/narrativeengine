// The unsafe_op_in_unsafe_fn lint fires because PyO3 0.22 does not
// wrap its unsafe internals with `unsafe {}` blocks.  Suppressing is
// correct here — the macro-generated functions are safe at the API
// boundary.  A future PyO3 release will fix this upstream.
#![allow(clippy::useless_conversion, unsafe_op_in_unsafe_fn)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

// TODO: Re-implement these functions for the new protobuf-based architecture
// The old JSON-based functions were removed during the narrativeengine refactor

#[pyfunction]
fn create_block_json(_id: String, _content: String) -> PyResult<String> {
    Err(PyValueError::new_err(
        "create_block_json not yet implemented in new architecture",
    ))
}

#[pyfunction]
fn generate_candidate_json(_lore_json: String, _config_json: String) -> PyResult<String> {
    Err(PyValueError::new_err(
        "generate_candidate_json not yet implemented in new architecture",
    ))
}

#[pyfunction]
fn render_lore_summary_json(_lore_json: String) -> PyResult<String> {
    Err(PyValueError::new_err(
        "render_lore_summary_json not yet implemented in new architecture",
    ))
}

#[pyfunction]
fn schema_bundle_json() -> PyResult<String> {
    Err(PyValueError::new_err(
        "schema_bundle_json not yet implemented in new architecture",
    ))
}

#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(create_block_json, module)?)?;
    module.add_function(wrap_pyfunction!(generate_candidate_json, module)?)?;
    module.add_function(wrap_pyfunction!(render_lore_summary_json, module)?)?;
    module.add_function(wrap_pyfunction!(schema_bundle_json, module)?)?;
    module.add_function(wrap_pyfunction!(version, module)?)?;
    Ok(())
}
