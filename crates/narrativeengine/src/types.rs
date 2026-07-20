//! Core narrative data types.
//!
//! Mirrors generated Protobuf structures for wire compatibility,
//! with extension traits for internal business logic.

pub use crate::narrative::v1::{BaseNarrativeBlock, BaseNarrativeLore};
use serde::{Deserialize, Serialize};

/// Internal representation of a block or lore entry ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockId {
    Num(i64),
    Str(String),
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockId::Num(n) => write!(f, "{n}"),
            BlockId::Str(s) => write!(f, "{s}"),
        }
    }
}

impl From<crate::narrative::v1::BlockId> for BlockId {
    fn from(proto: crate::narrative::v1::BlockId) -> Self {
        match proto.id {
            Some(crate::narrative::v1::block_id::Id::Num(n)) => BlockId::Num(n),
            Some(crate::narrative::v1::block_id::Id::Str(s)) => BlockId::Str(s),
            None => BlockId::Num(0),
        }
    }
}

impl From<BlockId> for crate::narrative::v1::BlockId {
    fn from(id: BlockId) -> Self {
        match id {
            BlockId::Num(n) => Self {
                id: Some(crate::narrative::v1::block_id::Id::Num(n)),
            },
            BlockId::Str(s) => Self {
                id: Some(crate::narrative::v1::block_id::Id::Str(s)),
            },
        }
    }
}

impl BlockId {
    pub fn as_num(&self) -> Option<i64> {
        match self {
            BlockId::Num(n) => Some(*n),
            _ => None,
        }
    }
}

// Extension methods for business logic
pub trait NarrativeBlockExt {
    fn is_notable(&self) -> bool;
    fn block_id(&self) -> BlockId;
}

impl NarrativeBlockExt for BaseNarrativeBlock {
    fn is_notable(&self) -> bool {
        self.is_notable.unwrap_or(false)
    }

    fn block_id(&self) -> BlockId {
        self.id
            .clone()
            .map(BlockId::from)
            .unwrap_or(BlockId::Num(0))
    }
}

pub trait NarrativeLoreExt {
    fn is_active(&self) -> bool;
    fn lore_id(&self) -> BlockId;
}

impl NarrativeLoreExt for BaseNarrativeLore {
    fn is_active(&self) -> bool {
        self.is_active.unwrap_or(true)
    }

    fn lore_id(&self) -> BlockId {
        self.id
            .clone()
            .map(BlockId::from)
            .unwrap_or(BlockId::Num(0))
    }
}
