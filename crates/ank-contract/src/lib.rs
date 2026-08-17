//! The machine contract: what every surface of Ank consumes (ADR-6fd69efb629c).
//!
//! One crate holds the verb table, the flag and refusal types, and the exit
//! codes, so that **no surface can describe a verb the CLI does not dispatch,
//! and none can drift from it**. That is the whole reason this crate exists,
//! and it is a stronger guarantee than review: a verb the table does not carry
//! is a verb no surface has, and a verb it carries is one they all have.
//!
//! The drift it prevents is not hypothetical. A verb table written twice is a
//! verb table that will disagree with itself, and the disagreement does not
//! land on whoever wrote the second copy — it lands, days later, on whoever
//! wrote a client against it, as a bug they cannot see from their side.
//!
//! **Nothing here decides anything.** The table says what the surface *is*; it
//! never says what a verb does. `ank-cli` parses against it, dispatches from
//! it, and renders `help` out of it; ADR-372b82af1ec7's protocol surface is
//! generated from the same table for the same reason. A reader who wants the
//! description rather than the types has `ank help --json`, which is generated
//! from exactly what is here.
//!
//! The two enums are here rather than in the CLI because they are the halves a
//! caller needs before it calls: [`ExitCode`] is what a refusal *means*, and
//! [`Renews`] is a property of a verb the compiler has to ask of every verb
//! ever added (§3, ADR-0bb7ea8991bc).

pub mod exit;
pub mod renews;
pub mod verbs;

pub use exit::ExitCode;
pub use renews::Renews;
pub use verbs::{
    find_flag, known_flags, long_of, short_of, spec_of, usage, CommandSpec, FlagSpec, Refusal,
    COMMANDS, GLOBAL_FLAGS, GROUPS, SHORT_FORMS,
};
