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
//! 4. The log ([`log`]): the line a reader sees, the rule that splits a
//!    message between the two fields an entry stores it in, and the two
//!    previous layouts, read for one window and never written.
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
pub use log::{
    body_remainder, message_fields, message_of, parse_log, parse_log_file, split_message, LogEntry,
};
pub use model::{
    Adr, AdrStatus, CriteriaBy, Entity, Log, Proof, ProofType, ProofVia, Spec, SpecStatus, Task,
    TaskStatus, Verified, MIN_SCHEMA, SCHEMA_VERSION,
};
pub use parse::{
    has_crlf, normalise_line_endings, parse_adr, parse_entity, parse_log_entity, parse_spec,
    parse_task, serialize_adr, serialize_entity, serialize_log_entity, serialize_spec,
    serialize_task,
};
pub use registry::{FieldSpec, Fields, KindSpec, KINDS};
pub use scope::{normalize_path, ScopeSet};
