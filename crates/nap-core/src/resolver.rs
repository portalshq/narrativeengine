//! NAP Resolver — URI → Manifest, with query and version selectors.
//!
//! The resolver is the primary interface for reading NAP resources.
//! It handles:
//! - Full manifest resolution: `nap://toystory/character/woody`
//! - Fragment queries: `nap://toystory/character/woody#references.appears_in`
//! - Version selectors: branch, commit
//! - Subtree extraction for efficient AI/application access
//!
//! Version and branch are NEVER in the URI. They are orthogonal selectors:
//! ```text
//! URI + Reference + Revision Selector
//! ```

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::error::NapError;
use crate::grpc_client::{LoreGrpcClient, block_on_grpc};
use crate::manifest::Manifest;
use crate::query::ManifestQuery;
use crate::repository::Repository;
use crate::uri::NapUri;
use crate::vcs::VcsBackend;
use crate::vcs_lore::LoreBackend;

/// Where repository-backed reads are performed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResolveSource {
    /// Use the configured Lore server, falling back to local Lore when no
    /// provider configuration exists.
    #[default]
    Auto,
    /// Require the configured (or default local) Lore server.
    Remote,
    /// Read a checked-out NAP working tree under `base_path`.
    Local,
}

/// Resolver configuration — set at construction time.
///
/// Controls how the resolver resolves URIs when no explicit branch or
/// commit is provided by the caller.
#[derive(Debug, Clone, Default)]
pub struct ResolveConfig {
    /// Branch to resolve when neither `branch` nor `commit` is specified
    /// in [`ResolveOptions`].  If `None`, resolves without a branch or
    /// commit — this will trigger a [`NapError::NoDefaultBranch`] error
    /// for any resolve call that omits both `branch` and `commit`.
    pub default_branch: Option<String>,
}

/// Options for resolving a NAP URI. All are optional — omitting all
/// causes the resolver to use its [`ResolveConfig::default_branch`] (if
/// configured) or fail with [`NapError::NoDefaultBranch`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolveOptions {
    /// Select server-backed or explicit working-tree resolution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ResolveSource>,
    /// Resolve at a specific branch. e.g., `"canon"`.
    /// Takes precedence over [`ResolveConfig::default_branch`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Resolve at a specific commit hash (BLAKE3). e.g.,
    /// `"af1349b9f5f9a1a6a0404deb36d020949b834f2a42e37e5f8d2e4ba2765f1a2f"`.
    /// Takes precedence over `branch` and [`ResolveConfig::default_branch`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,

    /// Subtree query path (overrides URI fragment). e.g., `"appearances.audienceVotes"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Recursively resolve nested URIs. When true, the resolver will follow
    /// all nap:// URIs found in the resolved manifest and resolve them as well.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recursive: Option<bool>,

    /// Maximum recursion depth for recursive resolution. Defaults to 10 to prevent
    /// infinite loops. Set to None for unlimited depth (not recommended).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,

    /// Include per-file provenance metadata for the manifest and direct representations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<bool>,

    /// Hydrate known readable provenance artifacts such as prompts and run records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_blobs: Option<bool>,
}

/// Options for creating a bearer URL for a committed representation.
#[derive(Clone, Default)]
pub struct PresignOptions {
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub ttl_seconds: Option<u64>,
    pub lore_http_url: Option<String>,
    pub bearer_token: Option<String>,
}

impl fmt::Debug for PresignOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PresignOptions")
            .field("branch", &self.branch)
            .field("commit", &self.commit)
            .field("ttl_seconds", &self.ttl_seconds)
            .field("lore_http_url", &self.lore_http_url)
            .field(
                "bearer_token",
                &self.bearer_token.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

/// A time-limited public URL for one immutable representation.
#[derive(Clone, Serialize, Deserialize)]
pub struct PresignedRepresentation {
    pub url: String,
    pub expires_at: u64,
    pub revision: String,
    pub repository_id: String,
    pub address: String,
    pub representation: String,
    pub format: String,
}

impl fmt::Debug for PresignedRepresentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PresignedRepresentation")
            .field("url", &"<redacted>")
            .field("expires_at", &self.expires_at)
            .field("revision", &self.revision)
            .field("repository_id", &self.repository_id)
            .field("address", &self.address)
            .field("representation", &self.representation)
            .field("format", &self.format)
            .finish()
    }
}

#[derive(Serialize)]
struct LorePresignRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl_seconds: Option<u64>,
}

#[derive(Deserialize)]
struct LorePresignResponse {
    url_suffix: String,
    expires_at: u64,
}

fn presign_http_client() -> Result<&'static reqwest::Client, NapError> {
    static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| e.to_string())
        })
        .as_ref()
        .map_err(|e| NapError::Other(format!("failed to initialize presign HTTP client: {e}")))
}

async fn read_bounded_response(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<(reqwest::StatusCode, Vec<u8>), NapError> {
    let status = response.status();
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(NapError::Other(format!(
            "Lore presign response exceeded {limit} bytes"
        )));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| NapError::Other(format!("failed to read Lore presign response: {e}")))?
    {
        if body.len().saturating_add(chunk.len()) > limit {
            return Err(NapError::Other(format!(
                "Lore presign response exceeded {limit} bytes"
            )));
        }
        body.extend_from_slice(&chunk);
    }
    Ok((status, body))
}

fn validate_presigned_url(
    base_url: &reqwest::Url,
    suffix: &str,
    expected_path: &str,
) -> Result<reqwest::Url, NapError> {
    if suffix.starts_with("//") || !suffix.starts_with('/') {
        return Err(NapError::Other(
            "Lore returned an unexpected presigned URL path".to_string(),
        ));
    }
    let url = base_url
        .join(suffix)
        .map_err(|e| NapError::Other(format!("invalid Lore presigned URL: {e}")))?;
    let query: Vec<_> = url.query_pairs().collect();
    if url.origin() != base_url.origin()
        || url.path() != expected_path
        || query.len() != 1
        || query[0].0 != "token"
        || query[0].1.is_empty()
    {
        return Err(NapError::Other(
            "Lore returned a cross-origin or malformed presigned URL".to_string(),
        ));
    }
    Ok(url)
}

fn format_lore_repository_id(bytes: &[u8]) -> String {
    if bytes.len() == 16 {
        let value = hex::encode(bytes);
        format!(
            "{}-{}-{}-{}-{}",
            &value[..8],
            &value[8..12],
            &value[12..16],
            &value[16..20],
            &value[20..]
        )
    } else {
        hex::encode(bytes)
    }
}

impl ResolveOptions {
    /// Returns the query path (from options or URI fragment).
    fn query_path(&self, uri: &NapUri) -> Option<String> {
        self.path.clone().or_else(|| uri.fragment.clone())
    }
}

/// The result of resolving a NAP URI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResolveResult {
    /// Full manifest (no query applied).
    Full(Box<Manifest>),
    /// Full manifest with Lore-backed per-file provenance envelope.
    Provenance(Box<ResolveEnvelope>),
    /// Subtree result from a query.
    Subtree(serde_json::Value),
}

/// Envelope returned when `ResolveOptions::provenance` is enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveEnvelope {
    pub manifest: Box<Manifest>,
    pub provenance: ResolveProvenanceEnvelope,
}

/// Per-resolution provenance metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveProvenanceEnvelope {
    pub revision: String,
    pub files: Vec<ResolveProvenanceFile>,
}

/// Provenance for one file participating in an entity resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveProvenanceFile {
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    pub provenance: serde_json::Value,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub blobs: BTreeMap<String, HydratedProvenanceBlob>,
}

/// Hydrated readable provenance artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydratedProvenanceBlob {
    pub address: String,
    pub content: String,
    pub truncated: bool,
    pub original_bytes: usize,
    pub included_bytes: usize,
}

const MAX_CONDENSED_METADATA_VALUE_BYTES: usize = 256;
const MAX_HYDRATED_BLOB_BYTES: usize = 12_000;

/// The NAP resolver — resolves URIs to manifests or subtrees.
pub struct Resolver {
    /// Base directory containing repository repositories.
    base_path: PathBuf,
    /// VCS backend factory (creates backend per-repo).
    vcs_factory: fn() -> Box<dyn VcsBackend>,
    /// Whether a version-control backend is configured. When `false`,
    /// repositories are opened in unversioned mode and resolution reads the
    /// current filesystem state (no branch/commit selectors available).
    use_vcs: bool,
    /// Resolution configuration (default branch, etc.).
    config: ResolveConfig,
    source: ResolveSource,
}

impl Resolver {
    /// Create a resolver that looks for repository repos under `base_path`.
    ///
    /// Uses [`LoreBackend::from_env()`] by default **when a version-control
    /// backend is configured** for `base_path` (i.e. a valid `provider.toml`
    /// exists). Otherwise repositories are opened in unversioned mode. For
    /// testing, use [`Resolver::with_vcs_factory()`] with a mock backend.
    ///
    /// Uses [`ResolveConfig::default()`] — meaning `default_branch` is
    /// `None`. In versioned mode any resolve that omits both `branch` and
    /// `commit` will fail with [`NapError::NoDefaultBranch`]; in unversioned
    /// mode such a resolve reads the current filesystem state.
    ///
    /// # Example layout
    /// ```text
    /// base_path/
    /// ├── toystory/    ← repository repo
    /// ├── toystory/    ← repository repo
    /// └── marvel/      ← repository repo
    /// ```
    pub fn new(base_path: &Path) -> Self {
        let source = match std::env::var("NAP_RESOLVE_SOURCE").ok().as_deref() {
            Some("local") => ResolveSource::Local,
            Some("remote") => ResolveSource::Remote,
            _ => ResolveSource::Auto,
        };
        Self {
            base_path: base_path.to_path_buf(),
            vcs_factory: || Box::new(LoreBackend::from_env()),
            // Even without provider.toml, NAP's established local Lore server
            // is the default server for explicit working-tree reads.
            use_vcs: true,
            config: ResolveConfig::default(),
            source,
        }
    }

    /// Create a resolver with a custom VCS backend factory and config.
    ///
    /// Repositories are always opened in versioned mode.
    pub fn with_vcs_factory(
        base_path: &Path,
        factory: fn() -> Box<dyn VcsBackend>,
        config: ResolveConfig,
    ) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
            vcs_factory: factory,
            use_vcs: true,
            config,
            source: ResolveSource::Local,
        }
    }

    /// Override the resolver's default source. Primarily useful to callers
    /// that need an explicit local working-tree read.
    pub fn with_source(mut self, source: ResolveSource) -> Self {
        self.source = source;
        self
    }

    fn effective_source(&self, options: &ResolveOptions) -> ResolveSource {
        options.source.unwrap_or(self.source)
    }

    fn remote_client(&self) -> Result<LoreGrpcClient, NapError> {
        let server = LoreBackend::configured_server_url(&self.base_path);
        let endpoint = server
            .strip_prefix("lore://")
            .map(|rest| format!("http://{rest}"))
            .or_else(|| {
                server
                    .strip_prefix("grpc://")
                    .map(|rest| format!("http://{rest}"))
            })
            .or_else(|| {
                server
                    .strip_prefix("lores://")
                    .map(|rest| format!("https://{rest}"))
            })
            .or_else(|| {
                server
                    .strip_prefix("grpcs://")
                    .map(|rest| format!("https://{rest}"))
            })
            .unwrap_or(server);
        let token = std::env::var("NAP_LORE_GRPC_TOKEN")
            .ok()
            .or_else(|| std::env::var("NAP_REMOTE_AUTH_TOKEN").ok());

        // `Endpoint::connect_lazy()` creates Tokio I/O state. Construct it
        // inside the bridge rather than on the synchronous CLI thread; doing
        // so outside a Tokio reactor panics in hyper-util before the first
        // RPC (for example, `nap list <repository>`).
        block_on_grpc(async move {
            let mut builder = LoreGrpcClient::builder().endpoint(endpoint);
            if let Some(token) = token {
                builder = builder.token(token);
            }
            builder.build()
        })
    }

    fn remote_manifest(
        &self,
        uri: &NapUri,
        options: &ResolveOptions,
    ) -> Result<Manifest, NapError> {
        if options.provenance.unwrap_or(false) || options.include_blobs.unwrap_or(false) {
            return Err(NapError::Other(
                "remote provenance is not supported yet".to_string(),
            ));
        }
        let client = self.remote_client()?;
        let repository = uri.repository.clone();
        let path = uri.manifest_path();
        let branch = options.branch.clone();
        let commit = options.commit.clone();
        let bytes = block_on_grpc(async move {
            let repo = client.get_repository_by_name(&repository).await?;
            let scoped = client.for_repository_id(repo.id.clone());
            if let Some(commit) = commit {
                let signature = hex::decode(commit.trim_start_matches("blake3:")).map_err(|e| {
                    NapError::InvalidUri {
                        uri: commit,
                        reason: format!("invalid revision signature: {e}"),
                    }
                })?;
                scoped
                    .read_file_at_signature(signature, path)
                    .await
                    .map(|(bytes, _)| bytes)
            } else {
                let branch_id = match branch {
                    Some(name) => scoped.get_branch_by_name(&name).await?.id.to_vec(),
                    None => repo.default_branch_id.to_vec(),
                };
                scoped
                    .read_file_at_revision(
                        crate::grpc_client::proto_gen::lore::model::v1::RevisionIdentifier {
                            branch_id: branch_id.into(),
                            number: 0,
                        },
                        path,
                    )
                    .await
                    .map(|(bytes, _)| bytes)
            }
        })?;
        serde_yaml::from_slice(&bytes).map_err(|source| NapError::ManifestParseError {
            path: uri.manifest_path(),
            source,
        })
    }

    fn remote_representation_address(
        &self,
        uri: &NapUri,
        options: &PresignOptions,
        path: String,
    ) -> Result<
        (
            Vec<u8>,
            crate::grpc_client::proto_gen::lore::model::v1::Address,
            String,
        ),
        NapError,
    > {
        let client = self.remote_client()?;
        let repository_name = uri.repository.clone();
        let branch = options.branch.clone();
        let commit = options.commit.clone();
        block_on_grpc(async move {
            let repo = client.get_repository_by_name(&repository_name).await?;
            let scoped = client.for_repository_id(repo.id.clone());
            let identifier = if let Some(commit) = commit {
                let signature = hex::decode(commit.trim_start_matches("blake3:"))
                    .map_err(|e| NapError::Other(format!("invalid revision signature: {e}")))?;
                scoped
                    .revision_info_at_signature(signature)
                    .await?
                    .identifier
                    .ok_or_else(|| {
                        NapError::GrpcError("RevisionInfo returned no identifier".to_string())
                    })?
            } else {
                let branch_id = match branch {
                    Some(name) => scoped.get_branch_by_name(&name).await?.id,
                    None => repo.default_branch_id.clone(),
                };
                crate::grpc_client::proto_gen::lore::model::v1::RevisionIdentifier {
                    branch_id,
                    number: 0,
                }
            };
            let (address, signature) = scoped.file_address_at_revision(identifier, path).await?;
            Ok((repo.id.to_vec(), address, hex::encode(signature)))
        })
    }

    async fn presign_remote_representation(
        &self,
        uri: &NapUri,
        representation_name: &str,
        options: &PresignOptions,
    ) -> Result<PresignedRepresentation, NapError> {
        let manifest = self.remote_manifest(
            uri,
            &ResolveOptions {
                branch: options.branch.clone(),
                commit: options.commit.clone(),
                source: Some(ResolveSource::Remote),
                ..Default::default()
            },
        )?;
        let representation = manifest
            .representations
            .get(representation_name)
            .ok_or_else(|| {
                NapError::Other(format!(
                    "representation '{representation_name}' does not exist on {}",
                    manifest.id
                ))
            })?;
        let representation_uri = representation.uri.as_deref().ok_or_else(|| {
            NapError::Other(format!(
                "representation '{representation_name}' has no repository-relative URI"
            ))
        })?;
        let file_path = Self::resolve_representation_path(uri, representation_uri)?
            .ok_or_else(|| NapError::Other(format!("representation '{representation_name}' is external; only direct repository files can be presigned")))?;
        let (repository_id_bytes, address, revision) =
            self.remote_representation_address(uri, options, file_path)?;
        // `Representation::hash` is BLAKE3 over the reassembled file bytes,
        // whereas a Lore address hashes its storage fragment tree for large
        // files. They are separate identifiers and must not be compared.
        let hash = hex::encode(&address.hash);
        let repository_id = format_lore_repository_id(&repository_id_bytes);
        let address_string = format!("{}-{}", hash, hex::encode(&address.context));
        let http_url = options
            .lore_http_url
            .clone()
            .or_else(|| std::env::var("NAP_LORE_HTTP_URL").ok())
            .unwrap_or(
                crate::provider::http::configured_origin(
                    &self.base_path,
                    &LoreBackend::configured_server_url(&self.base_path),
                )
                .map_err(|e| NapError::Other(e.to_string()))?,
            );
        let base_url = crate::provider::http::validate_origin(&http_url)
            .map_err(|e| NapError::Other(e.to_string()))?;
        let endpoint = base_url
            .join(&format!(
                "/v1/repository/{repository_id}/content/{address_string}/presign"
            ))
            .map_err(|e| NapError::Other(format!("failed to construct Lore presign URL: {e}")))?;
        let token = options
            .bearer_token
            .clone()
            .or_else(|| std::env::var("NAP_LORE_HTTP_TOKEN").ok())
            .or_else(|| std::env::var("NAP_LORE_GRPC_TOKEN").ok());
        let mut request = presign_http_client()?
            .post(endpoint)
            .json(&LorePresignRequest {
                ttl_seconds: options.ttl_seconds,
            });
        if let Some(token) = token.filter(|token| !token.is_empty()) {
            request = request.bearer_auth(token);
        }
        let (status, body) = read_bounded_response(
            request
                .send()
                .await
                .map_err(|e| NapError::Other(format!("Lore presign request failed: {e}")))?,
            64 * 1024,
        )
        .await?;
        if !status.is_success() {
            return Err(NapError::Other(format!(
                "Lore presign failed with HTTP {status}: {}",
                String::from_utf8_lossy(&body).trim()
            )));
        }
        let response: LorePresignResponse = serde_json::from_slice(&body)
            .map_err(|e| NapError::Other(format!("invalid Lore presign response: {e}")))?;
        let expected_path = format!("/v1/presigned/{repository_id}/{address_string}");
        let url = validate_presigned_url(&base_url, &response.url_suffix, &expected_path)?;
        Ok(PresignedRepresentation {
            url: url.to_string(),
            expires_at: response.expires_at,
            revision,
            repository_id,
            address: address_string,
            representation: representation_name.to_string(),
            format: representation.format.clone(),
        })
    }

    /// Open the repository for a given repository and read its resolve config.
    fn open_repo(&self, repository: &str) -> Result<(Repository, ResolveConfig), NapError> {
        let repo_path = self.base_path.join(repository);
        let vcs = if self.use_vcs {
            Some((self.vcs_factory)())
        } else {
            None
        };
        let repo = Repository::open_optional(&repo_path, vcs)?;
        let repo_config = repo.read_resolve_config();
        Ok((repo, repo_config))
    }

    /// Resolve a NAP URI string with options.
    ///
    /// # Examples
    /// ```text
    /// // Full manifest
    /// resolver.resolve("nap://toystory/character/woody", &Default::default())
    ///
    /// // Without scheme (auto-normalized)
    /// resolver.resolve("toystory/character/woody", &Default::default())
    ///
    /// // With branch
    /// resolver.resolve("nap://toystory/character/woody", &ResolveOptions {
    ///     branch: Some("canon".to_string()),
    ///     ..Default::default()
    /// })
    ///
    /// // With fragment query (via URI)
    /// resolver.resolve("nap://toystory/character/woody#references.appears_in", &Default::default())
    /// ```
    pub fn resolve(
        &self,
        uri_str: &str,
        options: &ResolveOptions,
    ) -> Result<ResolveResult, NapError> {
        // ── Normalization: Prepend nap:// if missing ─────────────────────
        let normalized_uri_str = if uri_str.starts_with("nap://") {
            uri_str.to_string()
        } else {
            format!("nap://{}", uri_str.trim_start_matches('/'))
        };

        debug!(
            original_uri = %uri_str,
            normalized_uri = %normalized_uri_str,
            "normalized NAP URI"
        );

        let uri: NapUri = normalized_uri_str.parse()?;
        self.resolve_uri(&uri, options)
    }

    /// Create a time-limited public URL for a direct, committed representation.
    ///
    /// `uri_str` identifies the entity (for example,
    /// `25th-chapter/character/nathan-gunn`, with an optional `nap://` prefix).
    /// `representation_name` is its manifest representation key, such as `item`.
    /// The representation URI is relative to the entity's asset directory.
    ///
    /// The returned URL is a bearer capability. Callers must not log it or
    /// persist it beyond `expires_at`.
    pub async fn presign_representation(
        &self,
        uri_str: &str,
        representation_name: &str,
        options: &PresignOptions,
    ) -> Result<PresignedRepresentation, NapError> {
        if options.branch.is_some() && options.commit.is_some() {
            return Err(NapError::Other(
                "presign accepts either branch or commit, not both".to_string(),
            ));
        }

        let normalized = if uri_str.starts_with("nap://") {
            uri_str.to_string()
        } else {
            format!("nap://{}", uri_str.trim_start_matches('/'))
        };
        let uri: NapUri = normalized.parse()?;
        if uri.fragment.is_some() {
            return Err(NapError::InvalidUri {
                uri: uri_str.to_string(),
                reason: "fragments are not supported when presigning a representation".to_string(),
            });
        }

        if self.source != ResolveSource::Local {
            return self
                .presign_remote_representation(&uri, representation_name, options)
                .await;
        }

        let (repo, repo_config) = self.open_repo(&uri.repository)?;
        let vcs = repo.vcs().ok_or_else(|| NapError::BackendNotConfigured {
            operation: "presign a representation".to_string(),
        })?;
        let revision = match (&options.commit, &options.branch) {
            (Some(commit), None) => commit.clone(),
            (None, Some(branch)) => vcs.resolve_branch_head(&repo.root, branch)?,
            (None, None) => {
                let branch = repo_config
                    .default_branch
                    .as_ref()
                    .or(self.config.default_branch.as_ref())
                    .ok_or(NapError::NoDefaultBranch)?;
                vcs.resolve_branch_head(&repo.root, branch)?
            }
            (Some(_), Some(_)) => unreachable!("validated above"),
        };

        let manifest = repo.read_manifest_at_ref(&uri.entity_type, &uri.entity_id, &revision)?;
        let representation = manifest
            .representations
            .get(representation_name)
            .ok_or_else(|| {
                NapError::Other(format!(
                    "representation '{representation_name}' does not exist on {}",
                    manifest.id
                ))
            })?;
        let representation_uri = representation.uri.as_deref().ok_or_else(|| {
            NapError::Other(format!(
                "representation '{representation_name}' has no repository-relative URI"
            ))
        })?;
        let file_path = Self::resolve_representation_path(&uri, representation_uri)?
            .ok_or_else(|| {
                NapError::Other(format!(
                    "representation '{representation_name}' is external; only direct repository files can be presigned"
                ))
            })?;

        let repository = vcs.repository_descriptor(&repo.root)?;
        let content = vcs.file_content_address_at_ref(&repo.root, &file_path, &revision)?;
        // `Representation::hash` is BLAKE3 over the reassembled file bytes,
        // whereas a Lore address hashes its storage fragment tree for large
        // files. They are separate identifiers and must not be compared.
        let address = content.as_lore_address();

        let configured_http_url = options
            .lore_http_url
            .clone()
            .or_else(|| std::env::var("NAP_LORE_HTTP_URL").ok());
        let http_url = match configured_http_url {
            Some(url) => url,
            None => {
                crate::provider::http::configured_origin(&self.base_path, &repository.remote_url)
                    .map_err(|e| NapError::Other(e.to_string()))?
            }
        };
        let base_url = crate::provider::http::validate_origin(&http_url)
            .map_err(|e| NapError::Other(e.to_string()))?;

        let endpoint_path = format!(
            "/v1/repository/{}/content/{}/presign",
            repository.id, address
        );
        let endpoint = base_url
            .join(&endpoint_path)
            .map_err(|e| NapError::Other(format!("failed to construct Lore presign URL: {e}")))?;
        let bearer_token = options
            .bearer_token
            .clone()
            .or_else(|| std::env::var("NAP_LORE_HTTP_TOKEN").ok())
            .or_else(|| std::env::var("NAP_LORE_GRPC_TOKEN").ok());
        let mut request = presign_http_client()?
            .post(endpoint)
            .json(&LorePresignRequest {
                ttl_seconds: options.ttl_seconds,
            });
        let explicit_token = bearer_token.as_ref().is_some_and(|token| !token.is_empty());
        if let Some(token) = bearer_token.filter(|token| !token.is_empty()) {
            let mut value = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|_| {
                    NapError::Other("bearer token is not a valid HTTP header value".to_string())
                })?;
            value.set_sensitive(true);
            request = request.header(AUTHORIZATION, value);
        }

        let retry = request.try_clone();
        let mut response = request
            .send()
            .await
            .map_err(|e| NapError::Other(format!("Lore presign request failed: {e}")))?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED && !explicit_token {
            let loopback = base_url.host_str().is_some_and(|host| {
                host == "localhost"
                    || host
                        .trim_matches(['[', ']'])
                        .parse::<std::net::IpAddr>()
                        .is_ok_and(|ip| ip.is_loopback())
            });
            if base_url.scheme() != "https" && !loopback {
                return Err(NapError::Other(
                    "automatic Lore authentication requires HTTPS for remote HTTP endpoints".into(),
                ));
            }
            if let Some(token) = vcs.http_bearer_token(&repo.root, &repository.id, &http_url)? {
                let mut value = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                    .map_err(|_| NapError::Other("invalid Lore bearer token".into()))?;
                value.set_sensitive(true);
                response = retry
                    .ok_or_else(|| NapError::Other("cannot retry Lore presign request".into()))?
                    .header(AUTHORIZATION, value)
                    .send()
                    .await
                    .map_err(|e| NapError::Other(format!("Lore presign request failed: {e}")))?;
            }
        }
        let (status, body) = read_bounded_response(response, 64 * 1024).await?;
        if !status.is_success() {
            let detail = String::from_utf8_lossy(&body);
            let message = match status {
                reqwest::StatusCode::UNAUTHORIZED => {
                    "Lore requires an authenticated repository identity; run nap auth login and retry".to_string()
                }
                reqwest::StatusCode::FORBIDDEN => {
                    "Lore denied permission to presign this repository representation".to_string()
                }
                reqwest::StatusCode::NOT_FOUND if detail.contains("not enabled") => {
                    "Lore presigned URLs are disabled; configure server.http.presigned_url_hmac_key and restart Lore".to_string()
                }
                reqwest::StatusCode::NOT_FOUND => {
                    "representation content is not available in the Lore remote; push the pinned revision before presigning".to_string()
                }
                _ => format!("Lore presign failed with HTTP {status}: {}", detail.trim()),
            };
            return Err(NapError::Other(message));
        }
        let response: LorePresignResponse = serde_json::from_slice(&body)
            .map_err(|e| NapError::Other(format!("invalid Lore presign response: {e}")))?;
        let expected_redeem_path = format!("/v1/presigned/{}/{}", repository.id, address);
        let url = validate_presigned_url(&base_url, &response.url_suffix, &expected_redeem_path)?;

        info!(
            repository_id = %repository.id,
            revision = %revision,
            representation = %representation_name,
            expires_at = response.expires_at,
            "created Lore presigned representation URL"
        );
        Ok(PresignedRepresentation {
            url: url.to_string(),
            expires_at: response.expires_at,
            revision,
            repository_id: repository.id,
            address,
            representation: representation_name.to_string(),
            format: representation.format.clone(),
        })
    }

    /// Resolve a parsed NAP URI with options.
    pub fn resolve_uri(
        &self,
        uri: &NapUri,
        options: &ResolveOptions,
    ) -> Result<ResolveResult, NapError> {
        debug!(
            uri = %uri,
            options = ?options,
            "resolving NAP URI"
        );

        let wants_provenance =
            options.provenance.unwrap_or(false) || options.include_blobs.unwrap_or(false);

        // Handle recursive resolution. Provenance is intentionally scoped to the
        // requested manifest and its direct representations, not related entities.
        if options.recursive.unwrap_or(false) && !wants_provenance {
            return self.resolve_uri_recursive(
                uri,
                options,
                0,
                &mut std::collections::HashSet::new(),
            );
        }

        self.resolve_uri_single(uri, options)
    }

    /// Resolve a single URI without recursion.
    fn resolve_uri_single(
        &self,
        uri: &NapUri,
        options: &ResolveOptions,
    ) -> Result<ResolveResult, NapError> {
        if self.effective_source(options) != ResolveSource::Local {
            let manifest = self.remote_manifest(uri, options)?;
            return match options.query_path(uri) {
                Some(path) => {
                    let value = ManifestQuery::query(&manifest.to_value()?, &path, &manifest.id)?;
                    Ok(ResolveResult::Subtree(
                        serde_json::to_value(value).map_err(|e| NapError::Other(e.to_string()))?,
                    ))
                }
                None => Ok(ResolveResult::Full(Box::new(manifest))),
            };
        }
        let (repo, repo_config) = self.open_repo(&uri.repository)?;
        let query_path = options.query_path(uri);

        // ── 4-Rule Resolution ────────────────────────────────────────
        // Rule 1: commit provided → use directly (bypass branch logic)
        // Rule 2: branch provided, no commit → resolve branch head
        // Rule 3: both null → use default_branch from repo config (fallback to global)
        // Rule 4: both null and no default_branch → hard error (versioned only)
        // In unversioned mode (no backend), resolving without a revision reads
        // the current filesystem state; branch/commit selectors are
        // unsatisfiable and produce a ResolutionFailed error.
        // ──────────────────────────────────────────────────────────────

        let unsatisfiable = |what: &str| NapError::ResolutionFailed {
            address: uri.to_string(),
            message: format!(
                "cannot resolve {what}: no version-control backend is configured. \
                     Configure one with 'nap backend configure' to use branch/commit selectors."
            ),
        };

        let revision: Option<String> = match (options.commit.as_ref(), options.branch.as_ref()) {
            (Some(commit), _) => {
                debug!(%commit, "resolve: rule 1 — commit provided");
                if repo.vcs().is_none() {
                    return Err(unsatisfiable(&format!("at commit '{commit}'")));
                }
                Some(commit.clone())
            }
            (None, Some(branch)) => {
                debug!(%branch, "resolve: rule 2 — branch provided");
                let vcs = repo
                    .vcs()
                    .ok_or_else(|| unsatisfiable(&format!("at branch '{branch}'")))?;
                Some(vcs.resolve_branch_head(&repo.root, branch)?)
            }
            (None, None) => {
                let default_branch = repo_config
                    .default_branch
                    .as_ref()
                    .or(self.config.default_branch.as_ref());
                match default_branch {
                    Some(default_branch) => {
                        debug!(%default_branch, "resolve: rule 3 — using default_branch");
                        let vcs = repo.vcs().ok_or_else(|| {
                            unsatisfiable(&format!("at default branch '{default_branch}'"))
                        })?;
                        Some(vcs.resolve_branch_head(&repo.root, default_branch)?)
                    }
                    None if repo.vcs().is_some() => {
                        debug!("resolve: rule 4 — no branch, no commit, no default_branch");
                        return Err(NapError::NoDefaultBranch);
                    }
                    None => {
                        debug!("resolve: unversioned — reading current filesystem state");
                        None
                    }
                }
            }
        };

        // Read the manifest at the resolved revision, or the current filesystem
        // state when resolving without a revision (unversioned mode).
        let manifest = match &revision {
            Some(revision) => {
                repo.read_manifest_at_ref(&uri.entity_type, &uri.entity_id, revision)?
            }
            None => repo.read_manifest(&uri.entity_type, &uri.entity_id)?,
        };

        let wants_provenance =
            options.provenance.unwrap_or(false) || options.include_blobs.unwrap_or(false);
        if wants_provenance {
            if let Some(path) = query_path {
                return Err(NapError::Other(format!(
                    "provenance envelopes are only supported for full manifest resolution, not subtree query '{path}'"
                )));
            }

            // Provenance is VCS-backed; it cannot be produced in unversioned mode.
            let revision = revision
                .as_deref()
                .ok_or_else(|| NapError::BackendNotConfigured {
                    operation: "provenance".to_string(),
                })?;

            let envelope = self.build_provenance_envelope(
                &repo,
                uri,
                manifest,
                revision,
                options.include_blobs.unwrap_or(false),
            )?;
            info!(uri = %uri, "resolved NAP URI with provenance");
            return Ok(ResolveResult::Provenance(Box::new(envelope)));
        }

        // Apply query if present
        match query_path {
            Some(ref path) => {
                debug!(query_path = %path, "applying subtree query");
                let yaml_value = manifest.to_value()?;
                let result = ManifestQuery::query(&yaml_value, path, &manifest.id)?;

                // Convert YAML value to JSON for consistent API output
                let json_str = serde_yaml::to_string(&result)
                    .map_err(|e| NapError::ManifestValidationError(e.to_string()))?;
                let json_value: serde_json::Value = serde_yaml::from_str(&json_str)
                    .map_err(|e| NapError::ManifestValidationError(e.to_string()))?;

                info!(
                    uri = %uri,
                    query_path = %path,
                    "resolved NAP URI with query"
                );
                Ok(ResolveResult::Subtree(json_value))
            }
            None => {
                info!(uri = %uri, "resolved NAP URI (full manifest)");
                Ok(ResolveResult::Full(Box::new(manifest)))
            }
        }
    }

    fn build_provenance_envelope(
        &self,
        repo: &Repository,
        uri: &NapUri,
        manifest: Manifest,
        revision: &str,
        include_blobs: bool,
    ) -> Result<ResolveEnvelope, NapError> {
        let manifest_path = uri.manifest_path();
        let mut files = vec![self.build_provenance_file(
            repo,
            revision,
            "manifest",
            None,
            Some(manifest_path.clone()),
            None,
            None,
            None,
            include_blobs,
        )?];

        for (name, representation) in &manifest.representations {
            let resolved_path = representation
                .uri
                .as_deref()
                .map(|representation_uri| {
                    Self::resolve_representation_path(uri, representation_uri)
                })
                .transpose()?
                .flatten();

            files.push(self.build_provenance_file(
                repo,
                revision,
                "representation",
                Some(name.clone()),
                resolved_path,
                representation.uri.clone(),
                Some(representation.hash.clone()),
                Some(representation.format.clone()),
                include_blobs,
            )?);
        }

        Ok(ResolveEnvelope {
            manifest: Box::new(manifest),
            provenance: ResolveProvenanceEnvelope {
                revision: revision.to_string(),
                files,
            },
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn build_provenance_file(
        &self,
        repo: &Repository,
        revision: &str,
        role: &str,
        name: Option<String>,
        path: Option<String>,
        uri: Option<String>,
        hash: Option<String>,
        format: Option<String>,
        include_blobs: bool,
    ) -> Result<ResolveProvenanceFile, NapError> {
        // Provenance is VCS-backed; in unversioned mode there is nothing to read.
        let vcs = repo.vcs().ok_or_else(|| NapError::BackendNotConfigured {
            operation: "provenance".to_string(),
        })?;

        let metadata = match path.as_deref() {
            Some(path) => vcs.file_metadata_at_ref(&repo.root, path, revision)?,
            None => None,
        };

        let blobs = if include_blobs {
            match metadata.as_ref() {
                Some(metadata) => Self::hydrate_known_blobs(vcs, repo, metadata)?,
                None => BTreeMap::new(),
            }
        } else {
            BTreeMap::new()
        };

        let provenance = match metadata {
            Some(metadata) => {
                let condensed = Self::condense_metadata(metadata);
                if condensed.is_empty() {
                    serde_json::Value::String("none".to_string())
                } else {
                    serde_json::to_value(condensed).map_err(|e| {
                        NapError::Other(format!("failed to serialize provenance metadata: {e}"))
                    })?
                }
            }
            None => serde_json::Value::String("none".to_string()),
        };

        Ok(ResolveProvenanceFile {
            role: role.to_string(),
            name,
            path,
            uri,
            hash,
            format,
            provenance,
            blobs,
        })
    }

    fn condense_metadata(metadata: BTreeMap<String, String>) -> BTreeMap<String, String> {
        metadata
            .into_iter()
            .filter(|(_, value)| value.len() <= MAX_CONDENSED_METADATA_VALUE_BYTES)
            .collect()
    }

    fn hydrate_known_blobs(
        vcs: &dyn VcsBackend,
        repo: &Repository,
        metadata: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, HydratedProvenanceBlob>, NapError> {
        let known_blob_keys = [
            ("prompt", "nap.provenance.prompt.address"),
            ("run", "nap.provenance.run.address"),
            ("parameters", "nap.provenance.parameters.address"),
        ];

        let mut blobs = BTreeMap::new();
        for (name, metadata_key) in known_blob_keys {
            let Some(address) = metadata.get(metadata_key) else {
                continue;
            };
            let content = vcs.read_provenance_blob(&repo.root, address)?;
            blobs.insert(name.to_string(), Self::truncate_blob(address, &content));
        }
        Ok(blobs)
    }

    fn truncate_blob(address: &str, content: &str) -> HydratedProvenanceBlob {
        let original_bytes = content.len();
        let mut included_bytes = 0;
        let mut truncated_content = String::new();

        for ch in content.chars() {
            let next_len = included_bytes + ch.len_utf8();
            if next_len > MAX_HYDRATED_BLOB_BYTES {
                break;
            }
            truncated_content.push(ch);
            included_bytes = next_len;
        }

        HydratedProvenanceBlob {
            address: address.to_string(),
            content: truncated_content,
            truncated: included_bytes < original_bytes,
            original_bytes,
            included_bytes,
        }
    }

    fn resolve_representation_path(
        uri: &NapUri,
        representation_uri: &str,
    ) -> Result<Option<String>, NapError> {
        if representation_uri.contains("://") {
            return Ok(None);
        }

        let representation_path = Path::new(representation_uri);
        if representation_path.is_absolute() {
            return Err(NapError::InvalidQueryPath(format!(
                "representation URI must be relative for provenance lookup: {representation_uri}"
            )));
        }

        let mut clean = PathBuf::new();
        for component in representation_path.components() {
            match component {
                Component::Normal(part) => clean.push(part),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(NapError::InvalidQueryPath(format!(
                        "unsafe representation URI for provenance lookup: {representation_uri}"
                    )));
                }
            }
        }

        // Match nap add: representation URIs are relative to the entity's
        // asset directory, including world entities whose manifest is at root.
        let entity_dir = Path::new(uri.entity_type.as_str()).join(&uri.entity_id);
        Ok(Some(Self::path_to_lore_path(&entity_dir.join(clean))))
    }

    fn path_to_lore_path(path: &Path) -> String {
        path.components()
            .filter_map(|component| match component {
                Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    /// Resolve a URI recursively, following nested nap:// URIs.
    fn resolve_uri_recursive(
        &self,
        uri: &NapUri,
        options: &ResolveOptions,
        depth: usize,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<ResolveResult, NapError> {
        // Check depth limit
        let max_depth = options.max_depth.unwrap_or(10);
        if depth >= max_depth {
            debug!(depth, max_depth, "reached maximum recursion depth");
            return self.resolve_uri_single(uri, options);
        }

        // Check for circular references
        let uri_str = uri.to_string();
        if visited.contains(&uri_str) {
            debug!(uri = %uri_str, "detected circular reference, stopping recursion");
            return self.resolve_uri_single(uri, options);
        }
        visited.insert(uri_str.clone());

        debug!(uri = %uri_str, depth, "recursively resolving URI");

        // Resolve the current URI
        let result = self.resolve_uri_single(uri, options)?;

        // Extract nested URIs from the result and resolve them
        match result {
            ResolveResult::Full(manifest) => {
                let nested_uris = self.extract_nested_uris(&manifest);
                if nested_uris.is_empty() {
                    debug!(uri = %uri_str, "no nested URIs found, returning manifest");
                    return Ok(ResolveResult::Full(manifest));
                }

                debug!(uri = %uri_str, count = nested_uris.len(), "found nested URIs, resolving recursively");

                // Resolve nested URIs and merge them into the result
                let mut resolved_manifest = (*manifest).clone();
                for nested_uri in nested_uris {
                    let nested_uri_parsed: NapUri = nested_uri.parse()?;

                    let nested_result = self
                        .resolve_uri_recursive(&nested_uri_parsed, options, depth + 1, visited)
                        .map_err(|e| {
                            NapError::Other(format!(
                                "failed to resolve nested URI '{}' while resolving '{}': {}",
                                nested_uri, uri_str, e
                            ))
                        })?;

                    if let ResolveResult::Full(nested_manifest) = nested_result {
                        // Merge nested manifest into parent (simple merge for now)
                        // In the future, this could be more sophisticated based on schema
                        for (key, value) in nested_manifest.properties {
                            resolved_manifest.properties.insert(key, value);
                        }
                    }
                }

                Ok(ResolveResult::Full(Box::new(resolved_manifest)))
            }
            ResolveResult::Subtree(value) => {
                // For subtree queries, we don't recurse (would be complex to merge)
                debug!("subtree query, skipping recursive resolution");
                Ok(ResolveResult::Subtree(value))
            }
            ResolveResult::Provenance(envelope) => Ok(ResolveResult::Provenance(envelope)),
        }
    }

    /// Extract all nap:// URIs from a manifest.
    fn extract_nested_uris(&self, manifest: &Manifest) -> Vec<String> {
        let mut uris = Vec::new();

        // Search in properties
        for value in manifest.properties.values() {
            self.extract_uris_from_yaml_value(value, &mut uris);
        }

        // Search in references
        for value in manifest.references.values() {
            self.extract_uris_from_yaml_value(value, &mut uris);
        }

        // Deduplicate URIs to avoid resolving the same URI multiple times
        uris.sort();
        uris.dedup();
        uris
    }

    /// Recursively extract nap:// URIs from YAML values.
    fn extract_uris_from_yaml_value(&self, value: &serde_yaml::Value, uris: &mut Vec<String>) {
        match value {
            serde_yaml::Value::String(s) if s.starts_with("nap://") => {
                uris.push(s.clone());
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq {
                    self.extract_uris_from_yaml_value(item, uris);
                }
            }
            serde_yaml::Value::Mapping(map) => {
                for (_, v) in map {
                    self.extract_uris_from_yaml_value(v, uris);
                }
            }
            _ => {}
        }
    }

    /// Convenience: query a specific path on a URI.
    pub fn query(&self, uri_str: &str, path: &str) -> Result<serde_json::Value, NapError> {
        let options = ResolveOptions {
            path: Some(path.to_string()),
            ..Default::default()
        };
        match self.resolve(uri_str, &options)? {
            ResolveResult::Subtree(v) => Ok(v),
            ResolveResult::Full(m) => m.to_json_value(),
            ResolveResult::Provenance(envelope) => serde_json::to_value(envelope).map_err(|e| {
                NapError::Other(format!("failed to serialize provenance envelope: {e}"))
            }),
        }
    }

    /// List all repositories available.
    pub fn list_repositories(&self) -> Result<Vec<String>, NapError> {
        if self.source != ResolveSource::Local {
            let client = self.remote_client()?;
            return block_on_grpc(async move { client.list_repositories().await });
        }
        let mut repositories = Vec::new();
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            // Check for repository.yaml or repository.yaml to identify valid repositories
            if path.is_dir()
                && (path.join("repository.yaml").exists() || path.join("repository.yaml").exists())
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                repositories.push(name.to_string());
            }
        }
        repositories.sort();
        Ok(repositories)
    }

    /// List entity YAMLs from the default branch of the configured Lore server.
    pub fn list_remote_entities(
        &self,
        repository: &str,
        entity_type: &crate::types::EntityType,
    ) -> Result<Vec<String>, NapError> {
        let client = self.remote_client()?;
        let repository_name = repository.to_string();
        let prefix = format!("{}/", entity_type.directory_name());
        let paths = block_on_grpc(async move {
            let repo = client.get_repository_by_name(&repository_name).await?;
            client
                .for_repository_id(repo.id.clone())
                .list_paths_at_revision(
                    crate::grpc_client::proto_gen::lore::model::v1::RevisionIdentifier {
                        branch_id: repo.default_branch_id,
                        number: 0,
                    },
                    prefix,
                )
                .await
        })?;
        let mut entities = paths
            .into_iter()
            .filter_map(|path| {
                path.strip_prefix(&format!("{}/", entity_type.directory_name()))
                    .map(str::to_string)
            })
            .filter(|path| !path.contains('/'))
            .filter_map(|path| path.strip_suffix(".yaml").map(str::to_string))
            .collect::<Vec<_>>();
        entities.sort();
        Ok(entities)
    }

    /// List repository-relative entity manifest paths from the default branch.
    pub fn list_remote_manifest_paths(&self, repository: &str) -> Result<Vec<String>, NapError> {
        let client = self.remote_client()?;
        let repository_name = repository.to_string();
        block_on_grpc(async move {
            let repo = client.get_repository_by_name(&repository_name).await?;
            let mut paths = client
                .for_repository_id(repo.id.clone())
                .list_paths_at_revision(
                    crate::grpc_client::proto_gen::lore::model::v1::RevisionIdentifier {
                        branch_id: repo.default_branch_id,
                        number: 0,
                    },
                    String::new(),
                )
                .await?;
            paths.retain(|path| {
                path != "repository.yaml"
                    && path.ends_with(".yaml")
                    && path.matches('/').count() == 1
            });
            paths.sort();
            Ok(paths)
        })
    }

    pub fn list_remote_branches(&self, repository: &str) -> Result<Vec<String>, NapError> {
        let client = self.remote_client()?;
        let repository_name = repository.to_string();
        block_on_grpc(async move {
            let repo = client.get_repository_by_name(&repository_name).await?;
            let mut branches = client
                .for_repository_id(repo.id)
                .list_branches()
                .await?
                .into_iter()
                .map(|branch| branch.name)
                .collect::<Vec<_>>();
            branches.sort();
            Ok(branches)
        })
    }

    pub fn remote_head_hash(&self, repository: &str) -> Result<String, NapError> {
        let client = self.remote_client()?;
        let repository_name = repository.to_string();
        block_on_grpc(async move {
            let repo = client.get_repository_by_name(&repository_name).await?;
            let branch = client
                .for_repository_id(repo.id)
                .get_branch_by_name(&repo.default_branch_name)
                .await?;
            Ok(hex::encode(branch.latest))
        })
    }

    pub fn remote_history(
        &self,
        uri: &NapUri,
        limit: usize,
    ) -> Result<Vec<crate::vcs::CommitInfo>, NapError> {
        let client = self.remote_client()?;
        let repository_name = uri.repository.clone();
        let manifest_path = uri.manifest_path();
        block_on_grpc(async move {
            let repo = client.get_repository_by_name(&repository_name).await?;
            let scoped = client.for_repository_id(repo.id.clone());
            let items = scoped
                .list_revisions(
                    crate::grpc_client::proto_gen::lore::model::v1::RevisionIdentifier {
                        branch_id: repo.default_branch_id,
                        number: 0,
                    },
                )
                .await?;
            let mut history = Vec::new();
            for item in items {
                let revision = scoped
                    .revision_info_at_signature(item.signature.to_vec())
                    .await?;
                let identifier = revision.identifier.clone().ok_or_else(|| {
                    NapError::GrpcError("RevisionInfo returned no identifier".to_string())
                })?;
                let current = match scoped
                    .file_address_at_revision(identifier, manifest_path.clone())
                    .await
                {
                    Ok(value) => Some(value),
                    Err(NapError::ManifestNotFound(_)) => None,
                    Err(error) => return Err(error),
                };
                let parent = revision.parent_self.as_ref();
                let previous = match parent.and_then(|parent| parent.identifier.clone()) {
                    Some(identifier) => match scoped
                        .file_address_at_revision(identifier, manifest_path.clone())
                        .await
                    {
                        Ok(value) => Some(value),
                        Err(NapError::ManifestNotFound(_)) => None,
                        Err(error) => return Err(error),
                    },
                    None => None,
                };
                let current_address = current
                    .as_ref()
                    .map(|(address, _)| (&address.hash, &address.context));
                let previous_address = previous
                    .as_ref()
                    .map(|(address, _)| (&address.hash, &address.context));
                if current_address == previous_address {
                    continue;
                }
                let parent = revision
                    .parent_self
                    .as_ref()
                    .map(|parent| hex::encode(&parent.signature));
                let timestamp =
                    chrono::DateTime::<chrono::Utc>::from_timestamp(revision.timestamp as i64, 0)
                        .map(|time| time.to_rfc3339())
                        .unwrap_or_default();
                history.push(crate::vcs::CommitInfo::from_lore_revision(
                    &hex::encode(revision.signature),
                    parent.as_deref(),
                    &revision.committed_by,
                    &revision.commit_message,
                    &timestamp,
                ));
                if history.len() == limit {
                    break;
                }
            }
            Ok(history)
        })
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::manifest::Representation;
    use crate::test_utils::MockBackend;
    use crate::types::EntityType;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Resolver) {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("toystory");
        let repo = Repository::init(&repo_path, "toystory", Box::new(MockBackend::new())).unwrap();

        // Create a character
        let (mut manifest, _) = repo
            .create_entity(&EntityType::new("character"), "woody", "Woody", "test")
            .unwrap();

        // Add properties and commit
        manifest.set_property("toy_type", serde_yaml::Value::String("plush".to_string()));
        manifest.set_property(
            "homeworld",
            serde_yaml::Value::String("nap://toystory/location/andys-room".to_string()),
        );
        manifest.add_reference(
            "appears_in",
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::String(
                "nap://toystory/scene/pizza-planet".to_string(),
            )]),
        );
        manifest.set_representation(
            "face_image",
            Representation {
                hash: "blake3:9753abf79e5aef60bd95ab76c1e5a14d01239beb37ff9897b6af8e040eb2413a"
                    .to_string(),
                format: "png".to_string(),
                uri: Some("face_image.png".to_string()),
                tier: None,
            },
        );

        use crate::commit::Change;
        repo.commit_manifest(
            &mut manifest,
            "add Woody details",
            "test",
            vec![Change::set(
                "properties.toy_type",
                None,
                "plush".to_string(),
            )],
        )
        .unwrap();

        let resolver = Resolver::with_vcs_factory(
            tmp.path(),
            || Box::new(MockBackend::new()),
            ResolveConfig {
                default_branch: Some("main".to_string()),
            },
        );
        (tmp, resolver)
    }

    #[test]
    fn test_resolve_full_manifest() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve("nap://toystory/character/woody", &Default::default())
            .unwrap();
        match result {
            ResolveResult::Full(m) => {
                assert_eq!(m.name, "Woody");
                assert_eq!(m.entity_type.as_str(), "character");
            }
            _ => panic!("expected full manifest"),
        }
    }

    fn write_mock_metadata(repo_path: &Path, metadata: BTreeMap<String, BTreeMap<String, String>>) {
        std::fs::write(
            repo_path.join(".mock_file_metadata.json"),
            serde_json::to_string(&metadata).unwrap(),
        )
        .unwrap();
    }

    fn write_mock_blobs(repo_path: &Path, blobs: BTreeMap<String, String>) {
        std::fs::write(
            repo_path.join(".mock_provenance_blobs.json"),
            serde_json::to_string(&blobs).unwrap(),
        )
        .unwrap();
    }

    fn resolve_with_provenance(resolver: &Resolver) -> ResolveEnvelope {
        let result = resolver
            .resolve(
                "nap://toystory/character/woody",
                &ResolveOptions {
                    provenance: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        match result {
            ResolveResult::Provenance(envelope) => *envelope,
            _ => panic!("expected provenance envelope"),
        }
    }

    #[test]
    fn test_resolve_with_provenance_returns_manifest_and_direct_file_entries() {
        let (tmp, resolver) = setup();
        let repo_path = tmp.path().join("toystory");
        write_mock_metadata(
            &repo_path,
            BTreeMap::from([
                (
                    "character/woody.yaml".to_string(),
                    BTreeMap::from([
                        ("nap.provenance.kind".to_string(), "edit".to_string()),
                        ("nap.provenance.model".to_string(), "gpt-5".to_string()),
                        (
                            "nap.provenance.long".to_string(),
                            "x".repeat(MAX_CONDENSED_METADATA_VALUE_BYTES + 1),
                        ),
                    ]),
                ),
                (
                    "character/woody/face_image.png".to_string(),
                    BTreeMap::from([("nap.provenance.kind".to_string(), "generation".to_string())]),
                ),
            ]),
        );

        let envelope = resolve_with_provenance(&resolver);
        assert_eq!(envelope.manifest.name, "Woody");
        assert_eq!(envelope.provenance.files.len(), 2);

        let manifest_file = &envelope.provenance.files[0];
        assert_eq!(manifest_file.role, "manifest");
        assert_eq!(manifest_file.path.as_deref(), Some("character/woody.yaml"));
        assert_eq!(manifest_file.provenance["nap.provenance.kind"], "edit");
        assert!(
            manifest_file
                .provenance
                .get("nap.provenance.long")
                .is_none()
        );

        let representation_file = &envelope.provenance.files[1];
        assert_eq!(representation_file.role, "representation");
        assert_eq!(representation_file.name.as_deref(), Some("face_image"));
        assert_eq!(
            representation_file.path.as_deref(),
            Some("character/woody/face_image.png")
        );
        assert_eq!(representation_file.uri.as_deref(), Some("face_image.png"));
        assert_eq!(representation_file.format.as_deref(), Some("png"));
    }

    #[test]
    fn test_resolve_with_provenance_records_path_and_revision_metadata_lookups() {
        let (tmp, resolver) = setup();
        let repo_path = tmp.path().join("toystory");
        let envelope = resolve_with_provenance(&resolver);

        let requests: Vec<BTreeMap<String, String>> = serde_json::from_str(
            &std::fs::read_to_string(repo_path.join(".mock_metadata_requests.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].get("path").unwrap(), "character/woody.yaml");
        assert_eq!(
            requests[0].get("revision").unwrap(),
            &envelope.provenance.revision
        );
        assert_eq!(
            requests[1].get("path").unwrap(),
            "character/woody/face_image.png"
        );
        assert_eq!(
            requests[1].get("revision").unwrap(),
            &envelope.provenance.revision
        );
        assert!(!requests.iter().any(|request| {
            request
                .get("path")
                .is_some_and(|path| path.starts_with("blake3:"))
        }));
    }

    #[test]
    fn test_resolve_with_provenance_uses_none_for_missing_metadata() {
        let (_tmp, resolver) = setup();
        let envelope = resolve_with_provenance(&resolver);
        assert_eq!(envelope.provenance.files[0].provenance, "none");
        assert_eq!(envelope.provenance.files[1].provenance, "none");
    }

    #[test]
    fn test_resolve_with_include_blobs_hydrates_known_readable_artifacts() {
        let (tmp, resolver) = setup();
        let repo_path = tmp.path().join("toystory");
        write_mock_metadata(
            &repo_path,
            BTreeMap::from([(
                "character/woody.yaml".to_string(),
                BTreeMap::from([
                    (
                        "nap.provenance.prompt.address".to_string(),
                        "lore:prompt:1".to_string(),
                    ),
                    (
                        "unrelated.artifact.address".to_string(),
                        "lore:binary:1".to_string(),
                    ),
                ]),
            )]),
        );
        write_mock_blobs(
            &repo_path,
            BTreeMap::from([("lore:prompt:1".to_string(), "Describe Woody".to_string())]),
        );

        let result = resolver
            .resolve(
                "nap://toystory/character/woody",
                &ResolveOptions {
                    provenance: Some(true),
                    include_blobs: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        let ResolveResult::Provenance(envelope) = result else {
            panic!("expected provenance envelope");
        };

        let blobs = &envelope.provenance.files[0].blobs;
        assert_eq!(blobs.len(), 1);
        assert_eq!(blobs["prompt"].content, "Describe Woody");
        assert!(!blobs["prompt"].truncated);
    }

    #[test]
    fn test_include_blobs_implies_provenance_envelope() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve(
                "nap://toystory/character/woody",
                &ResolveOptions {
                    include_blobs: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(matches!(result, ResolveResult::Provenance(_)));
    }

    #[test]
    fn test_resolve_with_include_blobs_truncates_readable_artifacts() {
        let (tmp, resolver) = setup();
        let repo_path = tmp.path().join("toystory");
        write_mock_metadata(
            &repo_path,
            BTreeMap::from([(
                "character/woody.yaml".to_string(),
                BTreeMap::from([(
                    "nap.provenance.prompt.address".to_string(),
                    "lore:prompt:large".to_string(),
                )]),
            )]),
        );
        write_mock_blobs(
            &repo_path,
            BTreeMap::from([(
                "lore:prompt:large".to_string(),
                "x".repeat(MAX_HYDRATED_BLOB_BYTES + 10),
            )]),
        );

        let result = resolver
            .resolve(
                "nap://toystory/character/woody",
                &ResolveOptions {
                    provenance: Some(true),
                    include_blobs: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        let ResolveResult::Provenance(envelope) = result else {
            panic!("expected provenance envelope");
        };
        let blob = &envelope.provenance.files[0].blobs["prompt"];
        assert!(blob.truncated);
        assert_eq!(blob.original_bytes, MAX_HYDRATED_BLOB_BYTES + 10);
        assert_eq!(blob.included_bytes, MAX_HYDRATED_BLOB_BYTES);
        assert_eq!(blob.content.len(), MAX_HYDRATED_BLOB_BYTES);
    }

    #[test]
    fn test_provenance_rejects_unsafe_representation_paths() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("toystory");
        let repo = Repository::init(&repo_path, "toystory", Box::new(MockBackend::new())).unwrap();
        let (mut manifest, _) = repo
            .create_entity(&EntityType::new("character"), "jessie", "Jessie", "test")
            .unwrap();
        manifest.set_representation(
            "unsafe",
            Representation {
                hash: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                format: "png".to_string(),
                uri: Some("../secret.png".to_string()),
                tier: None,
            },
        );
        use crate::commit::Change;
        repo.commit_manifest(
            &mut manifest,
            "add unsafe representation",
            "test",
            vec![Change::set(
                "representations.unsafe",
                None,
                "unsafe".to_string(),
            )],
        )
        .unwrap();

        let resolver = Resolver::with_vcs_factory(
            tmp.path(),
            || Box::new(MockBackend::new()),
            ResolveConfig {
                default_branch: Some("main".to_string()),
            },
        );
        let err = resolver
            .resolve(
                "nap://toystory/character/jessie",
                &ResolveOptions {
                    provenance: Some(true),
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("unsafe representation URI"));
    }

    #[test]
    fn test_resolve_with_fragment() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve(
                "nap://toystory/character/woody#properties.toy_type",
                &Default::default(),
            )
            .unwrap();
        match result {
            ResolveResult::Subtree(v) => {
                assert_eq!(v.as_str(), Some("plush"));
            }
            _ => panic!("expected subtree"),
        }
    }

    #[test]
    fn test_resolve_with_options_path() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve(
                "nap://toystory/character/woody",
                &ResolveOptions {
                    path: Some("properties.homeworld".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        match result {
            ResolveResult::Subtree(v) => {
                assert_eq!(v.as_str(), Some("nap://toystory/location/andys-room"));
            }
            _ => panic!("expected subtree"),
        }
    }

    #[test]
    fn test_query_convenience() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .query("nap://toystory/character/woody", "properties.toy_type")
            .unwrap();
        assert_eq!(result.as_str(), Some("plush"));
    }

    #[test]
    fn test_list_repositories() {
        let (_tmp, resolver) = setup();
        let repositories = resolver.list_repositories().unwrap();
        assert!(repositories.contains(&"toystory".to_string()));
    }

    #[test]
    fn test_resolve_not_found() {
        let (_tmp, resolver) = setup();
        let result = resolver.resolve("nap://toystory/character/nonexistent", &Default::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_without_scheme() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve("toystory/character/woody", &Default::default())
            .unwrap();
        match result {
            ResolveResult::Full(m) => {
                assert_eq!(m.name, "Woody");
                assert_eq!(m.entity_type.as_str(), "character");
            }
            _ => panic!("expected full manifest"),
        }
    }

    #[test]
    fn test_resolve_without_scheme_with_fragment() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve(
                "toystory/character/woody#properties.toy_type",
                &Default::default(),
            )
            .unwrap();
        match result {
            ResolveResult::Subtree(v) => {
                assert_eq!(v.as_str(), Some("plush"));
            }
            _ => panic!("expected subtree"),
        }
    }

    #[test]
    fn test_resolve_without_leading_slash() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve("toystory/character/woody", &Default::default())
            .unwrap();
        match result {
            ResolveResult::Full(m) => {
                assert_eq!(m.name, "Woody");
            }
            _ => panic!("expected full manifest"),
        }
    }

    #[test]
    fn test_resolve_with_leading_slash_without_scheme() {
        let (_tmp, resolver) = setup();
        let result = resolver
            .resolve("/toystory/character/woody", &Default::default())
            .unwrap();
        match result {
            ResolveResult::Full(m) => {
                assert_eq!(m.name, "Woody");
            }
            _ => panic!("expected full manifest"),
        }
    }

    #[test]
    fn presign_debug_output_redacts_secrets() {
        let options = PresignOptions {
            bearer_token: Some("secret-token".to_string()),
            ..Default::default()
        };
        let rendered = format!("{options:?}");
        assert!(rendered.contains("<redacted>"));
        assert!(!rendered.contains("secret-token"));

        let result = PresignedRepresentation {
            url: "https://example.test/v1/presigned/x?token=secret".to_string(),
            expires_at: 1,
            revision: "revision".to_string(),
            repository_id: "repository".to_string(),
            address: "address".to_string(),
            representation: "face_image".to_string(),
            format: "png".to_string(),
        };
        assert!(!format!("{result:?}").contains("token=secret"));
    }

    #[test]
    fn presigned_url_validation_rejects_cross_origin_and_extra_query_data() {
        let base = reqwest::Url::parse("https://lore.example.test").unwrap();
        let path = "/v1/presigned/repository/address";
        assert!(validate_presigned_url(&base, "//evil.test/x?token=x", path).is_err());
        assert!(
            validate_presigned_url(
                &base,
                "/v1/presigned/repository/address?token=x&redirect=https://evil.test",
                path,
            )
            .is_err()
        );
        assert!(
            validate_presigned_url(&base, "/v1/presigned/repository/address?token=opaque", path,)
                .is_ok()
        );
    }

    #[tokio::test]
    async fn presign_uses_repository_id_and_file_context_separately() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let (_tmp, resolver) = setup();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = vec![0_u8; 8192];
            let read = stream.read(&mut request).await.unwrap();
            let request = String::from_utf8_lossy(&request[..read]);
            assert!(request.contains(
                "POST /v1/repository/0123456789abcdef0123456789abcdef/content/9753abf79e5aef60bd95ab76c1e5a14d01239beb37ff9897b6af8e040eb2413a-fedcba9876543210fedcba9876543210/presign"
            ));
            assert!(request.contains("authorization: Bearer test-token"));
            assert!(request.contains("\"ttl_seconds\":90"));
            let body = concat!(
                "{\"url_suffix\":\"/v1/presigned/0123456789abcdef0123456789abcdef/",
                "9753abf79e5aef60bd95ab76c1e5a14d01239beb37ff9897b6af8e040eb2413a-",
                "fedcba9876543210fedcba9876543210?token=opaque\",\"expires_at\":12345}"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let result = resolver
            .presign_representation(
                "toystory/character/woody",
                "face_image",
                &PresignOptions {
                    ttl_seconds: Some(90),
                    lore_http_url: Some(format!("http://{address}")),
                    bearer_token: Some("test-token".to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        server.await.unwrap();
        assert_eq!(result.expires_at, 12345);
        assert_eq!(result.repository_id, "0123456789abcdef0123456789abcdef");
        assert!(result.url.ends_with("?token=opaque"));
    }

    #[tokio::test]
    async fn presign_reuses_existing_login_after_http_challenge() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let (tmp, resolver) = setup();
        std::fs::write(
            tmp.path().join("toystory/.mock_http_token"),
            "cached-repository-token",
        )
        .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            for authenticated in [false, true] {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = vec![0; 8192];
                let read = stream.read(&mut request).await.unwrap();
                let request = String::from_utf8_lossy(&request[..read]);
                assert_eq!(
                    request.contains("authorization: Bearer cached-repository-token"),
                    authenticated
                );
                let (status, body) = if authenticated {
                    (
                        "200 OK",
                        r#"{"url_suffix":"/v1/presigned/0123456789abcdef0123456789abcdef/9753abf79e5aef60bd95ab76c1e5a14d01239beb37ff9897b6af8e040eb2413a-fedcba9876543210fedcba9876543210?token=opaque","expires_at":12345}"#,
                    )
                } else {
                    ("401 Unauthorized", "unauthenticated")
                };
                stream.write_all(format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
            }
        });
        let result = resolver
            .presign_representation(
                "toystory/character/woody",
                "face_image",
                &PresignOptions {
                    lore_http_url: Some(origin),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(result.expires_at, 12345);
        server.await.unwrap();
    }

    #[test]
    fn representation_paths_use_the_entity_asset_directory() {
        for (entity, asset, expected) in [
            (
                "nap://25th-chapter/character/nathan-gunn",
                "item.jpg",
                "character/nathan-gunn/item.jpg",
            ),
            (
                "nap://25th-chapter/world/25th-chapter",
                "images/map.png",
                "world/25th-chapter/images/map.png",
            ),
        ] {
            let uri: NapUri = entity.parse().unwrap();
            assert_eq!(
                Resolver::resolve_representation_path(&uri, asset).unwrap(),
                Some(expected.to_string()),
            );
        }
    }

    #[tokio::test]
    async fn presign_rejects_fragment_and_conflicting_revision_selectors() {
        let (_tmp, resolver) = setup();
        assert!(
            resolver
                .presign_representation(
                    "nap://toystory/character/woody#properties",
                    "face_image",
                    &PresignOptions::default(),
                )
                .await
                .is_err()
        );
        assert!(
            resolver
                .presign_representation(
                    "nap://toystory/character/woody",
                    "face_image",
                    &PresignOptions {
                        branch: Some("main".to_string()),
                        commit: Some("abc".to_string()),
                        ..Default::default()
                    },
                )
                .await
                .unwrap_err()
                .to_string()
                .contains("either branch or commit")
        );
    }
}

#[cfg(all(test, feature = "lore-integration"))]
mod lore_tests {
    use super::*;
    use crate::types::EntityType;
    use crate::vcs_lore::LoreBackend;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::TempDir;

    fn unique_suffix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    fn setup_lore() -> (TempDir, Resolver, String) {
        let repository = format!("lr-{}", unique_suffix());
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join(&repository);
        let repo =
            Repository::init(&repo_path, &repository, Box::new(LoreBackend::from_env())).unwrap();

        // Create a character
        let (mut manifest, _) = repo
            .create_entity(&EntityType::new("character"), "woody", "Woody", "test")
            .unwrap();

        // Add properties and commit
        manifest.set_property("toy_type", serde_yaml::Value::String("plush".to_string()));
        use crate::commit::Change;
        repo.commit_manifest(
            &mut manifest,
            "add Woody details",
            "test",
            vec![Change::set(
                "properties.toy_type",
                None,
                "plush".to_string(),
            )],
        )
        .unwrap();

        let resolver = Resolver::with_vcs_factory(
            tmp.path(),
            || Box::new(LoreBackend::from_env()),
            ResolveConfig {
                default_branch: Some("main".to_string()),
            },
        );
        (tmp, resolver, repository)
    }

    #[test]
    fn test_resolve_lore_full_manifest() {
        let (_tmp, resolver, repository) = setup_lore();
        let uri = format!("nap://{}/character/woody", repository);
        let result = resolver.resolve(&uri, &Default::default()).unwrap();
        match result {
            ResolveResult::Full(m) => {
                assert_eq!(m.name, "Woody");
            }
            _ => panic!("expected full manifest"),
        }
    }
}
