//! The exit codes of §4, as one type.
//!
//! The semantics are carried by the code so that a shell can route without
//! parsing output, which makes the code part of the contract rather than a
//! detail of an error path. Before this crate they were bare `i32` literals at
//! two hundred and fifteen call sites, and the table they belonged to existed
//! only in the specification — so the surface's most machine-readable half was
//! the one half nothing held still.
//!
//! **Two of them are the ones an agentic loop must handle** (§4).
//! [`ExitCode::Conflict`] literally means "redo `context`, somebody moved", and
//! [`ExitCode::Unavailable`] means "take something else". The rest are read by a
//! pipeline, and the pipeline's whole reason for reading a number instead of a
//! message is that the number does not change wording.

use std::fmt;

/// One exit code of the surface (§4).
///
/// The discriminants are the codes, and they are written out rather than left
/// to the compiler: they are the contract, and a variant inserted in the middle
/// must not renumber the ones after it.
///
/// The variants are named for what the *caller* is being told, which is why
/// [`Self::Prerequisite`] and [`Self::Transition`] are two codes and not one.
/// §4 draws that line explicitly — `accept` off the default branch is a missing
/// prerequisite and "not illegal transition: the promotion is legal, the place
/// is not" — and a caller that conflates them reacts wrongly to one of the two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum ExitCode {
    /// The verb answered.
    Ok = 0,
    /// Generic error, and §4 calls it that. It is the code for a call the
    /// parser refuses and for a file the tool cannot make sense of — the cases
    /// with no reaction of their own to prescribe. Naming it for a narrower
    /// meaning would be inventing a precision the ninety-nine sites carrying it
    /// do not have.
    Generic = 1,
    /// No such entity, or a prefix matching more than one. The hint names the
    /// `find` or the `show` that resolves it.
    NotFound = 2,
    /// Version conflict: the entity moved under the caller. §4: "redo
    /// `context`, somebody moved".
    Conflict = 3,
    /// The task is not to be taken — held by another agent, or already finished
    /// on another branch. §4 unites the two causes under one code because they
    /// call for the same reaction, and the message says which task to take
    /// instead.
    Unavailable = 4,
    /// A proof is missing, malformed, or of a type this act does not accept
    /// (§8).
    Proof = 5,
    /// The act is not legal from the state the entity is in: a frozen field
    /// diverged from its anchoring hash, a transition the state machine does
    /// not allow, or a write attempted without holding the claim it needs.
    ///
    /// Distinct from [`Self::Prerequisite`], and §4 is explicit about the line:
    /// here the caller is asking for something the state forbids; there the
    /// thing asked for is legal and something it depends on is absent.
    Transition = 6,
    /// A prerequisite is missing: the task is blocked, it has no
    /// `done_criteria`, a mandatory flag was not given, `accept` was run off
    /// the default branch, or the calling identity already holds a live claim.
    ///
    /// §4 chose this code over [`Self::Unavailable`] for that last case on
    /// purpose: "the task asked for is available; it is the caller that is not",
    /// and 4 would have said "take something else" when the reaction called for
    /// is the opposite.
    Prerequisite = 7,
    /// `check` and `review` found something. Never [`Self::Generic`], so that a
    /// pipeline can tell a sick corpus from a broken tool (§4). A signal alone
    /// leaves the code at [`Self::Ok`]; only a fault reaches this.
    Findings = 8,
    /// An environment to repair rather than work that failed: `sh` or `git`
    /// absent, git older than 2.34, `$EDITOR` unset, a default branch that
    /// cannot be determined, a detached proof whose ref never reached the
    /// remote, a directory that refuses the lock.
    ///
    /// §4 keeps this apart from [`Self::Generic`] and [`Self::Proof`] for one
    /// reason: none of these is a failure of the agent's work, and confusing
    /// them would send an agent to fix sound code.
    Environment = 9,
}

impl ExitCode {
    /// Every code, in order: the table `docs/exit-codes.md` is generated from
    /// (ADR-2b62b9a1fe67), by `cargo run -p ank-contract --bin exit-codes`.
    ///
    /// [`Self::meaning`] is an exhaustive match, so a new variant cannot land
    /// without a line of its own; the list is what orders the rows, and
    /// `tests/exit_codes_page.rs` names the ten codes it expects on the page.
    pub const ALL: [ExitCode; 10] = [
        ExitCode::Ok,
        ExitCode::Generic,
        ExitCode::NotFound,
        ExitCode::Conflict,
        ExitCode::Unavailable,
        ExitCode::Proof,
        ExitCode::Transition,
        ExitCode::Prerequisite,
        ExitCode::Findings,
        ExitCode::Environment,
    ];

    /// What the code tells the caller, in one line, as the reference page
    /// prints it. The variant documentation above is the long form.
    pub const fn meaning(self) -> &'static str {
        match self {
            ExitCode::Ok => "the verb answered",
            ExitCode::Generic => {
                "generic error: a call the parser refuses, a file the tool cannot make sense of"
            }
            ExitCode::NotFound => "no such entity, or a prefix matching more than one",
            ExitCode::Conflict => {
                "version conflict: the entity moved under the caller, redo `context`"
            }
            ExitCode::Unavailable => {
                "the task is unavailable: held by another agent, or finished on another branch; take something else"
            }
            ExitCode::Proof => {
                "a proof is missing, malformed, or of a type this act does not accept"
            }
            ExitCode::Transition => {
                "the act is illegal from the state the entity is in: a frozen field diverged, a transition the state machine does not allow, or a write without the claim it needs"
            }
            ExitCode::Prerequisite => {
                "a prerequisite is missing: the task is blocked, it has no `done_criteria`, a mandatory flag was not given, `accept` ran off the default branch, or the caller already holds a live claim"
            }
            ExitCode::Findings => {
                "`check` or `review` found a fault; a signal alone leaves the code at 0"
            }
            ExitCode::Environment => {
                "the environment, not the work: `sh` or `git` absent, git older than 2.34, `$EDITOR` unset, a default branch that cannot be determined, a detached proof that never reached the remote, a directory that refuses the lock"
            }
        }
    }

    /// The integer a process exits with.
    ///
    /// `const` so a table or a test can name it where a function call would not
    /// be allowed.
    pub const fn code(self) -> i32 {
        self as i32
    }
}

impl fmt::Display for ExitCode {
    /// The integer, and nothing else.
    ///
    /// This is not a convenience: the code's only rendering anywhere in the
    /// tool is the one §4 fixes, `error[7]:` on standard error. A `Display` that
    /// printed a variant name would be a second rendering of a value whose whole
    /// purpose is to have exactly one.
    ///
    /// **`--json` prints nothing at all on a refusal**, which is the opposite of
    /// what this comment claimed until TASK-78431b544d01 measured it. Six
    /// refusals across six verbs, each run twice to separate the streams:
    /// `show TASK-9999 --json` (2), `claim TASK-9999 --json` (2), `attest <id>
    /// --proof bogus --json` (5), `done --json` (5), `new task --json` (7) and
    /// `config nosuchkey --json` (1) each left **zero bytes** on standard
    /// output and put `error[<code>]: <message>` on standard error. That is §4
    /// holding: a refusal is not a document, so nothing is written where a
    /// parser reads, and the code reaches the caller as the process status and
    /// in the line beside it. A caller that waits for a `"code"` field waits
    /// forever — the tool emits that key in exactly two places, neither of them
    /// an error path: the `refuses` array of `ank help --json`, where it is the
    /// code a verb *would* return, and the MCP server's error envelope, which
    /// is a protocol of its own (ADR-fd98).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// The exit-code reference page, as `docs/exit-codes.md` carries it.
///
/// Rendered from [`ExitCode::ALL`] and [`ExitCode::meaning`], so the page is
/// never typed (ADR-2b62b9a1fe67): the `exit-codes` binary prints this, and
/// the workspace suite fails when the committed page differs from it.
pub fn reference_page() -> String {
    let mut page = String::from(
        "<!-- Generated from crates/ank-contract/src/exit.rs; do not edit.\n     \
         Regenerate: cargo run -q -p ank-contract --bin exit-codes > docs/exit-codes.md -->\n\n\
         # Exit codes\n\n\
         Every ank verb exits with one of these codes, and the code carries the meaning so a \
         script can route without parsing output. They are stable. `ank help --json` publishes \
         which verb returns which, under each verb's `refuses`.\n\n\
         A refusal writes `error[<code>]: <message>` on stderr, followed by the exact command to \
         run next; under `--json` it leaves stdout empty.\n\n\
         | Code | Name | Meaning |\n\
         |---|---|---|\n",
    );
    for code in ExitCode::ALL {
        page.push_str(&format!(
            "| {} | `{:?}` | {} |\n",
            code.code(),
            code,
            code.meaning()
        ));
    }
    page.push_str(
        "\nTwo of them are the ones an agentic loop must handle: **3** means somebody moved, \
         read again, and **4** means take something else. **6 and 7 are two codes on purpose**: \
         in 6 the state forbids what was asked; in 7 the thing asked for is legal and something \
         it depends on is absent. **9 is not a failure of the work**, and a pipeline that \
         collapses it into \"the command failed\" sends somebody to fix sound code.\n",
    );
    page
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The codes are the contract, so the test that pins them names every one.
    ///
    /// Written as a list rather than derived from the enum: a test that read the
    /// discriminants back off the type would agree with any renumbering,
    /// including one nobody meant.
    #[test]
    fn the_codes_are_the_table_of_section_four() {
        for (variant, code) in [
            (ExitCode::Ok, 0),
            (ExitCode::Generic, 1),
            (ExitCode::NotFound, 2),
            (ExitCode::Conflict, 3),
            (ExitCode::Unavailable, 4),
            (ExitCode::Proof, 5),
            (ExitCode::Transition, 6),
            (ExitCode::Prerequisite, 7),
            (ExitCode::Findings, 8),
            (ExitCode::Environment, 9),
        ] {
            assert_eq!(variant.code(), code, "{variant:?} moved");
            assert_eq!(variant.to_string(), code.to_string(), "{variant:?} renders");
        }
    }
}
