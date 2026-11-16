//! What a verb writes into the corpus (ADR-3c8d94a0bbe0).
//!
//! Here rather than beside the code that writes, because it is a field of
//! [`crate::CommandSpec`] for the reason [`crate::Renews`] is one: it is part of
//! what a verb *is*, and a surface describing the verbs describes it.

/// Where a verb leaves bytes, as far as the corpus is concerned.
///
/// The rule ADR-3c8d94a0bbe0 carries was once stated with two enumerations,
/// the reading verbs and the writing verbs, and they went stale once per new
/// route: `archive` and `migrate` wrote `.ank/` and sat on neither list. **A field is how the
/// compiler asks the question of every verb that is ever added**, and the
/// workspace suite confronts each verb declaring [`Writes::Nothing`] with what
/// it leaves on a scratch corpus, so the declaration cannot go stale silently
/// either.
///
/// The domain is the corpus and nothing else: a path under `.ank/`, and a ref
/// under `refs/ank/`. `skills --install` writes a directory elsewhere and
/// `update` replaces the binary, and neither touches a corpus. A verb that can
/// write on some path declares that path: `log` reads with an id alone and
/// writes with a message, and declares the write.
///
/// **A lease renewal is not counted here.** `show` on the task the caller holds
/// moves its claim ref, and [`crate::Renews`] already declares that per verb
/// (ADR-0bb7ea8991bc); counting it twice would put every verb that looks at the
/// held task on the writing side and leave the reading side meaning nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Writes {
    /// Neither: the verb reads the corpus, or works outside it.
    Nothing,
    /// Files under `.ank/` only.
    Files,
    /// Refs under `refs/ank/` only.
    Refs,
    /// Both, on one path or across its paths.
    Both,
}

impl Writes {
    /// The word `help --json` carries.
    pub fn as_str(self) -> &'static str {
        match self {
            Writes::Nothing => "nothing",
            Writes::Files => "files",
            Writes::Refs => "refs",
            Writes::Both => "both",
        }
    }
}

/// What a verb writes **outside** the corpus (GitHub #781).
///
/// [`Writes`] answers for `.ank/` and `refs/ank/` and nothing else, which is
/// the meaning ADR-3c8d94a0bbe0 gives it and keeps. That left a verb like
/// `hooks install`, which rewrites the harness's `settings.json`, on the
/// reading side by construction, and a surface deriving "read-only" from
/// [`Writes::Nothing`] would have told a client so. This is the second
/// question, asked of every verb by the same compiler: does it leave bytes
/// anywhere that is not the corpus -- the reader's configuration directory, an
/// agent's settings, a directory handed to another program, the binary itself.
///
/// **Not the verb's scratch.** A temporary file removed before the verb exits,
/// the verifier output `done` reads back, and the build a verifier runs are
/// not counted: the first two are gone when the verb answers, and the third is
/// the declared verifier's act, which the task declared and not the verb. Nor
/// a verb's own index, which `writes` already ignores for the same reason.
///
/// **A verb that runs other verbs declares what they may write**, as
/// [`Writes`] does for `tui` and `mcp`: a reader of the table asking whether a
/// call can leave something behind gets the answer the call can actually give.
///
/// `crates/ank-cli/tests/writes.rs` runs every verb declaring
/// [`Outside::Nothing`] under a scratch home and scratch XDG directories and
/// fails on what it leaves there, and runs the declared writers a scratch
/// machine can run and fails if they leave nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outside {
    /// The verb leaves nothing outside the corpus.
    Nothing,
    /// The verb writes files outside the corpus on some path through it.
    Files,
}

impl Outside {
    /// The word `help --json` carries.
    pub fn as_str(self) -> &'static str {
        match self {
            Outside::Nothing => "nothing",
            Outside::Files => "files",
        }
    }
}
