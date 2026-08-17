//! Whether a verb renews the caller's lease (§3, ADR-0bb7ea8991bc).
//!
//! Here rather than beside the renewal code it drives, because it is a field of
//! [`crate::CommandSpec`]: it is part of what a verb *is*, and a surface
//! describing the verbs describes it. The act of renewing stays in `ank-cli`,
//! which is the only thing that touches a ref.

/// What a verb is about, as far as the lease is concerned (§3).
///
/// §3 renewed the lease on `log` alone, and `log` is *reporting* rather than
/// working: after the design is settled there is often an hour of mechanical
/// fixing with nothing worth logging, so the lease lapsed precisely during the
/// stretch where the work was least interruptible. Renewal follows **the
/// holder's verbs against the task it holds** instead.
///
/// **Declared per verb on [`crate::CommandSpec`] and read by the renewal.** The
/// rule is "the holder's verbs against the held task" and not a list of verb
/// names, because a list beside the dispatch is what goes stale when a verb is
/// added — the same argument `coordinates` makes on the same table, and for the
/// same reason: a field makes the compiler ask the question of every verb that
/// is ever added, where a separate enumeration lets a new one default to
/// silence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Renews {
    /// Nothing. The verb is about the repository or the corpus rather than about
    /// a task — `status`, `find`, `check` — or it is one of the verbs that
    /// settles the lease itself: `claim` grants one rather than extending it,
    /// `log` renews as part of its own write and reports what the write turned
    /// up, and `done` and `release` end the claim.
    Never,
    /// The task its `<id>` names, and only when that is the one the caller
    /// holds. `ank show` on another task renews nothing.
    Named,
    /// The task the caller holds, which is the only one the verb is ever about:
    /// `context` in execution mode.
    Held,
}
