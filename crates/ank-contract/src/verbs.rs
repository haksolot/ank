//! The verb table: what the surface is, declared once (ADR-6fd69efb629c).
//!
//! [`COMMANDS`] is the single source of truth for the verbs, their flags, their
//! groups, their notes and the states they refuse on. `ank-cli` parses against
//! it, dispatches from it and renders `help` out of it; `ank help --json` is
//! that table serialised. Nothing here knows what a verb *does*.
//!
//! **The order is the specification's, and it is load-bearing**, which is why
//! this is a `const` slice and not a map: §4 puts the loop first, and a
//! container that sorted its own contents would erase what §4 says by holding
//! it.
//!
//! Moved here from `ank-cli` whole, so that the surface no surface may
//! contradict is a thing they consume rather than a thing each of them keeps a
//! copy of (TASK-0549e0f960ef).

use crate::exit::ExitCode;
use crate::renews::Renews;
use crate::shape::{f, one, opt, when, Field, Shape, Type};

// ---------------------------------------------------------------------------
// The documents the verbs return (ADR-6fd69efb629c)
// ---------------------------------------------------------------------------
//
// Named above the table rather than inlined into it: several are shared by two
// verbs, and a `CommandSpec` whose `output` ran to twenty lines would bury the
// twelve fields that say what the verb *is*.
//
// The `contract` field is on every document and is on none of these lists. It
// is added by the rendering, because it is universal by construction —
// `Obj::document` seeds it — and repeating it twenty-two times would invite the
// one copy that gets it wrong.

/// A row of `blocked_by` or `unblocks`. `status` and `title` are null where the
/// reference resolves to nothing: `show` still prints the edge it could not
/// resolve, because a shorter list is a wrong answer to "what blocks this".
const EDGE: &[Field] = &[
    f("id", Type::Str),
    f("short", Type::Str),
    opt("status", Type::Str),
    opt("title", Type::Str),
];

/// A proof recorded on a ref rather than in the file (§7, ADR-4934).
const DETACHED_PROOF: &[Field] = &[
    f("type", Type::Str),
    f("ref", Type::Str),
    f("by", Type::Str),
    f("at", Type::Str),
];

/// A log entry. `id` is null for an entry read out of the previous layout,
/// which has no entity of its own.
const LOG_ENTRY: &[Field] = &[
    opt("id", Type::Str),
    f("timestamp", Type::Str),
    f("who", Type::Str),
    f("message", Type::Str),
    // Absent on the work trace, which is what an entry is unless it says
    // otherwise (ADR-f7dc76886db2). Present, it names what the entry records.
    opt("records", Type::Str),
];

/// The opening frame of the reader, as data (§4, ADR-8bd76e8d7c4e).
///
/// `tui` draws a screen and, under `--json`, answers what that screen holds:
/// the corpus it opened, who holds what, and the rows the list carries. It is
/// the reader's own answer rather than a passthrough of the four verbs it runs,
/// which is why it has a shape of its own here.
const TUI_OUT: &[Field] = &[
    f("corpus", Type::Str),
    f("branch", Type::Str),
    f("default_branch", Type::Str),
    f("identity", Type::Str),
    f("total", Type::Num),
    f("shown", Type::Num),
    f(
        "claims",
        Type::Array(&[
            f("id", Type::Str),
            f("short", Type::Str),
            f("holder", Type::Str),
            f("expires", Type::Str),
            f("mine", Type::Bool),
        ]),
    ),
    f(
        "entities",
        Type::Array(&[
            f("id", Type::Str),
            f("short", Type::Str),
            f("kind", Type::Str),
            f("status", Type::Str),
            f("title", Type::Str),
        ]),
    ),
];

/// An entity row as `scope` and `find` render one.
const ROW: &[Field] = &[
    f("id", Type::Str),
    f("kind", Type::Str),
    f("status", Type::Str),
    f("title", Type::Str),
];

const SHOW_TASK: &[Field] = &[
    f("id", Type::Str),
    opt("coordination", Type::Str),
    f("blocked_by", Type::Array(EDGE)),
    f("unblocks", Type::Array(EDGE)),
    f("detached_proofs", Type::Array(DETACHED_PROOF)),
    f("log_total", Type::Num),
    f("log_shown", Type::Num),
    f("log", Type::Array(LOG_ENTRY)),
    f("machinery", Type::Array(LOG_ENTRY)),
    f("content", Type::Str),
];

const SHOW_OTHER: &[Field] = &[
    f("id", Type::Str),
    opt("coordination", Type::Str),
    f("detached_proofs", Type::Array(DETACHED_PROOF)),
    f("log_total", Type::Num),
    f("log_shown", Type::Num),
    f("log", Type::Array(LOG_ENTRY)),
    f("machinery", Type::Array(LOG_ENTRY)),
    f("content", Type::Str),
];

const CONTEXT_OUT: &[Field] = &[
    f("mode", Type::Str),
    opt("head", Type::Str),
    opt("criteria", Type::Str),
    f(
        "constraints",
        Type::Array(&[
            f("id", Type::Str),
            f("short", Type::Str),
            f("title", Type::Str),
            f("constraint", Type::Str),
            opt("home", Type::Str),
        ]),
    ),
    f(
        "proposed",
        Type::Array(&[
            f("id", Type::Str),
            f("short", Type::Str),
            f("title", Type::Str),
            opt("home", Type::Str),
        ]),
    ),
    f(
        "specs",
        Type::Array(&[
            f("id", Type::Str),
            f("short", Type::Str),
            f("title", Type::Str),
        ]),
    ),
    f(
        "tasks",
        Type::Array(&[
            f("id", Type::Str),
            f("short", Type::Str),
            f("title", Type::Str),
            f("status", Type::Str),
            f("ready", Type::Bool),
            f("unblocks", Type::Num),
            f("state", Type::Str),
        ]),
    ),
    f("log", Type::Strings),
    f("ready", Type::Num),
    f("blocked", Type::Num),
    f("finished_elsewhere", Type::Num),
    f("warnings", Type::Strings),
];

// The claim taken, and nothing about corpora the reader did not declare
// (ADR-ed3e14d0f991).
const CLAIM_OUT: &[Field] = &[
    f("task", Type::Str),
    f("holder", Type::Str),
    f("expires", Type::Str),
    f("warnings", Type::Strings),
];

/// The same document, plus the live claims this identity already holds in the
/// other corpora the reader declared in `corpora.yml` (ADR-ed3e14d0f991).
///
/// **A second document and not a field that is always there.** A document may
/// gain a field within a contract version, and gaining one here would still be
/// a byte a caller who declared no corpus sees that it did not see before —
/// which is the one thing the decision says such a caller must not pay. `log`
/// already declares two documents for two calls; this declares two for two
/// readers, and a client binds `claims_elsewhere` where it is present exactly
/// as it binds either shape of `log`.
const CLAIM_OUT_ELSEWHERE: &[Field] = &[
    f("task", Type::Str),
    f("holder", Type::Str),
    f("expires", Type::Str),
    f("warnings", Type::Strings),
    f(
        "claims_elsewhere",
        Type::Array(&[
            f("task", Type::Str),
            // Where the reader declared that corpus, never the repository
            // identity: the map is keyed on the tree a reader stands in
            // (ADR-96174f1ac2b7), which is not the corpus the claim is in.
            f("corpus", Type::Str),
            f("expires", Type::Str),
        ]),
    ),
];

const STATUS_OUT: &[Field] = &[
    // The first field gained under the contract, and the demonstration of what
    // CONTRACT_VERSION promises: a document may gain a field within a version and
    // may never lose, rename or retype one, so this arrives at 1 and the version
    // does not move (ADR-621a7fd96ce1, ADR-6fd69efb629c). A client written against
    // `status --json` yesterday keeps working, which is the whole of what the
    // breaking change was spent to buy.
    opt("corpus", Type::Str),
    opt("branch", Type::Str),
    opt("default_branch", Type::Str),
    f(
        "identity",
        Type::Object(&[f("value", Type::Str), f("source", Type::Str)]),
    ),
    opt(
        "claim",
        Type::Object(&[
            f("id", Type::Str),
            f("expires", Type::Str),
            f("lapsed", Type::Bool),
        ]),
    ),
    opt(
        "drift",
        Type::Object(&[f("branch", Type::Str), f("entities", Type::Num)]),
    ),
    f(
        "also_held",
        Type::Array(&[f("id", Type::Str), f("expires", Type::Str)]),
    ),
    f("remote", Type::Bool),
    // Null when `--remote` was not given, which is not zero: zero would say
    // this checkout's `refs/ank/*` match origin's, and without the flag nothing
    // was compared (TASK-6596aae0713c). `stale` is a ref both planes carry
    // pointing at different objects, `absent` a ref origin holds and this
    // checkout does not.
    opt(
        "refs",
        Type::Object(&[f("stale", Type::Num), f("absent", Type::Num)]),
    ),
    f(
        "elsewhere",
        Type::Array(&[
            f("id", Type::Str),
            opt("title", Type::Str),
            opt("holder", Type::Str),
            opt("expires", Type::Str),
            opt("seen", Type::Str),
        ]),
    ),
    f("constraints", Type::Num),
    f("queue", Type::Num),
    f("unmerged", Type::Num),
    f("faults", Type::Num),
    f("signals", Type::Num),
];

const CHECK_OUT: &[Field] = &[
    f("faults", Type::Num),
    f("signals", Type::Num),
    f("tasks", Type::Num),
    f("adr", Type::Num),
    f("pruned", Type::Strings),
    f(
        "findings",
        Type::Array(&[
            f("level", Type::Str),
            f("subject", Type::Str),
            f("message", Type::Str),
            f("note", Type::Strings),
            f(
                "charge",
                Type::Array(&[f("id", Type::Str), f("characters", Type::Num)]),
            ),
        ]),
    ),
];

/// What `help --json` returns, which is this description of itself.
const HELP_OUT: &[Field] = &[f(
    "verbs",
    Type::Array(&[
        f("name", Type::Str),
        f("usage", Type::Str),
        f("summary", Type::Str),
        f("group", Type::Str),
        f(
            "flags",
            Type::Array(&[
                f("name", Type::Str),
                opt("short", Type::Str),
                f("takes_value", Type::Bool),
                f("repeatable", Type::Bool),
            ]),
        ),
        f("notes", Type::Strings),
        f(
            "refuses",
            Type::Array(&[f("code", Type::Num), f("when", Type::Str)]),
        ),
        // Flat, with the path in the name: `tasks` is followed by `tasks.id`,
        // `tasks.title` and the rest. Nesting is how a shape is *written* here,
        // because that is what reads well beside the code; a dotted path is how
        // it is *published*, because this very document would otherwise have to
        // describe its own recursion, and a declaration that recurses into
        // itself does not terminate. No key in any document contains a dot, so
        // the path is unambiguous, and a client that wants the tree splits on
        // one character.
        f(
            "returns",
            Type::Array(&[
                opt("when", Type::Str),
                f(
                    "fields",
                    Type::Array(&[
                        f("name", Type::Str),
                        f("type", Type::Str),
                        f("nullable", Type::Bool),
                    ]),
                ),
            ]),
        ),
    ]),
)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlagSpec {
    pub name: &'static str,
    pub takes_value: bool,
    pub repeatable: bool,
    /// Whether `help` offers it. False for a name the parser knows only so the
    /// verb can refuse it precisely (§9): `help` lists what a caller can use,
    /// and a name that is always rejected is worse than absent there, because
    /// the caller reads an offer.
    pub listed: bool,
}

const fn flag(name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        takes_value: true,
        repeatable: false,
        listed: true,
    }
}

const fn switch(name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        takes_value: false,
        repeatable: false,
        listed: true,
    }
}

const fn multi(name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        takes_value: true,
        repeatable: true,
        listed: true,
    }
}

// A `refused` builder stood here — a `FlagSpec` with `listed: false`, for a name
// the parser accepts only so the verb can refuse it precisely. It had no caller:
// the case it was written for is now carried by `CommandSpec::refuses_globals`,
// which reaches the per-verb page, `--json` and the parser's own error alike. It
// was dead in `cli.rs` before this move and is not carried across, and nothing is
// lost with it — `FlagSpec`'s fields are public, so a surface that needs an
// unlisted flag declares one.

/// One state a verb refuses on, and the code it exits with (§4, §9).
///
/// Carried on the spec rather than only in the verb that raises it, because the
/// question "what will this refuse" is asked *before* the call, and the error is
/// only available after.
#[derive(Debug, Clone, Copy)]
pub struct Refusal {
    /// The code the caller gets back, as a variant and not as a number. The
    /// table is where a client reads what a verb will refuse *before* calling
    /// it, so this field is the contract's most load-bearing integer — which is
    /// exactly why it stopped being one.
    pub code: ExitCode,
    pub when: &'static str,
}

const fn refuses(code: ExitCode, when: &'static str) -> Refusal {
    Refusal { code, when }
}

/// The refusal every path-taking verb performs, declared once and named six
/// times.
///
/// SPEC-fe8bdb84faca states it for all of them at once — a path naming nothing
/// inside the repository, because it is absolute or because it climbs above the
/// root, is refused with the command to run next and never answered — and the
/// six verbs reach it through one helper, `context::normalised`. One sentence
/// here for one code path there: six literals would be six chances for the same
/// refusal to be described six ways, which is the fourth surface §9 refuses.
const OUTSIDE_THE_REPOSITORY: Refusal = refuses(
    ExitCode::Generic,
    "the path names nothing inside this repository",
);

/// Global flags, deliberately limited to three (§4). `--json` is available on
/// every command without exception: full scriptability is an invariant, not an
/// option — hence adding them mechanically to each command's surface rather
/// than declaring them per command, which would leave room to forget one.
pub const GLOBAL_FLAGS: &[FlagSpec] = &[
    switch("--json"),
    switch("--quiet"),
    flag("--repo"),
    // The second half of an address (ADR-9e56318631f3). `--repo` says which
    // corpus; this says which tree that corpus is anchored to. Long form only:
    // §4 gives `-w` to nothing, and a short form is an addition to that table
    // rather than a consequence of declaring a flag here.
    flag("--worktree"),
];

/// The short forms of §4 (ADR-1f70ce2c3eac), and the whole of them.
///
/// One table rather than a letter beside each declaration: `--scope` and
/// `--criteria` are declared in three [`CommandSpec`]s each, and three
/// declarations of one letter are three chances for two of them to disagree.
/// Here a letter can only mean one thing, which is exactly the property §4
/// claims for it.
///
/// The letter is the first letter of the long flag, without exception. Where
/// several long flags share one, exactly one takes it and the others keep only
/// their long form — a `-s` that meant `--status` under `find` and `--scope`
/// under `new` would not be a saving but a silent wrong answer.
pub const SHORT_FORMS: &[(&str, char)] = &[
    ("--json", 'j'),
    ("--quiet", 'q'),
    ("--repo", 'r'),
    ("--blocked-by", 'b'),
    ("--criteria", 'c'),
    ("--limit", 'l'),
    ("--proof", 'p'),
    ("--status", 's'),
    ("--type", 't'),
    ("--unset", 'u'),
    ("--verify", 'v'),
];

/// The short form of a long flag, if §4 gave it one.
pub fn short_of(long: &str) -> Option<char> {
    SHORT_FORMS
        .iter()
        .find(|(name, _)| *name == long)
        .map(|(_, c)| *c)
}

/// The long flag a letter stands for, anywhere. Whether that flag is legal on
/// the verb being parsed is a separate question, and asking it separately is
/// what lets the error say "not for this verb" instead of "no such flag".
pub fn long_of(c: char) -> Option<&'static str> {
    SHORT_FORMS
        .iter()
        .find(|(_, letter)| *letter == c)
        .map(|(name, _)| *name)
}

#[derive(Debug, Clone, Copy)]
pub struct CommandSpec {
    pub name: &'static str,
    /// **When** the verb is reached for, which is the heading `ank help` prints
    /// it under (ADR-91b77f036884). One of [`GROUPS`], and never a claim about
    /// who may run it: `check` sits under keeping the corpus honest whether a
    /// human or an agent types it, and the refusal machinery consults no caller.
    ///
    /// Declared here for the reason `coordinates` and `renews` are: a field is
    /// how the compiler asks the question of every verb that is ever added. A
    /// list beside the renderer would let a twenty-second verb arrive with no
    /// home and drop off the end of the listing, which is the failure the
    /// grouping exists to make impossible rather than to create.
    pub group: &'static str,
    /// What the verb does, in one line, printed by both surfaces of §9: the
    /// listing shows it beside the verb, and `ank help <verb>` above the flags.
    pub summary: &'static str,
    /// Mandatory subcommands, as in `new task` / `new adr`.
    pub subcommands: &'static [&'static str],
    pub max_positionals: usize,
    pub positional_help: &'static str,
    pub flags: &'static [FlagSpec],
    /// The states this verb refuses on, with their codes (§9).
    pub refuses: &'static [Refusal],
    /// Global flags this verb refuses by name (§4). Empty for every verb but
    /// two. `init` refuses `--repo`: the flag names a repository that already
    /// carries a `.ank/`, and `init` is what produces one. `watch` refuses
    /// `--repo` and `--worktree`: both address *a* corpus, and the watcher is
    /// told which corpora to keep warm by the reader's declaration and by
    /// nothing else (ADR-a22cd3196529) -- a flag that looked like it named one
    /// would be the discovery that decision refuses, offered from the caller's
    /// side.
    ///
    /// Declared here rather than only in the verb, because §9 forbids offering
    /// a name the verb rejects by design — so the same list has to reach the
    /// per-verb page, `--json`, and the parser's own error, and a global hidden
    /// from one rendering and not the others would be that defect in a quieter
    /// place.
    pub refuses_globals: &'static [&'static str],
    /// What the usage line cannot carry and the caller needs before calling: a
    /// value's grammar, or what interprets it. One line each.
    pub notes: &'static [&'static str],
    /// Whether the verb **coordinates**, and so requires git 2.34 or newer
    /// inside a repository (ADR-9307e5d214a7).
    ///
    /// The distinction is not between git and something else, it is between
    /// coordinating — which needs an arbiter — and reading a corpus, which
    /// needs a parser. A verb that only reads or writes entities answers on the
    /// files alone, and `check` runs the half of its invariants that needs no
    /// arbiter rather than refusing.
    ///
    /// Declared here rather than as a list beside the dispatch, for the reason
    /// that matters more than tidiness: the field makes the compiler ask the
    /// question of every verb that is ever added. A separate enumeration would
    /// let a new coordinating verb default to silence, which is the shape of
    /// the defect this ADR corrects — a property of the verb decided somewhere
    /// the verb is not.
    pub coordinates: bool,
    /// Whether running this verb is **work on the task the caller holds**, and
    /// so renews its lease (§3, ADR-0bb7ea8991bc).
    ///
    /// Declared here for the reason `coordinates` is, and the reason is the
    /// stronger of the two: §3 states a rule — the holder's verbs against the
    /// held task — precisely because a list of verb names is what goes stale
    /// when a verb is added. A field is how a rule is asked of every verb; a
    /// list beside the dispatch would let a new one default to renewing nothing,
    /// which is the failure this ADR corrects wearing a different hat.
    pub renews: Renews,
    /// The document the verb returns under `--json`, declared once and rendered
    /// by `help --json` (ADR-6fd69efb629c).
    ///
    /// Declared here for the reason `coordinates` and `renews` are, and the
    /// reason is sharper for this one: a verb added without a shape is a verb
    /// whose output nothing describes, and the compiler is the only thing that
    /// asks every verb the question. A list of shapes kept beside the renderer
    /// would let the twenty-third verb answer with a document no client can
    /// bind, silently.
    ///
    /// The `contract` field is on every document and is on no list here; the
    /// rendering adds it, because it is universal by construction.
    pub output: &'static [Shape],
    /// The task that carries the implementation, **while it does not exist**.
    /// It is therefore also the marker of an unrouted verb: a command that
    /// [`dispatch`] reaches clears the field, so the two never drift apart the
    /// way the module headers did.
    pub owner_task: Option<&'static str>,
}

/// The moments a verb is reached for, in the order `ank help` prints them
/// (ADR-91b77f036884).
///
/// A group says **when** a verb is used and never **who** may use it. The
/// layering ADR-91b77f036884 removed was the residue of an agent surface and a
/// human surface — headings that told a caller which verbs were theirs, behind
/// a wall built from `$ANK_AGENT`, which the caller sets itself. Nothing here
/// reopens that: the distinction is the one between a map and a gate.
///
/// Lowercase, because a heading here is a signpost and not a title. The order
/// is the reader's path through the tool, so `run the loop` comes first for the
/// same reason §4 does.
pub const GROUPS: &[&str] = &[
    "run the loop",
    "shape the work",
    "look around",
    "keep the corpus honest",
    "set up a repository",
];

/// The twelve verbs of §4, plus `init` and `help` (§9).
///
/// **The order is the specification's, and it is load-bearing**: §4 puts the
/// loop first — `context claim show log done`, then `release new find` — and
/// the rest after it. `help` groups this table by [`GROUPS`] and keeps this
/// order inside each group (ADR-91b77f036884): the grouping is a second axis
/// laid over §4's order, not a re-sort, so a verb never moves relative to its
/// neighbours and sorting this list would still erase what §4 says.
pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "context",
        group: "run the loop",
        renews: Renews::Held,
        coordinates: false,
        summary: "what binds this perimeter and what is claimable; with a claim held, the criterion and the constraints in full",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[flag("--limit")],
        refuses: &[
            OUTSIDE_THE_REPOSITORY,
            refuses(ExitCode::Generic, "--limit is not a number"),
        ],
        notes: &["a constraint is never truncated in execution mode; a cut is always announced"],
        refuses_globals: &[],
        output: &[one(CONTEXT_OUT)],
        owner_task: None,
    },
    CommandSpec {
        name: "claim",
        group: "run the loop",
        renews: Renews::Never,
        coordinates: true,
        summary: "takes the task and freezes its done_criteria by hash; refuses one held, blocked, or finished on another branch",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--criteria"), flag("--ttl")],
        refuses: &[
            refuses(ExitCode::Unavailable, "the task is held by another agent, or finished on another branch"),
            refuses(ExitCode::Prerequisite, "the task is blocked, or has no done_criteria to freeze"),
        ],
        notes: &["--criteria sets a criterion the task does not have, and records it as the claimer's; it never replaces one"],
        refuses_globals: &[],
        output: &[one(CLAIM_OUT), when("this identity holds a live claim in another corpus the reader declared", CLAIM_OUT_ELSEWHERE)],
        owner_task: None,
    },
    CommandSpec {
        name: "show",
        group: "run the loop",
        renews: Renews::Named,
        coordinates: false,
        summary: "the entity whole, frontmatter and body, byte for byte",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        refuses: &[refuses(ExitCode::NotFound, "no such entity, or the prefix matches more than one")],
        notes: &[],
        refuses_globals: &[],
        output: &[when("over a task", SHOW_TASK), when("over an ADR, a spec or a log entry", SHOW_OTHER)],
        owner_task: None,
    },
    CommandSpec {
        name: "log",
        group: "run the loop",
        renews: Renews::Never,
        coordinates: true,
        // **The claim is a condition on the subject, not on the write** (§4,
        // ADR-25f977377fa0). A claim arbitrates work, and only an `open` or
        // `in_progress` task has work left to arbitrate: an ADR, a spec, and a
        // task already `done` or `closed` each take an entry with no claim held
        // and none taken.
        //
        // This line and the refusal below both said the write needs the claim,
        // flatly. That was the whole rule when it was written and became
        // narrower than the rule the day a correction gained somewhere to land
        // on a settled task (TASK-c34392707a7b). A help line narrower than the
        // binary is read as the binary refusing, which is the failure §9 exists
        // to prevent.
        summary: "an id alone reads the log; an id and a message appends one and renews the claim, needed where a claim arbitrates work and not on a done or closed task",
        subcommands: &[],
        max_positionals: 2,
        // Both optional, and what is given decides which of the two things the
        // verb does: an id alone reads, a message writes (§4).
        positional_help: "[<id>] [<message>]",
        flags: &[],
        refuses: &[refuses(ExitCode::Transition, "writing to an open or in_progress task with no claim held by this agent")],
        notes: &["a done or closed task has to be named: HEAD never points at one"],
        refuses_globals: &[],
        output: &[when("reading, `ank log <id>`", &[f("about", Type::Str), f("total", Type::Num), f("shown", Type::Num), f("entries", Type::Array(LOG_ENTRY)), f("machinery", Type::Array(LOG_ENTRY))]), when("appending, `ank log <id> <message>`", &[f("about", Type::Str), f("entry", Type::Str), f("logged", Type::Bool), f("warnings", Type::Strings)])],
        owner_task: None,
    },
    CommandSpec {
        name: "done",
        group: "run the loop",
        renews: Renews::Never,
        coordinates: true,
        // "the declared verifiers" left out who declares them, and a reader
        // filled the blank with config.yml -- which defines verifiers but never
        // selects any. One agent wrote that reading into the project guide and
        // found out only by running the verb. The task's `verify:` list is what
        // decides, so the page names it (TASK-ca784c5feda4).
        summary: "runs the verifiers the task's verify: list names, records what ran, and moves the task to done; needs the claim, and a proof when that list is empty",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<id>]",
        flags: &[flag("--proof")],
        refuses: &[
            refuses(ExitCode::Proof, "no proof, and the task's verify: list names no verifier to produce one"),
            refuses(ExitCode::Transition, "no claim held by this agent, or the frozen done_criteria has diverged"),
        ],
        notes: &[
            "--proof is <type>:<ref>; type is commit, human-review, assertion or test",
            "config.yml defines the verifiers; the task's verify: list decides which of them run",
        ],
        refuses_globals: &[],
        output: &[one(&[f("task", Type::Str), f("status", Type::Str), f("commit", Type::Str), opt("branch", Type::Str), f("proofs", Type::Num)])],
        owner_task: None,
    },
    CommandSpec {
        name: "release",
        group: "run the loop",
        renews: Renews::Never,
        coordinates: true,
        summary: "hands the task back, with the reason recorded in its log",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<id>]",
        flags: &[flag("--reason")],
        refuses: &[refuses(ExitCode::Transition, "no claim held by this agent")],
        notes: &[],
        refuses_globals: &[],
        output: &[one(&[f("task", Type::Str), f("status", Type::Str), f("reason", Type::Str), f("warnings", Type::Strings)])],
        owner_task: None,
    },
    CommandSpec {
        name: "new",
        group: "shape the work",
        renews: Renews::Never,
        coordinates: false,
        summary: "writes a task, an ADR or a spec that needs no hand finishing",
        subcommands: &["task", "adr", "spec"],
        max_positionals: 0,
        positional_help: "",
        flags: &[
            flag("--title"),
            multi("--scope"),
            flag("--criteria"),
            multi("--blocked-by"),
            flag("--constraint"),
            flag("--supersedes"),
            multi("--reference"),
            multi("--verify"),
            flag("--body"),
        ],
        refuses: &[refuses(ExitCode::Environment, "no --title or --scope and $EDITOR is unset, so there is nothing to open")],
        notes: &[
            "a scope is mandatory: an entity attached to nothing is invisible",
            "--body - reads the body from stdin, so a long one needs no shell quoting",
            "--reference declares what a spec rests on; it takes a spec or an adr, and check resolves it",
        ],
        refuses_globals: &[],
        output: &[one(&[f("id", Type::Str), f("kind", Type::Str), f("created", Type::Str)])],
        owner_task: None,
    },
    CommandSpec {
        name: "find",
        group: "look around",
        renews: Renews::Never,
        coordinates: false,
        summary: "searches titles, scopes and criteria; --type spec reaches the specification, --status open lists what remains",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<query>",
        flags: &[
            flag("--type"),
            flag("--status"),
            flag("--scope"),
            switch("--free"),
        ],
        refuses: &[
            refuses(ExitCode::Generic, "--type names a kind the registry does not declare"),
            refuses(ExitCode::Generic, "--scope names nothing inside this repository"),
        ],
        notes: &[
            "--status filters on the stored status; a claimed row still displays as [claimed:who]",
            "a listing counts the open rows a claim would refuse, and names --free",
            "--free keeps the open tasks no live claim's scope overlaps, and says how many it hid",
        ],
        refuses_globals: &[],
        output: &[one(&[f("total", Type::Num), f("shown", Type::Num), f("hidden", Type::Num), f("results", Type::Array(&[f("id", Type::Str), f("kind", Type::Str), f("status", Type::Str), f("state", Type::Str), f("title", Type::Str)]))])],
        owner_task: None,
    },
    // After `find` and before `review`, which is where §4 puts it. Placing it
    // beside `graph` instead read as tidy and was wrong; `tests/skill.rs`
    // refused the commit until it moved (TASK-15336a0012d5).
    CommandSpec {
        name: "status",
        group: "look around",
        renews: Renews::Never,
        coordinates: false,
        summary: "where am I: branch, claim, perimeter, queue, findings",
        subcommands: &[],
        max_positionals: 0,
        positional_help: "",
        flags: &[switch("--remote")],
        // **Empty, and measured rather than assumed** (TASK-106dccc7f71c). This
        // verb takes no path, parses no value of its own, and raises no refusal
        // anywhere in `status.rs`: `--remote` reads an unreachable origin as a
        // warning and answers on the local plane, which is the paragraph above.
        // So the vacuous case §9 allows is the true one here, and the empty
        // array is a fact about the verb rather than a gap nobody looked at.
        refuses: &[],
        // `coordinates` stays false, and the flag does not change that: without
        // it `status` pays for no network at all, and with it an unreachable
        // origin is a warning and the local answer rather than a refusal. A
        // reader never fails for want of something to say (§2).
        notes: &[
            "--remote reads the claim refs from origin with ls-remote and never fetches; without it status describes the local plane only",
        ],
        refuses_globals: &[],
        output: &[one(STATUS_OUT)],
        owner_task: None,
    },
    CommandSpec {
        name: "review",
        group: "shape the work",
        renews: Renews::Never,
        coordinates: false,
        summary: "the ratification queue and the health of the corpus: what is proposed, who may ratify it, and which scopes have gone dead",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        refuses: &[OUTSIDE_THE_REPOSITORY],
        // `review` shares `check`'s report and therefore its exit code, and for
        // a long time it said so nowhere: a caller reading 8 as "check found
        // something" met it from a verb whose page promised nothing of the
        // kind. Found while pinning the goldens for TASK-2c12b027f805.
        notes: &[
            "exit 8 means findings, as it does for check; a signal alone leaves it 0",
            "the signers are what .ank/allowed_signers declares; this is where they are read, since ADR-01b6dd05f0db closes that file to a direct read",
        ],
        refuses_globals: &[],
        output: &[one(&[f("proposed", Type::Array(&[f("id", Type::Str), f("title", Type::Str)])), f("signers", Type::Array(&[f("principal", Type::Str), f("keytype", Type::Str)])), f("live", Type::Array(&[f("id", Type::Str), f("title", Type::Str), f("files", Type::Num)])), f("dead", Type::Num), f("faults", Type::Num), f("signals", Type::Num)])],
        owner_task: None,
    },
    CommandSpec {
        name: "accept",
        group: "shape the work",
        renews: Renews::Never,
        coordinates: true,
        summary: "promotes a proposed ADR or spec to accepted, through a signed ratification commit; on the default branch only",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        refuses: &[
            refuses(ExitCode::NotFound, "no such entity, or the prefix matches more than one"),
            refuses(ExitCode::Prerequisite, "not on the default branch, and there is no way around it"),
            refuses(
                ExitCode::Environment,
                "the default branch cannot be determined, from config.yml or from origin",
            ),
        ],
        notes: &["the one act ank commits for; it is a human act, signed"],
        refuses_globals: &[],
        output: &[one(&[f("id", Type::Str), f("kind", Type::Str), f("status", Type::Str), opt("superseded", Type::Str), f("commit", Type::Str), f("anchor", Type::Str)])],
        owner_task: None,
    },
    // Between `accept` and `close`: §4's order, and beside the verb this one was
    // split from. `accept` records a reading as a consequence of ratifying, and
    // everything it cannot reach needs the act on its own.
    CommandSpec {
        name: "read",
        group: "shape the work",
        // The same answer `show` gives, and for the same reason: reading the
        // entity you hold is work on it, and working is what keeps the lock.
        renews: Renews::Named,
        // Nothing here touches a ref, a branch or a remote. The reading is a
        // field on the entity and the write is the store's, so this verb answers
        // on the files alone (ADR-9307e5d214a7).
        coordinates: false,
        summary: "records that a person read this entity and stands behind it",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        refuses: &[
            refuses(ExitCode::NotFound, "no such entity, or the prefix matches more than one"),
            refuses(
                ExitCode::Generic,
                "a log entry: it is written once, and a reading would be a second write",
            ),
        ],
        notes: &[
            "the reading names the identity in effect and the instant; nothing is signed, and nothing is refused on who is calling",
            "a second reading by the same actor is appended, because reading a document again is a different act",
        ],
        refuses_globals: &[],
        output: &[one(&[f("entity", Type::Str), f("kind", Type::Str), f("by", Type::Str), f("at", Type::Str), f("readings", Type::Num)])],
        owner_task: None,
    },
    CommandSpec {
        name: "close",
        group: "shape the work",
        renews: Renews::Never,
        coordinates: true,
        summary: "closes a task that will never be done; --reason is mandatory",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--reason")],
        refuses: &[
            refuses(
                ExitCode::Prerequisite,
                "no --reason: a closure nobody explained is one nobody can reopen",
            ),
            refuses(ExitCode::NotFound, "no such entity, or the prefix matches more than one"),
        ],
        notes: &[],
        refuses_globals: &[],
        output: &[one(&[f("task", Type::Str), f("status", Type::Str), f("claim_revoked", Type::Bool)])],
        owner_task: None,
    },
    CommandSpec {
        name: "amend",
        group: "shape the work",
        renews: Renews::Named,
        coordinates: false,
        summary: "changes blocked_by, references, scope, and a done_criteria no live claim freezes",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[
            multi("--blocked-by"),
            multi("--drop-blocked-by"),
            multi("--reference"),
            multi("--drop-reference"),
            multi("--scope"),
            multi("--drop-scope"),
            // Offered now, and it was `refused` here for as long as the verb
            // rejected it outright (TASK-84cfad83c308: help must not make an
            // offer the verb turns down). It stops being an offer the verb
            // rejects the moment the verb accepts it on state (§4).
            flag("--criteria"),
        ],
        refuses: &[refuses(
            ExitCode::Transition,
            "--criteria while a live claim freezes the criterion; that case is a release",
        )],
        notes: &[
            "adds and removes explicitly, never a replacement list, so nothing is dropped by being forgotten",
            "--criteria replaces the criterion outright, and leaves criteria_by where it stands",
            "--reference and --drop-reference reach a spec's citations, on an accepted one too: the anchor covers its body and scope, not what it cites",
        ],
        refuses_globals: &[],
        output: &[one(&[f("entity", Type::Str), f("amended", Type::Strings)])],
        owner_task: None,
    },
    CommandSpec {
        name: "attest",
        group: "shape the work",
        renews: Renews::Named,
        coordinates: true,
        summary: "appends a proof to a finished task: the one write allowed after done",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--proof"), switch("--detached")],
        refuses: &[
            refuses(ExitCode::NotFound, "no such entity, or the prefix matches more than one"),
            // Which side of ADR-af533e7a3e03 this verb is on, said here so that
            // no caller has to infer it from what the verb happens to touch.
            // `claim` writes a ref too and degrades; this one has nothing left
            // over when the push fails, so it fails.
            refuses(
                ExitCode::Environment,
                "--detached and the remote unreachable: the ref is the whole product, and a proof no other clone can read is no proof",
            ),
        ],
        notes: &[
            "--proof is <type>:<ref>; type is commit, human-review, assertion or test",
            "--detached records the proof in refs/ank/proof/<id> and writes no file, so a pipeline anchors a run without a commit",
        ],
        refuses_globals: &[],
        output: &[one(&[f("task", Type::Str), f("appended", Type::Object(&[f("type", Type::Str), f("ref", Type::Str)])), f("proofs", Type::Num)])],
        owner_task: None,
    },
    // After `attest` and before `graph`: §4's order, and the last gap in it.
    // `tests/skill.rs` is what holds this to §4 rather than to memory.
    CommandSpec {
        name: "edit",
        group: "keep the corpus honest",
        renews: Renews::Named,
        coordinates: false,
        summary: "changes the content field named, or opens the entity in $EDITOR",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--title"), flag("--body"), flag("--constraint")],
        refuses: &[
            refuses(ExitCode::Environment, "$EDITOR is unset with no field named, and there is no editor to open"),
            refuses(ExitCode::Generic, "a field named is not one the addressed kind carries"),
        ],
        notes: &[
            "a named field writes only what is named and never opens an editor; with none, $EDITOR opens on the whole entity",
            "--body - reads the body from stdin, so a long one needs no shell quoting",
            "the refusals are the editor path's, unchanged: a field a live claim has frozen is refused the same way on both",
            "$EDITOR is a command line run through sh, not a program name",
            "a GUI editor needs its wait flag, or it returns before you have typed and the file is written back unedited",
        ],
        refuses_globals: &[],
        output: &[one(&[f("entity", Type::Str), f("changed", Type::Strings), f("version", Type::Num)])],
        owner_task: None,
    },
    CommandSpec {
        name: "graph",
        group: "look around",
        renews: Renews::Never,
        coordinates: false,
        summary: "the blocked_by DAG in readable text, indented under what blocks it",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        refuses: &[OUTSIDE_THE_REPOSITORY],
        notes: &[],
        refuses_globals: &[],
        output: &[one(&[f("path", Type::Str), f("tasks", Type::Array(&[f("id", Type::Str), f("short", Type::Str), f("status", Type::Str), f("title", Type::Str)])), f("edges", Type::Array(&[f("task", Type::Str), f("blocked_by", Type::Str)]))])],
        owner_task: None,
    },
    CommandSpec {
        name: "scope",
        group: "look around",
        renews: Renews::Never,
        coordinates: false,
        summary: "what covers a path: the constraints that bind it, the specifications that govern it, and the tasks that touch it",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<path>",
        flags: &[],
        refuses: &[
            refuses(ExitCode::Generic, "no path given, and this verb answers about one"),
            OUTSIDE_THE_REPOSITORY,
        ],
        notes: &[],
        refuses_globals: &[],
        output: &[one(&[f("path", Type::Str), f("total", Type::Num), f("adr", Type::Array(ROW)), f("specs", Type::Array(ROW)), f("tasks", Type::Array(ROW))])],
        owner_task: None,
    },
    // Beside `graph` and `scope`, which is the neighbourhood §4 puts it in: the
    // verbs that answer *what is here* rather than change it. A reader that
    // also acts does not become a different kind of verb -- what it acts
    // through is `claim`, `log` and `done`, run as a shell would run them
    // (ADR-8bd76e8d7c4e).
    CommandSpec {
        name: "tui",
        group: "look around",
        renews: Renews::Never,
        // It runs the CLI, and the CLI checks git per verb (ADR-9307e5d214a7).
        // Requiring it here would refuse a reader over a corpus that four
        // read-only verbs answer about perfectly well.
        coordinates: false,
        summary: "a full-screen reader over this corpus: every entity with its status, a body whole, what binds it, and who holds what",
        subcommands: &[],
        max_positionals: 0,
        positional_help: "",
        flags: &[],
        refuses: &[refuses(
            ExitCode::Environment,
            "no terminal on stdin and stdout: a screen cannot be drawn into a pipe",
        )],
        notes: &[
            // "and writes nothing" stood here for as long as the reader only
            // read, and stopped being true the day claim, log, release, done and
            // amend became reachable from a selected entity (TASK-b50b340c0bb1).
            // A help line narrower than the binary is read as the binary
            // refusing, which is the failure §9 exists to prevent.
            "it reaches the corpus only by running this binary with --json, and writes nothing on its own",
            "a command is a short word and Enter; ? lists them, q leaves",
            "claim, log, release, done and amend act on the selected entity, each by running that verb",
            "--json answers the opening frame as data and opens no session",
        ],
        refuses_globals: &[],
        output: &[one(TUI_OUT)],
        owner_task: None,
    },
    // After `tui`, which is the order §4 puts them in and the neighbourhood
    // both belong to: a surface that answers *what is here* to a caller who
    // cannot type the verbs themselves. `tui` serves the human at a terminal,
    // this one serves the client with no shell at all. Neither becomes a
    // different kind of verb by having a client, because what they act through
    // is `claim`, `log` and `done`, run as a shell would run them
    // (ADR-fd98f4bc6dea, ADR-8bd76e8d7c4e).
    CommandSpec {
        name: "mcp",
        group: "look around",
        // The server renews nothing. A call that renews a lease renews it
        // because the CLI ran the holder's verb against the held task in the
        // child, which is the one place the rule is applied (ADR-0bb7ea8991bc);
        // a client that left a session open all night has done no work.
        renews: Renews::Never,
        // It runs this binary per call, and the CLI checks git per verb
        // (ADR-9307e5d214a7). Requiring it here would refuse the server before
        // a client had asked anything, over a corpus the read-only half of the
        // surface answers about perfectly well.
        coordinates: false,
        summary: "every verb of this table as a tool, over MCP on stdio, for a client that has no shell",
        subcommands: &[],
        max_positionals: 0,
        positional_help: "",
        flags: &[],
        refuses: &[refuses(
            ExitCode::Environment,
            "this binary cannot find itself on disk, and every call is a run of it",
        )],
        notes: &[
            "JSON-RPC on stdio, one message per line: initialize, tools/list, tools/call, ping",
            "every verb of this table is a tool, generated from it: no curated subset, and no second list kept in step by review",
            "a call runs `ank <verb> --repo <corpus> --json` as a child, so every refusal is the one this binary gives, carrying its exit code",
            "one process speaks for one corpus, the one --repo addressed at startup",
            "--json names what this surface already is, and changes nothing",
        ],
        refuses_globals: &[],
        // **No `--json` document leaves this verb, and the empty list is the
        // answer rather than a verb that forgot to give one.** What leaves the
        // process is JSON-RPC, one message per line, and the contract for that
        // shape is MCP's own -- the documents ADR-6fd69efb629c governs are the
        // ones the *children* return, each already declared by the verb that
        // returns it and carried through untouched.
        output: &[],
        owner_task: None,
    },
    // After `mcp`, which is §4's order, and in the same group for the reason §4
    // gives when it says both are verbs for the reason `tui` is one: every
    // route carries one executable (ADR-1ea31c2f3c5a), so a surface that is not
    // a verb is a surface that has to arrive beside the binary.
    //
    // **It is grouped with the verbs that look and never with the verbs that
    // act, and that is the honest place for it.** It changes no entity, takes
    // no claim and answers no question; what it leaves behind is a cache the
    // next `ank` finds warm. A reader scanning `ank help` for the thing that
    // makes a listing fast reads this group.
    CommandSpec {
        name: "watch",
        group: "look around",
        // It takes no claim, holds none on anybody's behalf and renews none
        // (ADR-a22cd3196529, §4). A lease is renewed by working, and this
        // process does not work on a task -- it warms an index
        // (ADR-0bb7ea8991bc).
        renews: Renews::Never,
        // **False, and this is the one verb where that is not about cost.**
        // `coordinates` makes the foundation check git in *the checkout's*
        // corpus, and this verb has no such corpus: what it watches is what the
        // reader declared outside every repository, so there is nothing here
        // for the check to be about. git is asked per declared checkout, and a
        // checkout whose git will not answer costs a line and the next poll,
        // because nothing depends on this process.
        coordinates: false,
        summary: "keeps the corpora you declared warm, so the ank you run answers sooner; it answers no verb and nothing depends on it",
        subcommands: &[],
        max_positionals: 0,
        positional_help: "",
        // Exactly the four §4 lists, and the shape of that list is the
        // decision rather than an omission: not one of them asks the watcher a
        // question about a corpus. A caller that wants an answer runs the CLI,
        // which is what keeps this from being a second dispatch path -- and it
        // is why being started by a verb leaves "it answers no verb" intact.
        flags: &[
            switch("--list"),
            switch("--once"),
            flag("--interval"),
            switch("--where"),
        ],
        refuses: &[
            refuses(
                ExitCode::Environment,
                "no home directory in the environment, so there is nowhere to declare a corpus",
            ),
            refuses(
                ExitCode::Environment,
                "the declaration does not exist, names a schema this build does not read, or is not YAML",
            ),
            refuses(
                ExitCode::Environment,
                "a declared directory holds no .ank/, or holds a corpus filed under another identity: nothing looks for one elsewhere",
            ),
            refuses(
                ExitCode::Environment,
                "--interval is not a number of milliseconds, or is 0, which would spin rather than watch",
            ),
            refuses(
                ExitCode::Generic,
                "--repo or --worktree: each addresses one corpus, and this verb warms the corpora you declared",
            ),
        ],
        notes: &[
            "it declares nothing: watch.yml in this reader's configuration directory says which corpora to keep warm, and --where prints where that is",
            "the only things it writes into a repository are that repository's own index and a mirror of refs/ank/* under refs/ank/watch/; it moves no branch, no tag and no claim of yours",
            "a change it sees becomes a line on events.jsonl beside watch.yml, saying which corpus moved and what kind of change it was, and never what to do about it",
            "every verb behaves the same with it stopped, its absence is never an error, and no route makes running it a condition of using ank",
            "--json changes nothing: what a program follows is events.jsonl, whose shape is the event stream's own",
        ],
        // **The two globals that address a corpus are refused rather than
        // ignored**, and §9 is why it is a refusal and not a silence: a name
        // the verb rejects by design may not be offered, and a name that is
        // offered has to do something. `--repo` and `--worktree` do nothing
        // here, because what this verb watches is what the reader declared in
        // watch.yml -- so a caller who typed `ank watch --repo <tree>` would
        // have addressed a corpus that is not being watched and been told
        // nothing. That is the discovery ADR-a22cd3196529 refuses, reached from
        // the caller's side instead of the filesystem's.
        refuses_globals: &["--repo", "--worktree"],
        // **No `--json` document leaves this verb either, and the empty list is
        // the answer rather than a verb that forgot to give one.** `--list`
        // prints a listing for a person reading a log, and the document a
        // program wanted is `events.jsonl` -- written by this process, followed
        // by anyone, and already declared where both ends of it read it.
        output: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "check",
        group: "keep the corpus honest",
        renews: Renews::Never,
        coordinates: false,
        // A verb called `check` reads as read-only, and this one writes: it is
        // the only command that prunes (§7). An agent ran it in a loop on that
        // assumption and read `pruned refs/ank/claims/...` back. The page is
        // where a caller finds out, before scripting around it.
        summary: "the mechanical invariants: parse, round-trip, references, frozen fields, orphaned claims; prunes the claim refs it finds stale, so it writes",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        refuses: &[OUTSIDE_THE_REPOSITORY],
        notes: &[
            "exit 8 means findings; a signal alone leaves it 0",
            "the only verb that prunes refs/ank/claims: orphans, and completion refs whose task is done or closed on the default branch",
        ],
        refuses_globals: &[],
        output: &[one(CHECK_OUT)],
        owner_task: None,
    },
    // After `check`, which is the verb that names it: a corpus still holding
    // the previous log directory is a `check` signal, and this is the command
    // that signal prints (§4).
    CommandSpec {
        name: "migrate",
        group: "keep the corpus honest",
        renews: Renews::Never,
        coordinates: false,
        summary: "rewrites the previous log directory as entries, one entity per entry, and removes what it read",
        subcommands: &[],
        max_positionals: 0,
        positional_help: "",
        flags: &[],
        refuses: &[refuses(
            ExitCode::Generic,
            "a log file the grammar refuses, or one whose entity is not in the corpus: named, and nothing is written",
        )],
        notes: &[
            "the entry count is asserted equal before and after, and every message is read back and compared",
            "it writes files and never commits: review with git status .ank",
        ],
        refuses_globals: &[],
        output: &[one(&[f("files", Type::Num), f("entries", Type::Num), f("created", Type::Num)])],
        owner_task: None,
    },
    // After `check` and before `init`: §4's order. It sits beside the verb
    // that writes `config.yml` in the first place, which is the reading §9
    // states -- what `init` writes, `config` maintains.
    CommandSpec {
        name: "config",
        group: "set up a repository",
        renews: Renews::Never,
        coordinates: false,
        summary: "reads and writes .ank/config.yml: the key alone reads, a value writes, --unset removes",
        subcommands: &[],
        max_positionals: 2,
        positional_help: "<key> [<value>]",
        flags: &[switch("--unset"), switch("--user")],
        refuses: &[
            refuses(
                ExitCode::Generic,
                "a key the parser does not know, or a value in a form the surgery cannot edit safely",
            ),
            refuses(ExitCode::Prerequisite, "verifiers.<name>.timeout on a verifier that is not declared"),
        ],
        notes: &[
            "keys: schema context_budget claim_ttl_max claim_ttl_default default_branch peers.<name> verifiers.<name>.run verifiers.<name>.timeout",
            "a resolved default prints marked as one; --json carries value and source as separate fields",
            "--unset verifiers.<name> removes a whole verifier, which is what makes declaring one reversible",
            "--user reads and writes the reader's corpora.yml instead, whose only key is corpora.<identity>",
        ],
        refuses_globals: &[],
        output: &[when("reading, `ank config <key>`", &[f("key", Type::Str), opt("value", Type::Str), f("source", Type::Str)]), when("writing, `ank config <key> <value>`", &[f("key", Type::Str), opt("previous", Type::Str), opt("value", Type::Str), f("changed", Type::Bool)])],
        owner_task: None,
    },
    CommandSpec {
        name: "init",
        group: "set up a repository",
        renews: Renews::Never,
        coordinates: true,
        summary: "creates .ank/ here or at <path>, writes config.yml, adds the refs/ank/* refspec; refuses --repo",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[flag("--at")],
        refuses: &[refuses(
            ExitCode::Generic,
            "--repo: it names a repository that exists, and this verb makes one; the target is positional",
        )],
        notes: &[
            "a target elsewhere is ank init <path>; with no argument it initialises the current directory",
            "--at <path> puts the corpus outside this tree and declares it, so the tree gains no file",
        ],
        refuses_globals: &["--repo"],
        output: &[one(&[f("created", Type::Strings), f("wrote", Type::Strings), f("added", Type::Strings), f("changed", Type::Bool)])],
        owner_task: None,
    },
    CommandSpec {
        name: "help",
        group: "set up a repository",
        renews: Renews::Never,
        coordinates: false,
        summary: "every verb grouped by the moment it is used, or one verb in full",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<verb>]",
        flags: &[],
        refuses: &[refuses(ExitCode::NotFound, "no such verb; never a fallback to the general listing")],
        notes: &[],
        refuses_globals: &[],
        output: &[one(HELP_OUT)],
        owner_task: None,
    },
];

pub fn spec_of(name: &str) -> Option<&'static CommandSpec> {
    COMMANDS.iter().find(|c| c.name == name)
}

pub fn known_flags(spec: &CommandSpec) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = spec.flags.iter().map(|f| f.name).collect();
    v.extend(GLOBAL_FLAGS.iter().map(|f| f.name));
    v.sort_unstable();
    v
}

pub fn find_flag(spec: &CommandSpec, name: &str) -> Option<FlagSpec> {
    spec.flags
        .iter()
        .chain(GLOBAL_FLAGS.iter())
        .find(|f| f.name == name)
        .copied()
}

/// The usage line of a verb, derived from the table and never written beside it.
///
/// Here rather than with the `help` rendering because it is not a rendering
/// choice: `ank claim <id>` is what the surface *is*, it appears in the listing,
/// in the per-verb page, in `--json` and in the hint of a parse error, and a
/// second way of spelling it would be a second surface.
pub fn usage(spec: &CommandSpec) -> String {
    let mut s = format!("ank {}", spec.name);
    if !spec.subcommands.is_empty() {
        s.push_str(&format!(" <{}>", spec.subcommands.join("|")));
    }
    if !spec.positional_help.is_empty() {
        s.push(' ');
        s.push_str(spec.positional_help);
    }
    s
}
