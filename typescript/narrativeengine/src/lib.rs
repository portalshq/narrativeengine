use napi::Error;
use napi_derive::napi;

// TODO: Re-implement these functions for the new protobuf-based architecture
// The old JSON-based functions were removed during the narrativeengine refactor

#[napi(js_name = "createBlockJson")]
pub fn create_block_json(_id: String, _content: String) -> napi::Result<String> {
    Err(Error::from_reason(
        "createBlockJson not yet implemented in new architecture",
    ))
}

#[napi(js_name = "generateCandidateJson")]
pub fn generate_candidate_json(_lore_json: String, _config_json: String) -> napi::Result<String> {
    Err(Error::from_reason(
        "generateCandidateJson not yet implemented in new architecture",
    ))
}

#[napi(js_name = "renderLoreSummaryJson")]
pub fn render_lore_summary_json(_lore_json: String) -> napi::Result<String> {
    Err(Error::from_reason(
        "renderLoreSummaryJson not yet implemented in new architecture",
    ))
}

#[napi(js_name = "schemaBundleJson")]
pub fn schema_bundle_json() -> napi::Result<String> {
    Err(Error::from_reason(
        "schemaBundleJson not yet implemented in new architecture",
    ))
}

#[napi]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
