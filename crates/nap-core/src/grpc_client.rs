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
        pub mod repository {
            pub mod v1 {
                tonic::include_proto!("lore.repository.v1");
            }
        }
        pub mod storage {
            pub mod v1 {
                tonic::include_proto!("lore.storage.v1");
            }
        }
        pub mod thin_client {
            pub mod v1 {
                tonic::include_proto!("lore.thin_client.v1");
            }
        }
    }
}

// Re-export the types callers need most frequently.
pub use proto_gen::lore::model::v1::Branch;
pub use proto_gen::lore::repository::v1::repository_service_client::RepositoryServiceClient;
pub use proto_gen::lore::repository::v1::{RepositoryGetRequest, RepositoryListRequest};
pub use proto_gen::lore::revision::v1::branch_get_request;
pub use proto_gen::lore::revision::v1::revision_service_client::RevisionServiceClient;
pub use proto_gen::lore::revision::v1::{BranchGetRequest, BranchPushRequest};
pub use proto_gen::lore::revision::v1::{BranchListRequest, RevisionListRequest};
pub use proto_gen::lore::storage::v1::storage_service_client::StorageServiceClient;
use proto_gen::lore::thin_client::v1::revision_tree_request::Query as RevisionTreeQuery;
pub use proto_gen::lore::thin_client::v1::thin_client_service_client::ThinClientServiceClient;
pub use proto_gen::lore::thin_client::v1::{RevisionInfoRequest, RevisionTreeRequest};

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
                .insert_bin("lore-partition-bin", bin_val.clone());
            request
                .metadata_mut()
                .insert_bin("urc-repository-id-bin", bin_val);
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

    /// Return a clone scoped to a repository returned by `RepositoryGet`.
    pub fn for_repository_id(&self, id: impl Into<Vec<u8>>) -> Self {
        Self {
            channel: self.channel.clone(),
            token: self.token.clone(),
            repository_id_bytes: id.into(),
        }
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

    pub async fn list_branches(&self) -> Result<Vec<Branch>, NapError> {
        let mut client = self.make_client();
        let mut stream = client
            .branch_list(BranchListRequest {
                creator: None,
                include_deleted: false,
            })
            .await
            .map_err(|status| map_grpc_status("BranchList", status))?
            .into_inner();
        let mut branches = Vec::new();
        while let Some(item) = stream
            .message()
            .await
            .map_err(|status| map_grpc_status("BranchList", status))?
        {
            if let Some(branch) = item.branch {
                branches.push(branch);
            }
        }
        Ok(branches)
    }

    pub async fn list_revisions(
        &self,
        identifier: proto_gen::lore::model::v1::RevisionIdentifier,
    ) -> Result<Vec<proto_gen::lore::model::v1::RevisionItem>, NapError> {
        let mut client = self.make_client();
        Ok(client
            .revision_list(RevisionListRequest {
                start: Some(
                    proto_gen::lore::revision::v1::revision_list_request::Start::Identifier(
                        identifier,
                    ),
                ),
            })
            .await
            .map_err(|status| map_grpc_status("RevisionList", status))?
            .into_inner()
            .items)
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

    fn make_repository_client(
        &self,
    ) -> RepositoryServiceClient<InterceptedService<Channel, GrpcAuthInterceptor>> {
        RepositoryServiceClient::with_interceptor(
            self.channel.clone(),
            GrpcAuthInterceptor {
                token: self.token.clone(),
                repository_id_bytes: Vec::new(),
            },
        )
    }

    fn make_storage_client(
        &self,
    ) -> StorageServiceClient<InterceptedService<Channel, GrpcAuthInterceptor>> {
        StorageServiceClient::with_interceptor(
            self.channel.clone(),
            GrpcAuthInterceptor {
                token: self.token.clone(),
                repository_id_bytes: self.repository_id_bytes.clone(),
            },
        )
    }

    fn make_thin_client(
        &self,
    ) -> ThinClientServiceClient<InterceptedService<Channel, GrpcAuthInterceptor>> {
        ThinClientServiceClient::with_interceptor(
            self.channel.clone(),
            GrpcAuthInterceptor {
                token: self.token.clone(),
                repository_id_bytes: self.repository_id_bytes.clone(),
            },
        )
    }

    /// Construct a directory-scoped `RevisionTree` request for one file.
    ///
    /// Lore's `RevisionTree` API cannot use a file as `path_prefix`; it walks
    /// from a directory root. File readers must therefore request the parent
    /// directory and inspect its direct children for the exact path.
    fn revision_tree_for_file(query: RevisionTreeQuery, path: &str) -> RevisionTreeRequest {
        RevisionTreeRequest {
            query: Some(query),
            path_prefix: Some(parent_tree_path(path)),
            max_depth: Some(1),
        }
    }

    /// Read a complete Lore storage object, reassembling its fragment tree.
    ///
    /// `StorageService.Get` returns the raw payload of an object. Large files
    /// are represented by a root payload containing 40-byte fragment
    /// references (32-byte hash plus an LE content offset), so callers must
    /// recursively fetch those leaves before parsing a manifest.
    async fn read_storage_content(
        &self,
        address: proto_gen::lore::model::v1::Address,
    ) -> Result<Vec<u8>, NapError> {
        let mut pending = vec![(address, 0_u64)];
        let mut content = None;
        let mut ranges = Vec::<std::ops::Range<usize>>::new();
        let mut fragments_seen = 0_usize;

        while let Some((address, base_offset)) = pending.pop() {
            fragments_seen += 1;
            if fragments_seen > 100_000 {
                return Err(storage_protocol_error(
                    "fragment tree exceeds 100,000 nodes",
                ));
            }

            let (fragment, payload) = self.read_storage_fragment(address.clone()).await?;
            let size_content = usize::try_from(fragment.size_content).map_err(|_| {
                storage_protocol_error("fragment content size exceeds platform limits")
            })?;

            if content.is_none() {
                content = Some(vec![0; size_content]);
            }
            let root_size = content.as_ref().expect("content initialized").len();
            let fragment_end = base_offset
                .checked_add(fragment.size_content)
                .and_then(|end| usize::try_from(end).ok())
                .ok_or_else(|| storage_protocol_error("fragment offset overflows"))?;
            if fragment_end > root_size {
                return Err(storage_protocol_error(
                    "fragment extends beyond root content",
                ));
            }

            if fragment.flags & FRAGMENT_PAYLOAD_FRAGMENTED != 0 {
                for reference in decode_fragment_references(&payload)? {
                    let child_offset =
                        base_offset.checked_add(reference.offset).ok_or_else(|| {
                            storage_protocol_error("fragment reference offset overflows")
                        })?;
                    pending.push((
                        proto_gen::lore::model::v1::Address {
                            hash: reference.hash.into(),
                            context: address.context.clone(),
                        },
                        child_offset,
                    ));
                }
            } else {
                let decoded = decode_fragment_payload(&fragment, &payload)?;
                let end = base_offset
                    .checked_add(decoded.len() as u64)
                    .and_then(|end| usize::try_from(end).ok())
                    .ok_or_else(|| storage_protocol_error("fragment payload offset overflows"))?;
                if end > root_size || decoded.len() != size_content {
                    return Err(storage_protocol_error(
                        "leaf payload has an invalid content size",
                    ));
                }
                let start = base_offset as usize;
                if ranges
                    .iter()
                    .any(|range| start < range.end && end > range.start)
                {
                    return Err(storage_protocol_error("fragment leaves overlap"));
                }
                content.as_mut().expect("content initialized")[start..end]
                    .copy_from_slice(&decoded);
                ranges.push(start..end);
            }
        }

        let content =
            content.ok_or_else(|| storage_protocol_error("storage returned no fragments"))?;
        ranges.sort_unstable_by_key(|range| range.start);
        let mut cursor = 0;
        for range in ranges {
            if range.start != cursor {
                return Err(storage_protocol_error(
                    "fragment leaves do not cover root content",
                ));
            }
            cursor = range.end;
        }
        if cursor != content.len() {
            return Err(storage_protocol_error(
                "fragment leaves do not cover root content",
            ));
        }
        Ok(content)
    }

    async fn read_storage_fragment(
        &self,
        address: proto_gen::lore::model::v1::Address,
    ) -> Result<(proto_gen::lore::model::v1::Fragment, Vec<u8>), NapError> {
        let mut storage = self.make_storage_client();
        let mut stream = storage
            .get(tokio_stream::iter([address]))
            .await
            .map_err(|status| map_grpc_status("StorageGet", status))?
            .into_inner();
        let response = stream
            .message()
            .await
            .map_err(|status| map_grpc_status("StorageGet", status))?
            .ok_or_else(|| storage_protocol_error("storage returned no response"))?;
        if stream
            .message()
            .await
            .map_err(|status| map_grpc_status("StorageGet", status))?
            .is_some()
        {
            return Err(storage_protocol_error(
                "storage returned multiple responses for one address",
            ));
        }
        let fragment = response.fragment.ok_or_else(|| {
            storage_protocol_error("storage response is missing fragment metadata")
        })?;
        if response.payload.len() != fragment.size_payload as usize {
            return Err(storage_protocol_error(
                "storage payload length does not match fragment metadata",
            ));
        }
        Ok((fragment, response.payload.to_vec()))
    }

    /// Look up a repository before constructing a repository-scoped client.
    pub async fn get_repository_by_name(
        &self,
        name: &str,
    ) -> Result<proto_gen::lore::model::v1::Repository, NapError> {
        let mut client = self.make_repository_client();
        client
            .repository_get(RepositoryGetRequest {
                query: Some(
                    proto_gen::lore::repository::v1::repository_get_request::Query::Name(
                        name.to_string(),
                    ),
                ),
            })
            .await
            .map_err(|status| map_grpc_status("RepositoryGet", status))?
            .into_inner()
            .repository
            .ok_or_else(|| {
                NapError::GrpcError(format!("RepositoryGet({name}) returned no repository"))
            })
    }

    /// Return names of repositories visible to the current identity.
    pub async fn list_repositories(&self) -> Result<Vec<String>, NapError> {
        let mut client = self.make_repository_client();
        let mut stream = client
            .repository_list(RepositoryListRequest { creator: None })
            .await
            .map_err(|status| map_grpc_status("RepositoryList", status))?
            .into_inner();
        let mut names = Vec::new();
        while let Some(item) = stream
            .message()
            .await
            .map_err(|status| map_grpc_status("RepositoryList", status))?
        {
            if let Some(repository) = item.repository {
                names.push(repository.name);
            }
        }
        Ok(names)
    }

    /// Read a single file at a revision tree path. The caller must scope this
    /// client with the repository id returned by `RepositoryGet`.
    pub async fn read_file_at_revision(
        &self,
        identifier: proto_gen::lore::model::v1::RevisionIdentifier,
        path: String,
    ) -> Result<(Vec<u8>, Vec<u8>), NapError> {
        let mut tree = self.make_thin_client();
        let mut stream = tree
            .revision_tree(Self::revision_tree_for_file(
                RevisionTreeQuery::Identifier(identifier),
                &path,
            ))
            .await
            .map_err(|status| map_grpc_status("RevisionTree", status))?
            .into_inner();
        let mut signature = Vec::new();
        let mut address = None;
        while let Some(item) = stream
            .message()
            .await
            .map_err(|status| map_grpc_status("RevisionTree", status))?
        {
            match item.payload {
                Some(
                    proto_gen::lore::thin_client::v1::revision_tree_response::Payload::Header(
                        header,
                    ),
                ) => signature = header.signature.to_vec(),
                Some(proto_gen::lore::thin_client::v1::revision_tree_response::Payload::Node(
                    node,
                )) if node.path == path => address = node.address,
                _ => {}
            }
        }
        let address = address.ok_or_else(|| NapError::ManifestNotFound(path.clone()))?;
        let bytes = self.read_storage_content(address).await?;
        Ok((bytes, signature))
    }

    /// Read a file selected by its immutable Lore revision signature.
    pub async fn read_file_at_signature(
        &self,
        signature: Vec<u8>,
        path: String,
    ) -> Result<(Vec<u8>, Vec<u8>), NapError> {
        let mut tree = self.make_thin_client();
        let mut stream = tree
            .revision_tree(Self::revision_tree_for_file(
                RevisionTreeQuery::Signature(signature.into()),
                &path,
            ))
            .await
            .map_err(|status| map_grpc_status("RevisionTree", status))?
            .into_inner();
        let mut resolved_signature = Vec::new();
        let mut address = None;
        while let Some(item) = stream
            .message()
            .await
            .map_err(|status| map_grpc_status("RevisionTree", status))?
        {
            match item.payload {
                Some(
                    proto_gen::lore::thin_client::v1::revision_tree_response::Payload::Header(
                        header,
                    ),
                ) => resolved_signature = header.signature.to_vec(),
                Some(proto_gen::lore::thin_client::v1::revision_tree_response::Payload::Node(
                    node,
                )) if node.path == path => address = node.address,
                _ => {}
            }
        }
        let address = address.ok_or(NapError::ManifestNotFound(path))?;
        let bytes = self.read_storage_content(address).await?;
        Ok((bytes, resolved_signature))
    }

    /// List file paths below a revision tree prefix without downloading them.
    pub async fn list_paths_at_revision(
        &self,
        identifier: proto_gen::lore::model::v1::RevisionIdentifier,
        prefix: String,
    ) -> Result<Vec<String>, NapError> {
        let mut tree = self.make_thin_client();
        let response = tree
            .revision_tree(RevisionTreeRequest {
                query: Some(
                    proto_gen::lore::thin_client::v1::revision_tree_request::Query::Identifier(
                        identifier,
                    ),
                ),
                path_prefix: Some(prefix),
                max_depth: None,
            })
            .await;
        let mut stream = match response {
            Ok(response) => response.into_inner(),
            // Lore represents a repository with no committed revisions as a
            // zero signature. Listing an empty repository is still valid;
            // surface it as an empty list rather than leaking this server
            // implementation detail to NAP users.
            Err(status)
                if status.code() == tonic::Code::InvalidArgument
                    && status.message().contains("zeroed revision") =>
            {
                return Ok(Vec::new());
            }
            Err(status) => return Err(map_grpc_status("RevisionTree", status)),
        };
        let mut paths = Vec::new();
        while let Some(item) = stream
            .message()
            .await
            .map_err(|status| map_grpc_status("RevisionTree", status))?
        {
            if let Some(proto_gen::lore::thin_client::v1::revision_tree_response::Payload::Node(
                node,
            )) = item.payload
                && node.node_type == proto_gen::lore::thin_client::v1::NodeType::File as i32
            {
                paths.push(node.path);
            }
        }
        Ok(paths)
    }

    /// Look up a file's content address without downloading its payload.
    pub async fn file_address_at_revision(
        &self,
        identifier: proto_gen::lore::model::v1::RevisionIdentifier,
        path: String,
    ) -> Result<(proto_gen::lore::model::v1::Address, Vec<u8>), NapError> {
        let mut tree = self.make_thin_client();
        let mut stream = tree
            .revision_tree(Self::revision_tree_for_file(
                RevisionTreeQuery::Identifier(identifier),
                &path,
            ))
            .await
            .map_err(|status| map_grpc_status("RevisionTree", status))?
            .into_inner();
        let mut signature = Vec::new();
        let mut address = None;
        while let Some(item) = stream
            .message()
            .await
            .map_err(|status| map_grpc_status("RevisionTree", status))?
        {
            match item.payload {
                Some(
                    proto_gen::lore::thin_client::v1::revision_tree_response::Payload::Header(
                        header,
                    ),
                ) => signature = header.signature.to_vec(),
                Some(proto_gen::lore::thin_client::v1::revision_tree_response::Payload::Node(
                    node,
                )) if node.path == path => address = node.address,
                _ => {}
            }
        }
        address
            .map(|address| (address, signature))
            .ok_or(NapError::ManifestNotFound(path))
    }

    pub async fn revision_info_at_signature(
        &self,
        signature: Vec<u8>,
    ) -> Result<proto_gen::lore::thin_client::v1::Revision, NapError> {
        let mut client = self.make_thin_client();
        client
            .revision_info(RevisionInfoRequest {
                query: Some(
                    proto_gen::lore::thin_client::v1::revision_info_request::Query::Signature(
                        signature.into(),
                    ),
                ),
            })
            .await
            .map_err(|status| map_grpc_status("RevisionInfo", status))?
            .into_inner()
            .revision
            .ok_or_else(|| NapError::GrpcError("RevisionInfo returned no revision".to_string()))
    }
}

/// Return the directory that contains a repository-relative file path.
///
/// Lore's `RevisionTree` RPC accepts directory prefixes, including the empty
/// repository root, but rejects a file path as a prefix.
fn parent_tree_path(path: &str) -> String {
    path.rsplit_once('/')
        .map_or_else(String::new, |(parent, _)| parent.to_string())
}

const FRAGMENT_PAYLOAD_FRAGMENTED: u32 = 1;
const FRAGMENT_PAYLOAD_COMPRESSED_LZ4: u32 = 1 << 1;
const FRAGMENT_PAYLOAD_COMPRESSED_OODLE: u32 = 1 << 2;
const FRAGMENT_PAYLOAD_COMPRESSED_ZSTD: u32 = 1 << 3;
const FRAGMENT_PAYLOAD_COMPRESSED: u32 = 0b1111_1110;
const FRAGMENT_REFERENCE_SIZE: usize = 40;

#[derive(Debug, PartialEq, Eq)]
struct FragmentReference {
    hash: Vec<u8>,
    offset: u64,
}

fn storage_protocol_error(message: impl Into<String>) -> NapError {
    NapError::GrpcError(format!(
        "StorageGet (invalid Lore fragment): {}",
        message.into()
    ))
}

fn decode_fragment_references(payload: &[u8]) -> Result<Vec<FragmentReference>, NapError> {
    if payload.is_empty() || !payload.len().is_multiple_of(FRAGMENT_REFERENCE_SIZE) {
        return Err(storage_protocol_error(
            "fragmented payload is not a list of references",
        ));
    }
    Ok(payload
        .chunks_exact(FRAGMENT_REFERENCE_SIZE)
        .map(|chunk| FragmentReference {
            hash: chunk[..32].to_vec(),
            offset: u64::from_le_bytes(chunk[32..].try_into().expect("fixed-size offset")),
        })
        .collect())
}

fn decode_fragment_payload(
    fragment: &proto_gen::lore::model::v1::Fragment,
    payload: &[u8],
) -> Result<Vec<u8>, NapError> {
    let expected_size = usize::try_from(fragment.size_content)
        .map_err(|_| storage_protocol_error("fragment content size exceeds platform limits"))?;
    let compression = fragment.flags & FRAGMENT_PAYLOAD_COMPRESSED;
    if compression == 0 {
        return Ok(payload.to_vec());
    }
    if compression.count_ones() != 1 {
        return Err(storage_protocol_error(
            "fragment has incompatible compression flags",
        ));
    }
    if compression == FRAGMENT_PAYLOAD_COMPRESSED_ZSTD {
        let mut decoded = vec![0; expected_size];
        // The allocated output buffer is exactly `expected_size` bytes and
        // the input pointer/length come from the gRPC payload slice.
        let decoded_size = unsafe {
            zstd_sys::ZSTD_decompress(
                decoded.as_mut_ptr().cast(),
                decoded.len(),
                payload.as_ptr().cast(),
                payload.len(),
            )
        };
        // `ZSTD_isError` only inspects the return value from Zstd.
        if unsafe { zstd_sys::ZSTD_isError(decoded_size) } != 0 || decoded_size != expected_size {
            return Err(storage_protocol_error("Zstd decompression failed"));
        }
        return Ok(decoded);
    }
    let codec = match compression {
        FRAGMENT_PAYLOAD_COMPRESSED_LZ4 => "LZ4",
        FRAGMENT_PAYLOAD_COMPRESSED_OODLE => "Oodle",
        _ => "an unknown",
    };
    Err(storage_protocol_error(format!(
        "encountered {codec}-compressed content, which this client cannot decode"
    )))
}

#[cfg(test)]
mod tests {
    use super::{
        FRAGMENT_PAYLOAD_FRAGMENTED, FragmentReference, LoreGrpcClient, RevisionTreeQuery,
        decode_fragment_references, parent_tree_path,
    };
    use crate::grpc_client::proto_gen::lore::model::v1::RevisionIdentifier;

    #[test]
    fn revision_tree_for_file_uses_a_parent_directory_and_one_level_walk() {
        let request = LoreGrpcClient::revision_tree_for_file(
            RevisionTreeQuery::Identifier(RevisionIdentifier {
                branch_id: vec![1, 2, 3].into(),
                number: 0,
            }),
            "character/nathan-gunn.yaml",
        );
        assert_eq!(request.path_prefix.as_deref(), Some("character"));
        assert_eq!(request.max_depth, Some(1));
        assert_eq!(parent_tree_path("character/nathan-gunn.yaml"), "character");
        assert_eq!(
            parent_tree_path("assets/portraits/nathan.png"),
            "assets/portraits"
        );
        assert_eq!(parent_tree_path("repository.yaml"), "");
    }

    #[test]
    fn fragmented_storage_payload_decodes_lore_reference_layout() {
        let mut payload = vec![7; 32];
        payload.extend_from_slice(&512_u64.to_le_bytes());
        let references = decode_fragment_references(&payload).expect("valid reference payload");
        assert_eq!(
            references,
            vec![FragmentReference {
                hash: vec![7; 32],
                offset: 512
            }]
        );
        assert_ne!(FRAGMENT_PAYLOAD_FRAGMENTED, 0);
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
