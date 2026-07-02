//! gRPC client for lore-server's revision service (branch ref sync).
//!
//! The lore-server exposes its mutable state — branch pointers, revision
//! pointers — exclusively over gRPC, whereas content-addressed blob data
//! is transferred via HTTP / the `lore` CLI.  This module implements the
//! gRPC half of the push/pull protocol.
//!
//! # Architecture
//!
//! ```text
//! LoreBackend::push / pull   (sync, on tokio runtime)
//!     │
//!     ▼
//! block_on_grpc(…)           (spawns dedicated OS thread)
//!     │
//!     ▼
//! LoreGrpcClient             (tonic RevisionServiceClient wrapper)
//!     │
//!     ▼
//! lore-server gRPC endpoint
//!     ├── RevisionService.BranchGet   → fetch remote tip
//!     └── RevisionService.BranchPush  → advance remote tip
//! ```
//!
//! # Sync/Async Bridge
//!
//! The [`VcsBackend`] trait is synchronous.  gRPC is inherently async.
//! Rather than changing the trait (which would break every implementation),
//! we bridge via [`block_on_grpc`]: a dedicated OS thread hosts a shared
//! single-threaded tokio runtime that executes the async gRPC call.  This
//! avoids the "Cannot start a runtime from within a runtime" panic that
//! would occur if we called `Runtime::block_on` directly inside axum
//! request handlers.

// ---------------------------------------------------------------------------
// Generated proto modules — must nest exactly as prost expects for
// cross-package references (lore.revision.v1 → lore.model.v1)
// ---------------------------------------------------------------------------

/// Generated gRPC service and message types.
///
/// Two packages are compiled:
/// - `lore.model.v1`      — Branch, BranchPoint, etc.
/// - `lore.revision.v1`   — RevisionService, BranchGetRequest, etc.
pub mod proto_gen {
    #![allow(unreachable_pub)]
    pub mod lore {
        pub mod model {
            pub mod v1 {
                tonic::include_proto!("lore.model.v1");
            }
        }
        pub mod revision {
            pub mod v1 {
                tonic::include_proto!("lore.revision.v1");
            }
        }
    }
}

// Re-export the types callers need most frequently.
pub use proto_gen::lore::model::v1::Branch;
pub use proto_gen::lore::revision::v1::branch_get_request;
pub use proto_gen::lore::revision::v1::revision_service_client::RevisionServiceClient;
pub use proto_gen::lore::revision::v1::{BranchGetRequest, BranchPushRequest};

use std::future::Future;
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

use tonic::codegen::InterceptedService;
use tonic::metadata::{BinaryMetadataValue, MetadataValue};
use tonic::service::Interceptor;
use tonic::transport::{Channel, Endpoint};

use crate::error::NapError;

// ===========================================================================
// Auth interceptor
// ===========================================================================

/// Injects JWT bearer token and repository-scope metadata into every
/// outgoing gRPC request.
///
/// The token is sent as `Authorization: Bearer <token>` with
/// `set_sensitive(true)` so proxy logs do not leak it.
///
/// The repository ID is sent as binary metadata (keys with `-bin` suffix)
/// matching the lore-client's `inject_repository()` protocol.
#[derive(Clone)]
struct GrpcAuthInterceptor {
    token: Option<String>,
    repository_id_bytes: Vec<u8>,
}

impl Interceptor for GrpcAuthInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        // ── Authorization header ──────────────────────────────────────
        if let Some(ref token) = self.token
            && !token.is_empty()
        {
            let mut value: MetadataValue<_> = format!("Bearer {token}")
                .parse()
                .map_err(|e| tonic::Status::invalid_argument(format!("bad token metadata: {e}")))?;
            value.set_sensitive(true);
            request.metadata_mut().insert("authorization", value);
        }

        // ── Repository-scope binary metadata ──────────────────────────
        if !self.repository_id_bytes.is_empty() {
            let bin_val = BinaryMetadataValue::from_bytes(&self.repository_id_bytes);
            request
                .metadata_mut()
                .insert_bin("lore-partition", bin_val.clone());
            request
                .metadata_mut()
                .insert_bin("urc-repository-id", bin_val);
        }

        Ok(request)
    }
}

// ===========================================================================
// LoreGrpcClient
// ===========================================================================

/// A gRPC client for lore-server's [`RevisionService`].
///
/// This client handles **only** lightweight metadata operations:
///
/// | Operation | RPC | Purpose |
/// |-----------|-----|---------|
/// | `get_branch_by_name` | `BranchGet` | Fetch remote branch tip before pull |
/// | `push_branch` | `BranchPush` | Advance remote branch tip after push |
///
/// Blob transfer (the heavy payload) remains on the `lore` CLI / HTTP.
///
/// [`RevisionService`]: proto_gen::lore::revision::v1::revision_service_client::RevisionServiceClient
#[derive(Debug, Clone)]
pub struct LoreGrpcClient {
    channel: Channel,
    token: Option<String>,
    repository_id_bytes: Vec<u8>,
}

impl LoreGrpcClient {
    /// Return a builder for fine-grained configuration.
    pub fn builder() -> Builder {
        Builder::default()
    }

    // ── Public RPC methods ───────────────────────────────────────────

    /// Look up a branch by its human-readable name.
    ///
    /// Returns the [`Branch`] record containing `id` (binary UUID),
    /// `name`, `latest` (tip signature), and other metadata.
    pub async fn get_branch_by_name(&self, name: &str) -> Result<Branch, NapError> {
        let mut client = self.make_client();
        let response = client
            .branch_get(BranchGetRequest {
                query: Some(branch_get_request::Query::Name(name.to_string())),
            })
            .await
            .map_err(|status| map_grpc_status("BranchGet", status))?;

        response.into_inner().branch.ok_or_else(|| {
            NapError::GrpcError(format!("BranchGet({name}) returned empty branch record"))
        })
    }

    /// Push a revision as the new tip of a branch.
    ///
    /// * `branch_id` — binary branch UUID (obtained from
    ///   [`get_branch_by_name`]).
    /// * `revision_signature` — raw content hash of the revision to set as
    ///   the new tip.
    /// * `force` — if `true`, bypasses fast-forward checks on the server.
    ///   When `false`, the server requires the new tip to descend from the
    ///   current tip (or performs a fast-forward merge).
    pub async fn push_branch(
        &self,
        branch_id: bytes::Bytes,
        revision_signature: bytes::Bytes,
        force: bool,
    ) -> Result<(), NapError> {
        let mut client = self.make_client();
        client
            .branch_push(BranchPushRequest {
                id: branch_id,
                revision_signature,
                force,
                fast_forward_merge: !force,
            })
            .await
            .map_err(|status| map_grpc_status("BranchPush", status))?;
        Ok(())
    }

    // ── Internal helpers ─────────────────────────────────────────────

    /// Convenience constructor that reads all configuration from environment
    /// variables.  Returns `Ok(None)` when `NAP_LORE_GRPC_ENDPOINT` is not
    /// set, allowing callers to gracefully skip gRPC integration.
    ///
    /// See [`Builder::from_env`] for the list of recognised variables.
    pub fn builder_from_env() -> Result<Option<Self>, NapError> {
        Builder::from_env()
    }

    /// Build a fresh client with the interceptor wired in.
    fn make_client(
        &self,
    ) -> RevisionServiceClient<InterceptedService<Channel, GrpcAuthInterceptor>> {
        RevisionServiceClient::with_interceptor(
            self.channel.clone(),
            GrpcAuthInterceptor {
                token: self.token.clone(),
                repository_id_bytes: self.repository_id_bytes.clone(),
            },
        )
    }
}

// ===========================================================================
// Builder
// ===========================================================================

/// Configuration builder for [`LoreGrpcClient`].
///
/// # Environment variables
///
/// | Variable | Required | Default | Description |
/// |----------|----------|---------|-------------|
/// | `NAP_LORE_GRPC_ENDPOINT` | Yes | — | gRPC endpoint URL |
/// | `NAP_LORE_GRPC_TOKEN` | No | — | JWT bearer token |
/// | `NAP_LORE_GRPC_RID` | No | — | Repository ID (hex-encoded binary) |
/// | `NAP_LORE_GRPC_INSECURE` | No | `0` | Skip TLS verification when `1` |
#[derive(Default)]
pub struct Builder {
    endpoint: Option<String>,
    token: Option<String>,
    repository_id_bytes: Vec<u8>,
    insecure: bool,
}

impl Builder {
    /// Set the gRPC endpoint URL.
    ///
    /// Format: `https://host:port` (TLS) or `http://host:port` (plain).
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Set a JWT bearer token for authenticated requests.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set the repository ID to inject as binary metadata.
    ///
    /// This should match the repository / partition UUID the lore-server
    /// expects.  Pass the raw bytes (not hex-encoded).
    pub fn repository_id(mut self, id: impl Into<Vec<u8>>) -> Self {
        self.repository_id_bytes = id.into();
        self
    }

    /// When `true`, skip TLS certificate validation.
    ///
    /// Use this in development environments where the lore-server uses
    /// self-signed certificates.
    pub fn insecure(mut self, insecure: bool) -> Self {
        self.insecure = insecure;
        self
    }

    /// Build the [`LoreGrpcClient`].
    ///
    /// Connection is deferred via [`Endpoint::connect_lazy`]; the
    /// first RPC will establish the TCP + TLS handshake.
    pub fn build(self) -> Result<LoreGrpcClient, NapError> {
        let endpoint_str = self.endpoint.ok_or_else(|| {
            NapError::GrpcError(
                "gRPC endpoint is required — set via .endpoint() or NAP_LORE_GRPC_ENDPOINT"
                    .to_string(),
            )
        })?;

        // In insecure mode, downgrade https:// → http:// to skip TLS
        // verification entirely (self-signed certs in development).
        // In secure mode, Endpoint::from_shared auto-configures TLS with
        // native roots for https:// URLs — no explicit tls_config needed.
        let effective_url = if self.insecure {
            endpoint_str
                .strip_prefix("https://")
                .map(|rest| format!("http://{rest}"))
                .unwrap_or_else(|| endpoint_str.clone())
        } else {
            endpoint_str.clone()
        };

        let channel = Endpoint::from_shared(effective_url)
            .map_err(|e| {
                NapError::GrpcError(format!("invalid gRPC endpoint '{endpoint_str}': {e}"))
            })?
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(20))
            .user_agent(concat!("nap-core/", env!("CARGO_PKG_VERSION")))
            .map_err(|e| NapError::GrpcError(format!("user-agent configuration error: {e}")))?
            .connect_lazy();

        Ok(LoreGrpcClient {
            channel,
            token: self.token,
            repository_id_bytes: self.repository_id_bytes,
        })
    }

    /// Build from environment variables.
    ///
    /// Returns `Ok(None)` when `NAP_LORE_GRPC_ENDPOINT` is not set
    /// (allowing the caller to skip gRPC integration gracefully).
    pub fn from_env() -> Result<Option<LoreGrpcClient>, NapError> {
        let endpoint = match std::env::var("NAP_LORE_GRPC_ENDPOINT") {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };

        let token = std::env::var("NAP_LORE_GRPC_TOKEN").ok();
        let insecure = std::env::var("NAP_LORE_GRPC_INSECURE")
            .ok()
            .is_some_and(|v| v == "1" || v == "true" || v == "yes");

        let repository_id_bytes = std::env::var("NAP_LORE_GRPC_RID")
            .ok()
            .map(|hex| {
                hex::decode(&hex).map_err(|e| {
                    NapError::GrpcError(format!("invalid NAP_LORE_GRPC_RID hex '{hex}': {e}"))
                })
            })
            .transpose()?
            .unwrap_or_default();

        let mut builder = Builder::default().endpoint(endpoint).insecure(insecure);

        if let Some(t) = token {
            builder = builder.token(t);
        }
        if !repository_id_bytes.is_empty() {
            builder = builder.repository_id(repository_id_bytes);
        }

        builder.build().map(Some)
    }
}

// ===========================================================================
// Sync→async bridge
// ===========================================================================

/// Execute an async gRPC operation from a synchronous context.
///
/// # Why a dedicated thread?
///
/// The [`VcsBackend`] trait methods (`push`, `pull`) are synchronous.
/// gRPC client calls are async.  If we called `Runtime::block_on` directly
/// from within an axum HTTP handler (which already runs on a tokio runtime),
/// tokio would panic with "Cannot start a runtime from within a runtime".
///
/// This function spawns a **dedicated OS thread** that hosts the future
/// on a shared single-threaded tokio runtime.  The runtime is created once
/// and reused across all gRPC calls, preserving HTTP/2 keepalive state and
/// TLS session tickets.
///
/// # Type bounds
///
/// * `F` must be `Send + 'static` because it crosses a thread boundary.
/// * `T` must be `Send + 'static` for the same reason.
/// * The closure return type is `Result<T, NapError>` so that error
///   propagation through the thread join is straightforward.
///
/// [`VcsBackend`]: crate::vcs::VcsBackend
pub fn block_on_grpc<F, T>(f: F) -> Result<T, NapError>
where
    F: Future<Output = Result<T, NapError>> + Send + 'static,
    T: Send + 'static,
{
    static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build gRPC tokio runtime")
    });

    // `&'static Runtime` is both `Send` and `Sync` because the static
    // reference lives forever.  It is safe to pass to a spawned thread.
    let rt: &'static tokio::runtime::Runtime = &RUNTIME;

    thread::Builder::new()
        .name("nap-grpc".into())
        .spawn(move || rt.block_on(f))
        .expect("failed to spawn gRPC worker thread")
        .join()
        .map_err(|panic_payload| {
            NapError::GrpcError(format!("gRPC thread panicked: {panic_payload:?}"))
        })?
}

// ===========================================================================
// Error mapping
// ===========================================================================

/// Map a [`tonic::Status`] to a structured [`NapError`].
fn map_grpc_status(context: &str, status: tonic::Status) -> NapError {
    let code = status.code();
    let message = status.message();
    match code {
        tonic::Code::NotFound => NapError::RefNotFound(format!("{context}: {message}")),
        tonic::Code::Unauthenticated | tonic::Code::PermissionDenied => {
            NapError::PermissionDenied(format!("{context}: {message}"))
        }
        _ => NapError::GrpcError(format!("{context} ({code}): {message}")),
    }
}
