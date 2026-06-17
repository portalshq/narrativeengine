use napi::Error;
use napi_derive::napi;
use std::path::Path;

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
