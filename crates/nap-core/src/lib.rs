//! # NAP Core — Narrative Addressing Protocol
//!
//! Four primitives for entertainment infrastructure:
//! - **URI**: Identity — `nap://starwars/character/lukeskywalker`
//! - **Manifest**: Current state — YAML, human/machine/agent-readable
//! - **Commit**: History — snapshot + delta metadata
//! - **Resolver**: Resolution — URI → Manifest, with query/version selectors

pub mod commit;
pub mod content;
pub mod error;
pub mod manifest;
pub mod query;
pub mod repository;
pub mod resolver;
pub mod schema;
pub mod types;
pub mod uri;
pub mod vcs;
pub mod vcs_git;

// Re-exports for ergonomic top-level usage
pub use commit::{Change, ChangeOp, Commit};
pub use content::ContentHash;
pub use error::NapError;
pub use manifest::{Manifest, Principal, Provenance, Representation};
pub use query::ManifestQuery;
pub use repository::Repository;
pub use resolver::{ResolveOptions, Resolver};
pub use types::EntityType;
pub use uri::NapUri;
pub use vcs::VcsBackend;
pub use vcs_git::GitBackend;
