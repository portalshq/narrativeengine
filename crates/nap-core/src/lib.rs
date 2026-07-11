//! # NAP Core — Narrative Addressing Protocol
//!
//! Four primitives for entertainment infrastructure:
//! - **URI**: Identity — `nap://starwars/character/lukeskywalker`
//! - **Manifest**: Current state — YAML, human/machine/agent-readable
//! - **Commit**: History — snapshot + delta metadata
//! - **Resolver**: Resolution — URI → Manifest, with query/version selectors

pub mod commit;
pub mod content;
pub mod context_docs;
pub mod error;
pub mod grpc_client;
pub mod manifest;
pub mod merge;
pub mod permission_gate;
pub mod provider;
pub mod query;
pub mod repo_service;
pub mod repository;
pub mod repository_api;
pub mod resolver;
pub mod schema;
pub mod server;
pub mod storage;
#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
pub mod types;
pub mod uri;
pub mod validation;
pub mod vcs;

// Lore VCS backend (production).
pub mod vcs_lore;

// Re-exports for ergonomic top-level usage
pub use commit::{Change, ChangeOp, Commit};
pub use content::ContentHash;
pub use context_docs::ContextDocsManager;
pub use error::NapError;
pub use manifest::{Manifest, Principal, Provenance, Representation};
pub use merge::{
    conflict::{Conflict, ConflictType, MergeResult},
    diff::{Change as DiffChange, ChangeOp as DiffChangeOp, DiffResult, diff, diff_normalized},
    merge_engine::MergeEngine,
    normalization::normalize,
    path::CanonicalPath,
    sdl::{
        IdentityRule, MergeStrategyDef, MergeStrategyType, PropertyDef, PropertyType, SdlDocument,
        SdlError,
    },
    strategies,
};
pub use permission_gate::PermissionGate;
pub use provider::{Provider, ProviderFactory, ProviderManager, ProviderType};
pub use query::ManifestQuery;
pub use repo_service::RepoService;
pub use repository::Repository;
pub use repository_api::{RepositoryApi, RepositoryHandle};
pub use resolver::{ResolveConfig, ResolveOptions, Resolver};
pub use server::PINNED_LORE_VERSION;
pub use server::{
    LoreInstaller, LoreProcessManager, LoreVersionInfo, NapDoctor, ServerManager,
    generate_certificates, generate_local_config, verify_lore_installation,
};
pub use storage::{StorageBackend, StorageConfig, StorageEngine, StorageError, get_engine};
pub use types::EntityType;
pub use uri::NapUri;
pub use vcs::{AccessLevel, CommitInfo, ContextDocument, Permission, Revision, VcsBackend};
pub use vcs_lore::LoreBackend;
