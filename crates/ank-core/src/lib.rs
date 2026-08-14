//! ank-core — parser and data model for the Ank format.
//!
//! "The format is the specification": this crate is the reference
//! implementation of the format, independent of the CLI. It has four
//! responsibilities:
//!
//! 1. Read and write `.ank/` files with an identical round-trip
//!    ([`parse_entity`], [`serialize_entity`]), from one kind registry
//!    ([`registry`]) that declares each kind once;
//! 2. The model invariants: identifiers ([`id`]), status transitions
//!    ([`model`]), a mandatory and valid scope ([`scope`]), derived blocking;
//! 3. The hash freeze of immutable fields ([`freeze`]);
//! 4. The append-only log ([`log`]), a file of its own since schema 3.
//!
//! What this crate deliberately does not do: no disk I/O, no git, no index,
//! no permissions — the caller (CLI, `check`, third-party tool) composes
//! these building blocks.

pub mod error;
pub mod freeze;
pub mod id;
pub mod log;
pub mod model;
pub mod parse;
pub mod registry;
pub mod scope;

pub use error::{Error, Result};
pub use freeze::{freeze_hash, freeze_hash_short, verify_frozen};
pub use id::{resolve_prefix, EntityId, EntityKind};
pub use log::{append_log, append_log_file, parse_log, parse_log_file, LogEntry};
pub use model::{
    Adr, AdrStatus, CriteriaBy, Entity, Proof, ProofType, ProofVia, Task, TaskStatus, Verified,
    MIN_SCHEMA, SCHEMA_VERSION,
};
pub use parse::{
    has_crlf, normalise_line_endings, parse_adr, parse_entity, parse_task, serialize_adr,
    serialize_entity, serialize_task,
};
pub use registry::{FieldSpec, Fields, KindSpec, KINDS};
pub use scope::{normalize_path, ScopeSet};
