// The unsafe_op_in_unsafe_fn lint fires because PyO3 0.22 does not
// wrap its unsafe internals with `unsafe {}` blocks.  Suppressing is
// correct here — the macro-generated functions are safe at the API
// boundary.  A future PyO3 release will fix this upstream.
#![allow(clippy::useless_conversion, unsafe_op_in_unsafe_fn)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use narrativeengine::{
    engine::{LabConfig, NarrativeEngine},
    narrative::v1::{BaseNarrativeBlock, BaseNarrativeLore, BatchGenerationOptions, GenerationParameters},
    provider::InMemoryNarrativeProvider,
};

// ─────────────────────────────────────────────────────────────────────────────
// Basic utility functions (legacy compatibility - simplified versions)
// ─────────────────────────────────────────────────────────────────────────────

#[pyfunction]
fn create_block_json(id: String, content: String) -> PyResult<String> {
    // Return a simple JSON representation for compatibility
    let block_json = format!(
        r#"{{"id":"{}","index":0,"content":"{}","happened_at":0,"is_notable":false}}"#,
        id, content
    );
    Ok(block_json)
}

#[pyfunction]
fn generate_candidate_json(_lore_json: String, _config_json: String) -> PyResult<String> {
    // Return a simple placeholder for compatibility
    Ok(r#"{"score":0.5,"block":{"content":"placeholder"},"lore_atoms":[]}"#.to_string())
}

#[pyfunction]
fn render_lore_summary_json(_lore_json: String) -> PyResult<String> {
    // Return a simple placeholder for compatibility
    Ok("Lore summary placeholder".to_string())
}

#[pyfunction]
fn schema_bundle_json() -> PyResult<String> {
    Ok(r#"{"version":"0.6.0","features":["basic_generation","context_generation"]}"#.to_string())
}

#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ─────────────────────────────────────────────────────────────────────────────
// NarrativeEngine class bindings
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass]
pub struct PyNarrativeEngine {
    engine: NarrativeEngine<BaseNarrativeBlock, BaseNarrativeLore>,
}

#[pymethods]
impl PyNarrativeEngine {
    #[new]
    fn new() -> PyResult<Self> {
        let provider = InMemoryNarrativeProvider::new(vec![], vec![]);
        let engine = NarrativeEngine::new(std::sync::Arc::new(provider));
        
        Ok(PyNarrativeEngine { engine })
    }

    fn generate_context(&self, channel_id: String, query: String) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyValueError::new_err(format!("Failed to create runtime: {}", e)))?;

        let context = rt.block_on(async {
            self.engine.generate_context(&channel_id, &query).await
        });

        Ok(context)
    }

    fn set_lab_config(&mut self, config_json: String) -> PyResult<()> {
        let config: LabConfig = serde_json::from_str(&config_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse config: {}", e)))?;

        self.engine.set_lab_config(config);
        Ok(())
    }

    fn get_lab_config(&self) -> PyResult<String> {
        let config = self.engine.get_lab_config();
        let json = serde_json::to_string(&config)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize config: {}", e)))?;
        Ok(json)
    }

    fn generate_block(
        &self,
        channel_id: String,
        input_query: String,
        parameters_json: String,
    ) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyValueError::new_err(format!("Failed to create runtime: {}", e)))?;

        let parameters: GenerationParameters = serde_json::from_str(&parameters_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse parameters: {}", e)))?;

        let result = rt.block_on(async {
            self.engine.generate_block(&channel_id, &input_query, parameters).await
        });

        match result {
            Ok(envelope) => {
                let json = serde_json::to_string(&envelope)
                    .map_err(|e| PyValueError::new_err(format!("Failed to serialize result: {}", e)))?;
                Ok(json)
            }
            Err(error) => Err(PyValueError::new_err(format!("Generation failed: {}", error.message))),
        }
    }

    fn generate_blocks_sequential(
        &self,
        channel_id: String,
        previous_context: String,
        options_json: String,
    ) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyValueError::new_err(format!("Failed to create runtime: {}", e)))?;

        let options: BatchGenerationOptions = serde_json::from_str(&options_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse options: {}", e)))?;

        let result = rt.block_on(async {
            self.engine
                .generate_blocks_sequential(&channel_id, &previous_context, options)
                .await
        });

        match result {
            Ok(result) => {
                let json = serde_json::to_string(&result)
                    .map_err(|e| PyValueError::new_err(format!("Failed to serialize result: {}", e)))?;
                Ok(json)
            }
            Err(error) => Err(PyValueError::new_err(format!("Batch generation failed: {}", error.message))),
        }
    }

    fn generate_blocks_parallel(
        &self,
        channel_id: String,
        branch_contexts: Vec<String>,
        options_json: String,
    ) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyValueError::new_err(format!("Failed to create runtime: {}", e)))?;

        let options: BatchGenerationOptions = serde_json::from_str(&options_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse options: {}", e)))?;

        let result = rt.block_on(async {
            self.engine
                .generate_blocks_parallel(&channel_id, &branch_contexts, options)
                .await
        });

        match result {
            Ok(result) => {
                let json = serde_json::to_string(&result)
                    .map_err(|e| PyValueError::new_err(format!("Failed to serialize result: {}", e)))?;
                Ok(json)
            }
            Err(error) => Err(PyValueError::new_err(format!("Parallel generation failed: {}", error.message))),
        }
    }
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(create_block_json, module)?)?;
    module.add_function(wrap_pyfunction!(generate_candidate_json, module)?)?;
    module.add_function(wrap_pyfunction!(render_lore_summary_json, module)?)?;
    module.add_function(wrap_pyfunction!(schema_bundle_json, module)?)?;
    module.add_function(wrap_pyfunction!(version, module)?)?;
    module.add_class::<PyNarrativeEngine>()?;
    Ok(())
}
