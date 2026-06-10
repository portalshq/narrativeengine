#![allow(clippy::useless_conversion)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn map_error(error: narrativeengine_core::NarrativeError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

#[pyfunction]
fn create_block_json(id: String, content: String) -> PyResult<String> {
    narrativeengine_core::create_block_json(id, content).map_err(map_error)
}

#[pyfunction]
fn generate_candidate_json(lore_json: String, config_json: String) -> PyResult<String> {
    narrativeengine_core::generate_candidate_json(&lore_json, &config_json).map_err(map_error)
}

#[pyfunction]
fn render_lore_summary_json(lore_json: String) -> PyResult<String> {
    narrativeengine_core::render_lore_summary_json(&lore_json).map_err(map_error)
}

#[pyfunction]
fn schema_bundle_json() -> PyResult<String> {
    narrativeengine_core::schema_bundle_json().map_err(map_error)
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
