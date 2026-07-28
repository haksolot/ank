//! ankor-core — parseur et modele de donnees du format Ankor.
//!
//! « Le format est la spec » : ce crate est l'implementation de reference
//! du format, independante du CLI. Il porte quatre responsabilites :
//!
//! 1. Lire et ecrire les fichiers `.ankor/` avec round-trip a l'identique
//!    ([`parse_entity`], [`serialize_entity`]) ;
//! 2. Les invariants du modele : identifiants ([`id`]), transitions de
//!    statut ([`model`]), scope obligatoire et valide ([`scope`]),
//!    blocage derive ;
//! 3. Le gel par hash des champs immuables ([`freeze`]) ;
//! 4. La section `## Log` append-only ([`log`]).
//!
//! Ce que ce crate ne fait pas, deliberement : pas d'E/S disque, pas de
//! git, pas d'index, pas de permissions — l'appelant (CLI, `check`, outil
//! tiers) compose ces briques.

pub mod error;
pub mod freeze;
pub mod id;
pub mod log;
pub mod model;
pub mod parse;
pub mod scope;

pub use error::{Error, Result};
pub use freeze::{freeze_hash, freeze_hash_short, verify_frozen};
pub use id::{resolve_prefix, EntityId, EntityKind};
pub use log::{append_log, parse_log, LogEntry};
pub use model::{
    Adr, AdrStatus, CriteriaBy, Entity, Proof, ProofType, Task, TaskStatus, SCHEMA_VERSION,
};
pub use parse::{
    parse_adr, parse_entity, parse_task, serialize_adr, serialize_entity, serialize_task,
};
pub use scope::ScopeSet;
