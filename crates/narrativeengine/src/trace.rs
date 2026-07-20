//! Trace / observability utilities.
//!
//! Mirrors `trace.ts`: `TraceObject` and `logger_narrative_trace`.
//! Writing is gated behind `NARRATIVE_VERBOSE=true` or `RUST_ENV=development`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::engine::LabConfig;

/// Represents one full context-generation trace, organized by pipeline phase.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TraceObject {
    pub timestamp: String,
    pub channel_id: String,
    pub input_query: String,
    pub provider_type: Option<String>,
    pub lab_config: Option<LabConfig>,
    pub phases: TracePhases,
    pub finalized_prompt: Option<String>,
    pub discarded_candidates: Option<Vec<serde_json::Value>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TracePhases {
    pub harvest: Option<serde_json::Value>,
    pub fusion: Option<serde_json::Value>,
    pub saliency: Option<serde_json::Value>,
    pub timeline: Option<serde_json::Value>,
    pub prose: Option<serde_json::Value>,
}

/// Appends `trace` as a JSON-L line to `.traces/narrative_ledger.jsonl`.
///
/// Tracing is only active when `NARRATIVE_VERBOSE=true` or `RUST_ENV=development`.
pub fn logger_narrative_trace(trace: &TraceObject) {
    let verbose = std::env::var("NARRATIVE_VERBOSE")
        .map(|v| v == "true")
        .unwrap_or(false);
    let dev = std::env::var("RUST_ENV")
        .map(|v| v == "development")
        .unwrap_or(false);

    if !verbose && !dev {
        return;
    }

    let trace_dir = PathBuf::from(".traces");
    if let Err(e) = fs::create_dir_all(&trace_dir) {
        eprintln!("[Trace] Failed to create .traces directory: {e}");
        return;
    }

    let filepath = trace_dir.join("narrative_ledger.jsonl");
    match serde_json::to_string(trace) {
        Ok(json) => {
            if let Ok(mut file) = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&filepath)
            {
                let _ = writeln!(file, "{json}");
            }
        }
        Err(e) => eprintln!("[Trace] Failed to serialize trace: {e}"),
    }
}
