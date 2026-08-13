//! Argument parsing, `help` and dispatch (§4, §9, §12).
//!
//! Parsing is done by hand, with no library. The reason is not saving a
//! dependency but character-level control over two surfaces read by agents:
//! the self-correcting errors, which a generic parser would replace with its
//! own messages, and the help, whose cost is paid on every call that triggers
//! it (§12). Neither argument depends on the verb list being frozen, which it
//! is not: hand-written parsing costs once per verb, the verbs share one
//! parser, and what grows is linear and small against a `help` and an error
//! surface that stay exactly as written.
//!
//! `help` lives here rather than in a verb module because it has no data of
//! its own: it is a rendering of [`COMMANDS`], [`GLOBAL_FLAGS`] and [`usage`],
//! which are all in this file. A second, hand-maintained list of the verbs is
//! exactly the drift the `owner_task` field was added to prevent.
//!
//! **The listing is flat, and the order of [`COMMANDS`] is the whole of its
//! structure** (ADR-c656cbcc33a9). It used to group verbs under headings named
//! after callers, which was the two-surface model still speaking through the
//! output an agent reads; a heading printed by the binary is a claim about who
//! a verb is for, and there is no such claim left to make. §4 already orders
//! the table with the loop first, so the order says what the headings said,
//! without asserting a category. What the loop *is* stays in SKILL.md, whose
//! content is frozen and loaded permanently — that is where the token budget
//! is spent, and `help` is loaded on demand precisely so it does not compete.
//!
//! The edge cases of parsing are where hand-written code goes wrong, and they
//! look like business bugs once in production: every one of them is therefore
//! tested, one test per case.

use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;

// ---------------------------------------------------------------------------
// Error carrying its exit code
// ---------------------------------------------------------------------------

/// A CLI error. The code comes from the table in §4; `hint` carries the exact
/// command to run next. Never generic help: one well-designed error round trip
/// costs less than three blind attempts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub code: i32,
    pub message: String,
    pub hint: Option<String>,
}

impl CliError {
    pub fn new(code: i32, message: impl Into<String>) -> CliError {
        CliError {
            code,
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> CliError {
        self.hint = Some(hint.into());
        self
    }

    /// Terse rendering, `git status` style, on standard error.
    pub fn render(&self) -> String {
        self.render_styled(crate::style::PLAIN)
    }

    /// The same line, with the `error[N]:` tag painted when §4 allows it.
    ///
    /// Only the tag: the message and the hint are the part a reader has to
    /// read, and the hint is a command to copy. Painting either would be
    /// decoration, and the hint is the last thing that should be hard to
    /// select.
    pub fn render_styled(&self, style: crate::style::Style) -> String {
        let tag = style.red(&format!("error[{}]:", self.code));
        match &self.hint {
            Some(h) => format!("{tag} {}\n  -> {}", self.message, h),
            None => format!("{tag} {}", self.message),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

impl From<crate::store::StoreError> for CliError {
    fn from(e: crate::store::StoreError) -> CliError {
        let code = e.code();
        let hint = e.hint();
        let mut err = CliError::new(code, e.to_string());
        err.hint = hint;
        err
    }
}

pub type Result<T> = std::result::Result<T, CliError>;

// ---------------------------------------------------------------------------
// Description of the surface
// ---------------------------------------------------------------------------

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

/// A name the parser accepts so that the verb can refuse it by name, with the
/// command to run instead. The parser's "unknown flag" would list the valid
/// ones and leave the caller to work out why the obvious one is missing.
const fn refused(name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        takes_value: true,
        repeatable: false,
        listed: false,
    }
}

/// One state a verb refuses on, and the code it exits with (§4, §9).
///
/// Carried on the spec rather than only in the verb that raises it, because the
/// question "what will this refuse" is asked *before* the call, and the error is
/// only available after.
#[derive(Debug, Clone, Copy)]
pub struct Refusal {
    pub code: i32,
    pub when: &'static str,
}

const fn refuses(code: i32, when: &'static str) -> Refusal {
    Refusal { code, when }
}

/// Global flags, deliberately limited to three (§4). `--json` is available on
/// every command without exception: full scriptability is an invariant, not an
/// option — hence adding them mechanically to each command's surface rather
/// than declaring them per command, which would leave room to forget one.
pub const GLOBAL_FLAGS: &[FlagSpec] = &[switch("--json"), switch("--quiet"), flag("--repo")];

/// The short forms of §4 (ADR-962c25797569), and the whole of them.
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
fn long_of(c: char) -> Option<&'static str> {
    SHORT_FORMS
        .iter()
        .find(|(_, letter)| *letter == c)
        .map(|(name, _)| *name)
}

#[derive(Debug, Clone, Copy)]
pub struct CommandSpec {
    pub name: &'static str,
    /// What the verb does, in one line, for `ank help <verb>` (§9). Not for the
    /// flat listing, which stays what it was.
    pub summary: &'static str,
    /// Mandatory subcommands, as in `new task` / `new adr`.
    pub subcommands: &'static [&'static str],
    pub max_positionals: usize,
    pub positional_help: &'static str,
    pub flags: &'static [FlagSpec],
    /// The states this verb refuses on, with their codes (§9).
    pub refuses: &'static [Refusal],
    /// Global flags this verb refuses by name (§4). Empty for every verb but
    /// `init`, which refuses `--repo`: the flag names a repository that already
    /// carries a `.ank/`, and `init` is what produces one.
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
    /// The task that carries the implementation, **while it does not exist**.
    /// It is therefore also the marker of an unrouted verb: a command that
    /// [`dispatch`] reaches clears the field, so the two never drift apart the
    /// way the module headers did.
    pub owner_task: Option<&'static str>,
}

/// The twelve verbs of §4, plus `init` and `help` (§9).
///
/// **The order is the specification's, and it is load-bearing**: `help` prints
/// this table in this order and adds nothing to it (ADR-c656cbcc33a9). §4 puts
/// the loop first — `context claim show log done`, then `release new find` —
/// and the rest after it, so sorting this list would erase the only structure
/// the listing has.
pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "context",
        coordinates: false,
        summary: "what binds this perimeter and what is claimable; with a claim held, the criterion and the constraints in full",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[flag("--limit")],
        refuses: &[],
        notes: &["a constraint is never truncated in execution mode; a cut is always announced"],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "claim",
        coordinates: true,
        summary: "takes the task and freezes its done_criteria by hash; refuses one held, blocked, or finished on another branch",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--criteria"), flag("--ttl")],
        refuses: &[
            refuses(4, "the task is held by another agent, or finished on another branch"),
            refuses(7, "the task is blocked, or has no done_criteria to freeze"),
        ],
        notes: &["--criteria sets a criterion the task does not have, and records it as the claimer's; it never replaces one"],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "show",
        coordinates: false,
        summary: "the entity whole, frontmatter and body, byte for byte",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        refuses: &[refuses(2, "no such entity, or the prefix matches more than one")],
        notes: &[],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "log",
        coordinates: true,
        summary: "an id alone reads the log; an id and a message appends one and renews the claim, which needs holding it",
        subcommands: &[],
        max_positionals: 2,
        // Both optional, and what is given decides which of the two things the
        // verb does: an id alone reads, a message writes (§4).
        positional_help: "[<id>] [<message>]",
        flags: &[],
        refuses: &[refuses(6, "writing with no claim held by this agent")],
        notes: &[],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "done",
        coordinates: true,
        summary: "runs the declared verifiers, records what ran, and moves the task to done; needs the claim, and a proof if nothing is declared",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<id>]",
        flags: &[flag("--proof")],
        refuses: &[
            refuses(5, "no proof, and no verifier declared to produce one"),
            refuses(6, "no claim held by this agent, or the frozen done_criteria has diverged"),
        ],
        notes: &["--proof is <type>:<ref>; type is commit, human-review, assertion or test"],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "release",
        coordinates: true,
        summary: "hands the task back, with the reason recorded in its log",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<id>]",
        flags: &[flag("--reason")],
        refuses: &[refuses(6, "no claim held by this agent")],
        notes: &[],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "new",
        coordinates: false,
        summary: "writes a task or an ADR that needs no hand finishing",
        subcommands: &["task", "adr"],
        max_positionals: 0,
        positional_help: "",
        flags: &[
            flag("--title"),
            multi("--scope"),
            flag("--criteria"),
            multi("--blocked-by"),
            flag("--constraint"),
            flag("--supersedes"),
            multi("--verify"),
            flag("--body"),
        ],
        refuses: &[refuses(9, "no --title or --scope and $EDITOR is unset, so there is nothing to open")],
        notes: &[
            "a scope is mandatory: an entity attached to nothing is invisible",
            "--body - reads the body from stdin, so a long one needs no shell quoting",
        ],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "find",
        coordinates: false,
        summary: "searches titles, scopes and criteria; --status open lists what remains, with no query",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<query>",
        flags: &[flag("--type"), flag("--status"), flag("--scope")],
        refuses: &[],
        notes: &["--status filters on the stored status; a claimed row still displays as [claimed:who]"],
        refuses_globals: &[],
        owner_task: None,
    },
    // After `find` and before `review`, which is where §4 puts it. Placing it
    // beside `graph` instead read as tidy and was wrong; `tests/skill.rs`
    // refused the commit until it moved (TASK-15336a0012d5).
    CommandSpec {
        name: "status",
        coordinates: false,
        summary: "where am I: branch, claim, perimeter, queue, findings",
        subcommands: &[],
        max_positionals: 0,
        positional_help: "",
        flags: &[],
        refuses: &[],
        notes: &[],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "review",
        coordinates: false,
        summary: "the ratification queue and the health of the corpus: what is proposed, and which scopes have gone dead",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        refuses: &[],
        notes: &[],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "accept",
        coordinates: true,
        summary: "promotes a proposed ADR to accepted, through a signed ratification commit; on the default branch only",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        refuses: &[
            refuses(2, "no such entity, or the prefix matches more than one"),
            refuses(7, "not on the default branch, and there is no way around it"),
            refuses(
                9,
                "the default branch cannot be determined, from config.yml or from origin",
            ),
        ],
        notes: &["the one act ank commits for; it is a human act, signed"],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "close",
        coordinates: true,
        summary: "closes a task that will never be done; --reason is mandatory",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--reason")],
        refuses: &[
            refuses(
                7,
                "no --reason: a closure nobody explained is one nobody can reopen",
            ),
            refuses(2, "no such entity, or the prefix matches more than one"),
        ],
        notes: &[],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "amend",
        coordinates: false,
        summary: "changes blocked_by, scope, and a done_criteria no live claim freezes",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[
            multi("--blocked-by"),
            multi("--drop-blocked-by"),
            multi("--scope"),
            multi("--drop-scope"),
            // Offered now, and it was `refused` here for as long as the verb
            // rejected it outright (TASK-84cfad83c308: help must not make an
            // offer the verb turns down). It stops being an offer the verb
            // rejects the moment the verb accepts it on state (§4).
            flag("--criteria"),
        ],
        refuses: &[refuses(
            6,
            "--criteria while a live claim freezes the criterion; that case is a release",
        )],
        notes: &[
            "adds and removes explicitly, never a replacement list, so nothing is dropped by being forgotten",
            "--criteria replaces the criterion outright, and leaves criteria_by where it stands",
        ],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "attest",
        coordinates: true,
        summary: "appends a proof to a finished task: the one write allowed after done",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--proof"), switch("--detached")],
        refuses: &[refuses(2, "no such entity, or the prefix matches more than one")],
        notes: &[
            "--proof is <type>:<ref>; type is commit, human-review, assertion or test",
            "--detached records the proof in refs/ank/proof/<id> and writes no file, so a pipeline anchors a run without a commit",
        ],
        refuses_globals: &[],
        owner_task: None,
    },
    // After `attest` and before `graph`: §4's order, and the last gap in it.
    // `tests/skill.rs` is what holds this to §4 rather than to memory.
    CommandSpec {
        name: "edit",
        coordinates: false,
        summary: "opens an entity in $EDITOR and validates what comes back",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        refuses: &[refuses(9, "$EDITOR is unset, and there is no editor to open")],
        notes: &[
            "$EDITOR is a command line run through sh, not a program name",
            "a GUI editor needs its wait flag, or it returns before you have typed and the file is written back unedited",
        ],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "graph",
        coordinates: false,
        summary: "the blocked_by DAG in readable text, indented under what blocks it",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        refuses: &[],
        notes: &[],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "scope",
        coordinates: false,
        summary: "what covers a path: the constraints that bind it and the tasks that touch it",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<path>",
        flags: &[],
        refuses: &[],
        notes: &[],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "check",
        coordinates: false,
        summary: "the mechanical invariants: parse, round-trip, references, frozen fields, orphaned claims",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        refuses: &[],
        notes: &["exit 8 means findings; a signal alone leaves it 0"],
        refuses_globals: &[],
        owner_task: None,
    },
    // After `check` and before `init`: §4's order. It sits beside the verb
    // that writes `config.yml` in the first place, which is the reading §9
    // states -- what `init` writes, `config` maintains.
    CommandSpec {
        name: "config",
        coordinates: false,
        summary: "reads and writes .ank/config.yml: the key alone reads, a value writes, --unset removes",
        subcommands: &[],
        max_positionals: 2,
        positional_help: "<key> [<value>]",
        flags: &[switch("--unset")],
        refuses: &[
            refuses(
                1,
                "a key the parser does not know, or a value in a form the surgery cannot edit safely",
            ),
            refuses(7, "verifiers.<name>.timeout on a verifier that is not declared"),
        ],
        notes: &[
            "keys: schema context_budget claim_ttl_max default_branch verifiers.<name>.run verifiers.<name>.timeout",
            "a resolved default prints marked as one; --json carries value and source as separate fields",
            "--unset verifiers.<name> removes a whole verifier, which is what makes declaring one reversible",
        ],
        refuses_globals: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "init",
        coordinates: true,
        summary: "creates .ank/ here or at <path>, writes config.yml, adds the refs/ank/* refspec; refuses --repo",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        refuses: &[refuses(
            1,
            "--repo: it names a repository that exists, and this verb makes one; the target is positional",
        )],
        notes: &["a target elsewhere is ank init <path>; with no argument it initialises the current directory"],
        refuses_globals: &["--repo"],
        owner_task: None,
    },
    CommandSpec {
        name: "help",
        coordinates: false,
        summary: "every verb in one flat listing, or one verb in full",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<verb>]",
        flags: &[],
        refuses: &[refuses(2, "no such verb; never a fallback to the general listing")],
        notes: &[],
        refuses_globals: &[],
        owner_task: None,
    },
];

pub fn spec_of(name: &str) -> Option<&'static CommandSpec> {
    COMMANDS.iter().find(|c| c.name == name)
}

fn known_flags(spec: &CommandSpec) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = spec.flags.iter().map(|f| f.name).collect();
    v.extend(GLOBAL_FLAGS.iter().map(|f| f.name));
    v.sort_unstable();
    v
}

fn find_flag(spec: &CommandSpec, name: &str) -> Option<FlagSpec> {
    spec.flags
        .iter()
        .chain(GLOBAL_FLAGS.iter())
        .find(|f| f.name == name)
        .copied()
}

// ---------------------------------------------------------------------------
// Parsed invocation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub command: &'static str,
    pub subcommand: Option<String>,
    pub positionals: Vec<String>,
    /// Values per flag. A switch carries an empty list.
    pub flags: BTreeMap<String, Vec<String>>,
    /// How this invocation is allowed to paint (§4).
    ///
    /// Not parsed — [`parse`] has no way to know what the process is attached
    /// to and leaves it [`style::PLAIN`], which is what keeps every unit test
    /// that builds an `Invocation` through the parser uncolored without saying
    /// so. [`dispatch`] is what fills it in, and what forces it back off under
    /// `--json`.
    pub style: crate::style::Style,
}

impl Invocation {
    pub fn has(&self, flag: &str) -> bool {
        self.flags.contains_key(flag)
    }

    pub fn value(&self, flag: &str) -> Option<&str> {
        self.flags.get(flag)?.last().map(|s| s.as_str())
    }

    pub fn values(&self, flag: &str) -> &[String] {
        self.flags.get(flag).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn json(&self) -> bool {
        self.has("--json")
    }

    pub fn quiet(&self) -> bool {
        self.has("--quiet")
    }

    pub fn repo(&self) -> Option<&str> {
        self.value("--repo")
    }

    pub fn style(&self) -> crate::style::Style {
        self.style
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// `name=value` split, for both forms: `--status=open` and `-s=open` reach it
/// with the dashes already accounted for by the caller.
fn split_inline(arg: &str) -> (String, Option<String>) {
    match arg.split_once('=') {
        Some((n, v)) => (n.to_string(), Some(v.to_string())),
        None => (arg.to_string(), None),
    }
}

/// A single-dash argument, resolved to the long flag it names (§4).
///
/// Bundling is refused rather than parsed. Accepting `-st` forces a decision
/// about `-sopen` — is that three flags, or `-s` with a value? — and every
/// answer to that is a guess about what the caller meant. The refusal costs one
/// round trip and names exactly what to type instead; a guess costs a wrong
/// answer that looks right.
fn short_flag(spec: &CommandSpec, arg: &str) -> Result<(String, String, Option<String>)> {
    // No flag contains whitespace, so this one is a positional that starts with
    // a dash -- `ank log "-1 rebuilt the index"`. Saying so beats reporting it
    // as a bundle of nine unknown letters, and it is a fact rather than a guess
    // about intent. Single-dash arguments became flags the day short forms did,
    // and that is the one consequence a caller has to be told rather than left
    // to discover.
    if arg.contains(char::is_whitespace) {
        return Err(
            CliError::new(1, format!("'{arg}' is not a flag: it contains a space"))
                .with_hint(format!("ank {} -- \"{arg}\"", spec.name)),
        );
    }

    let (letters, inline) = split_inline(&arg[1..]);
    let chars: Vec<char> = letters.chars().collect();

    if chars.len() > 1 {
        return Err(bundled(spec, &chars, &letters));
    }

    let unknown = |typed: &str| {
        CliError::new(1, format!("unknown flag '{typed}' for '{}'", spec.name))
            .with_hint(format!("valid flags: {}", known_flags(spec).join(" ")))
    };

    // `-` alone never reaches here, and `-=v` leaves nothing to resolve.
    let Some(c) = chars.first() else {
        return Err(unknown(arg));
    };
    let typed = format!("-{c}");
    let Some(long) = long_of(*c) else {
        return Err(unknown(&typed));
    };
    Ok((typed, long.to_string(), inline))
}

/// `-st`, and the two flags to type instead.
///
/// A letter that names nothing on this verb is reported as itself: naming a
/// flag the caller could type separately would be advice about a command that
/// would refuse too.
fn bundled(spec: &CommandSpec, chars: &[char], letters: &str) -> CliError {
    let mut parts: Vec<String> = Vec::new();
    for c in chars {
        match long_of(*c).and_then(|long| find_flag(spec, long)) {
            Some(fs) if fs.takes_value => parts.push(format!("-{c} <v>")),
            Some(_) => parts.push(format!("-{c}")),
            None => {
                return CliError::new(1, format!("unknown flag '-{c}' for '{}'", spec.name))
                    .with_hint(format!("valid flags: {}", known_flags(spec).join(" ")))
            }
        }
    }
    CliError::new(1, format!("'-{letters}' bundles short flags")).with_hint(format!(
        "ank {} {}",
        spec.name,
        parts.join(" ")
    ))
}

pub fn parse(argv: &[String]) -> Result<Invocation> {
    let Some(first) = argv.first() else {
        return Err(CliError::new(1, "no command").with_hint("ank context"));
    };

    let spec = spec_of(first).ok_or_else(|| {
        let names: Vec<&str> = COMMANDS.iter().map(|c| c.name).collect();
        CliError::new(1, format!("unknown command '{first}'"))
            .with_hint(format!("ank <{}>", names.join("|")))
    })?;

    let mut rest = &argv[1..];
    let mut subcommand = None;
    if !spec.subcommands.is_empty() {
        let sub =
            rest.first().ok_or_else(|| {
                CliError::new(1, format!("'{}' expects a subcommand", spec.name)).with_hint(
                    format!("ank {} <{}>", spec.name, spec.subcommands.join("|")),
                )
            })?;
        if !spec.subcommands.contains(&sub.as_str()) {
            return Err(CliError::new(
                1,
                format!("unknown subcommand '{sub}' for '{}'", spec.name),
            )
            .with_hint(format!(
                "ank {} <{}>",
                spec.name,
                spec.subcommands.join("|")
            )));
        }
        subcommand = Some(sub.clone());
        rest = &rest[1..];
    }

    let mut positionals: Vec<String> = Vec::new();
    let mut flags: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut i = 0usize;
    let mut terminated = false;

    while i < rest.len() {
        let arg = &rest[i];

        if !terminated && arg == "--" {
            // Everything that follows is positional, verbatim: this is the
            // only way to write a message that starts with a dash.
            terminated = true;
            i += 1;
            continue;
        }

        // `typed` is what the caller wrote and `name` is the long flag it
        // names; they differ only for a short form. Every error below quotes
        // `typed`, because an agent that typed `-s` is helped by an error about
        // `-s` and not about a flag it never wrote.
        let named: Option<(String, String, Option<String>)> = if terminated {
            None
        } else if arg.starts_with("--") {
            let (name, inline) = split_inline(arg);
            Some((name.clone(), name, inline))
        } else if arg.starts_with('-') && arg.len() > 1 {
            Some(short_flag(spec, arg)?)
        } else {
            None
        };

        let Some((typed, name, inline)) = named else {
            positionals.push(arg.clone());
            i += 1;
            continue;
        };

        let Some(fs) = find_flag(spec, &name) else {
            // A letter that names a real flag the verb does not take is a
            // different mistake from a letter that names nothing, and saying
            // which one it is saves the round trip.
            let message = if typed == name {
                format!("unknown flag '{typed}' for '{}'", spec.name)
            } else {
                format!("'{typed}' is {name}, which '{}' does not take", spec.name)
            };
            return Err(CliError::new(1, message)
                .with_hint(format!("valid flags: {}", known_flags(spec).join(" "))));
        };

        if !fs.takes_value {
            if inline.is_some() {
                return Err(CliError::new(1, format!("'{typed}' takes no value"))
                    .with_hint(format!("ank {} {typed}", spec.name)));
            }
            flags.entry(name).or_default();
            i += 1;
            continue;
        }

        let value = match inline {
            Some(v) => {
                i += 1;
                v
            }
            None => {
                let v = rest.get(i + 1).ok_or_else(|| {
                    CliError::new(1, format!("'{typed}' expects a value"))
                        .with_hint(format!("ank {} {typed} <value>", spec.name))
                })?;
                i += 2;
                v.clone()
            }
        };

        let slot = flags.entry(name).or_default();
        if !fs.repeatable {
            slot.clear();
        }
        slot.push(value);
    }

    if positionals.len() > spec.max_positionals {
        let extra = &positionals[spec.max_positionals];
        return Err(CliError::new(
            1,
            format!(
                "extra argument '{extra}': '{}' accepts {}",
                spec.name, spec.max_positionals
            ),
        )
        .with_hint(usage(spec)));
    }

    Ok(Invocation {
        command: spec.name,
        subcommand,
        positionals,
        flags,
        style: crate::style::PLAIN,
    })
}

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

// ---------------------------------------------------------------------------
// help (§9)
// ---------------------------------------------------------------------------

/// A flag as `help` shows it: the name alone says nothing about whether a value
/// follows, and an agent that guesses wrong pays a round trip to find out.
///
/// `with_short` is what separates the two surfaces of §9. `ank help <verb>`
/// shows both forms, since that is the call made to learn one verb precisely.
/// The flat listing shows neither: it carries a description per verb and sends
/// the flags here, which is the split, and printing a second spelling of every
/// flag in an overview would spend exactly what the split saves.
fn flag_display(f: &FlagSpec, with_short: bool) -> String {
    let name = match short_of(f.name) {
        Some(c) if with_short => format!("-{c}, {}", f.name),
        _ => f.name.to_string(),
    };
    match (f.takes_value, f.repeatable) {
        (false, _) => name,
        (true, false) => format!("{name} <v>"),
        (true, true) => format!("{name} <v>..."),
    }
}

/// The description, folded to the listing's width and indented under itself.
///
/// **One string, printed by both surfaces** (§9). The listing shows
/// `spec.summary` and so does `ank help <verb>`, which is what makes the
/// overview a compression rather than a second text: two strings would be two
/// things to keep true, and the one that drifts is the one nobody reads twice.
/// That drift is exactly how `amend` came to advertise a criterion edit the
/// binary refused always (TASK-84cfad83c308).
///
/// Folded on words and never mid-word, at a width that leaves the whole line
/// inside 100 columns. A description too long to fit is folded rather than cut:
/// truncation would put the clause a verb refuses on — always the tail of the
/// sentence — exactly where it disappears.
fn wrapped_summary(summary: &str, indent: usize, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in summary.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
        .into_iter()
        .enumerate()
        .map(|(i, l)| {
            if i == 0 {
                l
            } else {
                format!("{:indent$}{l}", "")
            }
        })
        .collect()
}

/// The flags `help` offers, which is not every flag the parser accepts (§9).
///
/// One filter, read by the flat listing, the per-verb page and `--json` alike:
/// three renderings of one surface, and a name hidden from one of them and not
/// the others would be the same defect in a quieter place.
fn listed_flags(spec: &CommandSpec) -> Vec<&'static FlagSpec> {
    spec.flags.iter().filter(|f| f.listed).collect()
}

/// The globals that apply to a verb, which is all of them but one (§4).
///
/// Same filter as [`listed_flags`], for the same reason and read by the same
/// three renderings: §9 says a flag the verb rejects by design belongs in the
/// refusals and not in the offer, and being global is not an exemption —
/// `ank help init` offering `--repo` is precisely the offer that let the flag
/// look supported while `init` wrote somewhere else (TASK-b8a12d60686d).
fn globals_of(spec: &CommandSpec) -> Vec<&'static FlagSpec> {
    GLOBAL_FLAGS
        .iter()
        .filter(|f| !spec.refuses_globals.contains(&f.name))
        .collect()
}

/// **The flat listing passes [`GLOBAL_FLAGS`] whole, and that is deliberate.**
/// It states what the three globals of §4 are, once, for the surface — the
/// exception belongs on the page of the verb that makes it, where a reader
/// asking about `init` is looking. Qualifying it in the flat listing would be a
/// second structure in an output ADR-c656cbcc33a9 keeps flat.
fn globals_line(globals: &[&'static FlagSpec], with_short: bool) -> String {
    globals
        .iter()
        .map(|f| flag_display(f, with_short))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Minimal JSON string escaping. The strings rendered here are `&'static str`
/// literals and none of them needs it today, which is precisely why it is
/// written rather than assumed: the day a verb or a flag carries a quote, the
/// output stays parseable instead of becoming a bug in whatever consumes it.
pub fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_of(specs: &[&CommandSpec]) -> String {
    let verbs: Vec<String> = specs
        .iter()
        .map(|spec| {
            let refusals: Vec<String> = spec
                .refuses
                .iter()
                .map(|r| format!("{{\"code\":{},\"when\":{}}}", r.code, json_str(r.when)))
                .collect();
            let notes: Vec<String> = spec.notes.iter().map(|n| json_str(n)).collect();
            let flags: Vec<String> = listed_flags(spec)
                .into_iter()
                .chain(globals_of(spec))
                .map(|f| {
                    // The short form is here and not only in the human listing:
                    // `--json` is how a script reads the surface, and a mapping
                    // it cannot see is a mapping it cannot use.
                    let short = match short_of(f.name) {
                        Some(c) => json_str(&format!("-{c}")),
                        None => "null".to_string(),
                    };
                    format!(
                        "{{\"name\":{},\"short\":{},\"takes_value\":{},\"repeatable\":{}}}",
                        json_str(f.name),
                        short,
                        f.takes_value,
                        f.repeatable
                    )
                })
                .collect();
            format!(
                "{{\"name\":{},\"usage\":{},\"summary\":{},\"flags\":[{}],\"notes\":[{}],\"refuses\":[{}]}}",
                json_str(spec.name),
                json_str(&usage(spec)),
                json_str(spec.summary),
                flags.join(","),
                notes.join(","),
                refusals.join(",")
            )
        })
        .collect();
    format!("{{\"verbs\":[{}]}}", verbs.join(","))
}

/// `ank help` and `ank help <verb>` (§9).
///
/// The listing is derived from [`COMMANDS`] on every call, so a verb cannot be
/// added without appearing here. An unknown verb is a **code 2** — "entity not
/// found" in the table of §4, the same code a missing task gets, because the
/// thing being looked up did not exist. It is never a fallback to the general
/// listing: an agent that asked about one verb and received all sixteen has to
/// work out that its question went unanswered, and answering the wrong question
/// silently is worse than refusing the wrong one loudly.
///
/// One pass, no grouping, no heading (ADR-c656cbcc33a9). The order is
/// [`COMMANDS`]', which is §4's, and it is the only structure the output has.
pub fn help(inv: &Invocation, out: &mut dyn Write) -> Result<i32> {
    let asked = inv.positionals.first();

    if let Some(name) = asked {
        let spec = spec_of(name).ok_or_else(|| {
            CliError::new(2, format!("no such verb '{name}'")).with_hint("ank help")
        })?;
        if inv.json() {
            let _ = writeln!(out, "{}", json_of(&[spec]));
            return Ok(0);
        }
        if inv.quiet() {
            return Ok(0);
        }
        let _ = writeln!(out, "{}", usage(spec));
        if !spec.summary.is_empty() {
            let _ = writeln!(out, "  {}", spec.summary);
        }
        let listed = listed_flags(spec);
        if !listed.is_empty() {
            let flags: Vec<String> = listed.iter().map(|f| flag_display(f, true)).collect();
            let _ = writeln!(out, "  flags:    {}", flags.join(" "));
        }
        let _ = writeln!(out, "  global:   {}", globals_line(&globals_of(spec), true));
        for (i, note) in spec.notes.iter().enumerate() {
            let label = if i == 0 { "note:" } else { "" };
            let _ = writeln!(out, "  {label:<9} {note}");
        }
        // What the verb refuses, and the code it comes back with (§9). The
        // question is asked before the call; the error is only available after.
        for (i, r) in spec.refuses.iter().enumerate() {
            let label = if i == 0 { "refuses:" } else { "" };
            let _ = writeln!(out, "  {label:<9} {} ({})", r.when, r.code);
        }
        return Ok(0);
    }

    let all: Vec<&CommandSpec> = COMMANDS.iter().collect();
    if inv.json() {
        let _ = writeln!(out, "{}", json_of(&all));
        return Ok(0);
    }
    if inv.quiet() {
        return Ok(0);
    }

    // One column for the usage, so the descriptions line up and the shape of
    // the surface is readable at a glance rather than one verb at a time.
    //
    // **The description takes the place of the flag names** (§9). They used to
    // sit here, which is the shape that says what none of them does: a bare
    // `--criteria` beside `amend` names a flag without saying what the verb is
    // for, and a caller choosing between twenty-one verbs was choosing on the
    // usage line alone. They are one `ank help <verb>` away, with their value
    // placeholders and the refusals that qualify them, and the trailer below
    // says so rather than leaving the reader to discover it.
    let width = COMMANDS.iter().map(|c| usage(c).len()).max().unwrap_or(0);
    let indent = width + 2;
    for spec in COMMANDS {
        if spec.summary.is_empty() {
            let _ = writeln!(out, "{}", usage(spec));
            continue;
        }
        for (i, line) in wrapped_summary(spec.summary, indent, 98 - indent)
            .into_iter()
            .enumerate()
        {
            if i == 0 {
                let _ = writeln!(out, "{:width$}  {line}", usage(spec));
            } else {
                let _ = writeln!(out, "{line}");
            }
        }
    }
    let all: Vec<&FlagSpec> = GLOBAL_FLAGS.iter().collect();
    let _ = writeln!(out, "\nglobal: {}", globals_line(&all, false));
    let _ = writeln!(
        out,
        "ank help <verb> for one verb: its flags, and what it refuses"
    );
    // A trailing pointer, beside the one above it and in the same shape: not a
    // heading and not a grouping, so the flat listing ADR-c656cbcc33a9 requires
    // is untouched. A flag nobody can discover answers nobody's question.
    let _ = writeln!(out, "ank --version for the build");
    Ok(0)
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// What the binary is, on one line (§4): the crate version, the commit
/// `build.rs` stamped in, and the revision of the `SKILL.md` it was built
/// alongside. `unknown` where there was no checkout, or no `skill/`, to ask.
///
/// The third value is the one a reader can act on with nothing else in hand
/// (TASK-ecda4070354f). The commit places the binary against the repository,
/// which is a thing to go and look at; the skill revision is compared against
/// `metadata.revision` in the file the agent has already loaded, so a stale
/// instruction set announces itself offline instead of costing an
/// investigation.
pub fn version_line() -> String {
    format!(
        "ank {} ({}, skill {})",
        env!("CARGO_PKG_VERSION"),
        env!("ANK_COMMIT"),
        env!("ANK_SKILL")
    )
}

fn not_implemented(spec: &CommandSpec) -> CliError {
    let task = spec.owner_task.unwrap_or("TASK-unknown");
    CliError::new(1, format!("'{}' is not implemented yet", spec.name))
        .with_hint(format!("ank show {task}"))
}

/// Entry point. Returns the exit code; never calls `exit` itself, so that it
/// stays testable.
pub fn run(
    argv: &[String],
    cwd: &std::path::Path,
    out: &mut dyn std::io::Write,
    style: crate::style::Style,
) -> i32 {
    match dispatch(argv, cwd, out, style) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}", err.render_styled(style.on_stderr()));
            err.code
        }
    }
}

/// The foundation shared by every verb except `init`, which by definition
/// precedes the existence of the repository.
///
/// The order matters and is not arbitrary: the repository is resolved before
/// git is checked, because a wrong `--repo` must be named as such and not
/// disguised as an environment problem. That resolution happens **before** the
/// rejection of an unimplemented verb: without it, `--repo` would be exercised
/// by no real path while no verb exists, and the foundation would be tested
/// without ever being reached.
struct Startup {
    repo: crate::repo::Repo,
    config: crate::config::Config,
    identity: String,
    /// Carried beside the value rather than asked for again where it is
    /// printed: the identity is resolved once, and a second reading of the
    /// environment would be a second answer free to disagree with the first.
    identity_source: crate::identity::Source,
}

/// Names the corpus when the walk crossed a git repository boundary
/// (TASK-2f01baf94632).
///
/// `discover` stops at the first `.ank/` and at nothing else, so from a
/// checkout nested inside another repository it silently resolves the outer
/// corpus. That is not merely reading the wrong files: claims are git refs
/// (ADR-4e7c), so they land in the resolved root's repository and not in the
/// one holding the code being changed. Measured on a fixture, the inner
/// repository ends with no ank ref at all.
///
/// **Only on the walk.** `--repo` is the caller saying which corpus they mean,
/// and warning about an answer that was asked for by name would fire forever in
/// the one layout this behaviour makes usable — a single `.ank/` above several
/// checkouts, scopes written as `repoA/src/**`. That layout was never designed
/// for and is not forbidden either; naming `--repo` once is what separates it
/// from the accident.
///
/// **On stderr**, and that is not incidental. It is not part of any verb's
/// answer, and §4 requires `--json` to stay byte-for-byte what a caller's
/// parser already reads; a line on stdout would break every one of them to say
/// something no parser asked for. It degrades and never fails (§2): the walk
/// succeeded, and the caller may well have meant it.
fn warn_if_outside_repository(inv: &Invocation, repo: &crate::repo::Repo, cwd: &std::path::Path) {
    if inv.repo().is_some() || inv.quiet() {
        return;
    }
    let here = crate::git::common_dir(cwd);
    let root = crate::git::common_dir(&repo.root);
    if !crate::git::crosses_repository(here.as_deref(), root.as_deref()) {
        return;
    }
    let style = inv.style().on_stderr();
    let tag = style.yellow("warning:");
    let root_display = repo.root.display();
    match here {
        Some(_) => eprintln!(
            "{tag} .ank/ resolved to {root_display}, outside the git repository holding {}",
            cwd.display()
        ),
        None => eprintln!(
            "{tag} .ank/ resolved to {root_display}; {} is in no git repository",
            cwd.display()
        ),
    }
    eprintln!(
        "  -> claims are refs and land in {root_display}: ank init here, or --repo to confirm"
    );
}

fn startup(inv: &Invocation, cwd: &std::path::Path) -> Result<Startup> {
    let repo = crate::repo::resolve(inv.repo(), cwd)?;
    // git is required by the verbs that coordinate, and never at startup
    // (ADR-9307e5d214a7, superseding ADR-b8884edcebe3). The gate used to stand
    // in front of the dispatch rather than in front of the operation, so `show`,
    // `find`, `graph`, `scope`, `new`, `amend` and the whole formal half of
    // `check` refused outside a repository although none of them touches a ref,
    // a commit or a branch.
    //
    // Repository resolution never needed git and does not gain it here:
    // `repo::discover` walks up for `.ank/` exactly as it always has.
    if spec_of(inv.command).is_some_and(|s| s.coordinates) {
        crate::git::ensure_usable(&repo.root)?;
    }
    warn_if_outside_repository(inv, &repo, cwd);
    let config = crate::config::load(&repo.config_path())?;
    let (identity, identity_source) = crate::identity::resolved();
    Ok(Startup {
        repo,
        config,
        identity,
        identity_source,
    })
}

/// Routes a parsed invocation to the module that owns the verb.
///
/// Every arm receives the same three things — the repository, the config and
/// the identity — resolved once by [`startup`]. A verb never resolves them
/// itself: the order in which they are established is a property of the
/// foundation, and a module doing it again would be free to get it wrong.
///
/// A verb whose module is still a stub falls through to [`not_implemented`],
/// which names the task that owns it. The arms therefore arrive one per task,
/// and the fall-through is the honest default rather than a placeholder: until
/// TASK-45d18f45de2c the fall-through was total, while six module headers
/// asserted the opposite.
fn dispatch(
    argv: &[String],
    cwd: &std::path::Path,
    out: &mut dyn std::io::Write,
    style: crate::style::Style,
) -> Result<i32> {
    // Before `parse`, and not as a flag on a verb (§4). `--version` replaces the
    // verb rather than modifying one, so the parser — which resolves a command
    // first and would reject this as an unknown one — never sees it. It is also
    // ahead of every check below for the reason `help` is: the caller who needs
    // it is holding an artifact they cannot identify, and a version that
    // demanded a healthy repository would go quiet exactly there.
    if argv.first().is_some_and(|a| a == "--version") {
        let _ = writeln!(out, "{}", version_line());
        return Ok(0);
    }
    let mut inv = parse(argv)?;
    let spec = spec_of(inv.command).expect("spec resolved during parsing");

    // The one gate, and the reason it is one. `--json` is never colored (§4),
    // and suppressing color at each printing site would be one chance per site
    // to forget one; suppressing it here makes "no escape sequence under
    // --json" a property of the invocation rather than of the discipline.
    //
    // This comment used to record that three verbs printed an unconditional
    // non-JSON line onto stdout while `--json` was set — `done`'s `running:`,
    // and the takeover warnings of `log` and `amend` — and treated it as a
    // given. It was not: §4 says `--json` stays byte-for-byte what a caller's
    // parser reads, and `ank done --json` emitted its progress line ahead of
    // the JSON document. All three now write to standard error
    // (TASK-2eefcdd80124), and `tests/cli.rs` walks the surface so a fourth
    // cannot arrive quietly.
    inv.style = if inv.json() {
        crate::style::PLAIN
    } else {
        style
    };

    // Three verbs run without the foundation. `init` precedes the existence of
    // the repository. `help` describes the surface rather than acting on it,
    // and the caller most in need of it is the one whose environment is wrong:
    // making `ank help` demand a `.ank/`, a git of 2.34, and a readable
    // `config.yml` would withhold the explanation exactly when it is needed.
    // `config` is the third and the sharpest case of the same reasoning
    // (ADR-e64dfaafd578): `startup` loads `config.yml` for every other verb, so
    // an unreadable one fails all of them — `check` included — and the verb
    // that exists to repair the file would be disabled by exactly the file it
    // repairs. It resolves the repository, because the file it edits lives in
    // one, and nothing else.
    if inv.command == "init" {
        return crate::init::run(&inv, cwd, out);
    }
    if inv.command == "help" {
        return help(&inv, out);
    }
    if inv.command == "config" {
        let repo = crate::repo::resolve(inv.repo(), cwd)?;
        // The same hazard, and it reaches this verb too: a `config` run from a
        // nested checkout edits the outer repository's configuration. `git` is
        // unchecked here, which costs nothing — `common_dir` answers `None`
        // when it cannot run, and two `None`s say nothing.
        warn_if_outside_repository(&inv, &repo, cwd);
        return crate::config::run(&inv, &repo, out);
    }

    let s = startup(&inv, cwd)?;
    match inv.command {
        "context" => crate::context::run(&inv, &s.repo, &s.config, &s.identity, out),
        "done" => crate::done::run(&inv, &s.repo, &s.config, &s.identity, out),
        "claim" => crate::claim::run(&inv, &s.repo, &s.config, &s.identity, out),
        "new" => crate::commands::new(&inv, &s.repo, &s.config, &s.identity, out),
        "find" => crate::commands::find(&inv, &s.repo, &s.config, &s.identity, out),
        "status" => crate::status::run(
            &inv,
            &s.repo,
            &s.config,
            &s.identity,
            s.identity_source,
            out,
        ),
        "graph" => crate::graph::run(&inv, &s.repo, out),
        "scope" => crate::commands::scope(&inv, &s.repo, &s.identity, out),
        "log" => crate::commands::log(&inv, &s.repo, &s.config, &s.identity, out),
        "release" => crate::commands::release(&inv, &s.repo, &s.identity, out),
        "check" => crate::human::check(&inv, &s.repo, &s.config, out),
        "review" => crate::human::review(&inv, &s.repo, &s.config, out),
        "accept" => crate::human::accept(&inv, &s.repo, &s.config, &s.identity, out),
        "close" => crate::human::close(&inv, &s.repo, &s.identity, out),
        "attest" => crate::human::attest(&inv, &s.repo, &s.identity, out),
        "edit" => crate::edit::run(&inv, &s.repo, out),
        "amend" => crate::human::amend(&inv, &s.repo, &s.identity, out),
        "show" => crate::human::show(&inv, &s.repo, out),
        _ => Err(not_implemented(spec)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    fn ok(s: &[&str]) -> Invocation {
        parse(&argv(s)).unwrap_or_else(|e| panic!("{s:?} should have parsed: {}", e.render()))
    }

    #[test]
    fn repo_in_both_of_its_forms() {
        let a = ok(&["check", "--repo=/path/to/project"]);
        let b = ok(&["check", "--repo", "/path/to/project"]);
        assert_eq!(a.repo(), Some("/path/to/project"));
        assert_eq!(b.repo(), Some("/path/to/project"));
        assert_eq!(a.flags, b.flags);
    }

    #[test]
    fn the_terminator_passes_the_message_through_intact() {
        // The case that motivates the terminator: a message starting with two
        // dashes, which would otherwise be read as a flag.
        let inv = ok(&["log", "--", "-- message"]);
        assert_eq!(inv.positionals, vec!["-- message".to_string()]);
        assert!(inv.flags.is_empty());

        // After the terminator, nothing is a flag any more.
        let inv = ok(&["log", "--", "--json"]);
        assert_eq!(inv.positionals, vec!["--json".to_string()]);
        assert!(!inv.json(), "--json after -- is text, not a flag");

        // Before the terminator, flags are still flags.
        let inv = ok(&["log", "--json", "--", "--repo"]);
        assert!(inv.json());
        assert_eq!(inv.positionals, vec!["--repo".to_string()]);
    }

    #[test]
    fn an_unknown_flag_names_the_valid_flags() {
        let err = parse(&argv(&["claim", "--tll", "30m"])).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("--tll"), "{}", err.message);
        let hint = err.hint.unwrap();
        for expected in ["--criteria", "--ttl", "--json", "--quiet", "--repo"] {
            assert!(hint.contains(expected), "{expected} missing from: {hint}");
        }
    }

    #[test]
    fn a_missing_value_after_a_flag_that_expects_one() {
        let err = parse(&argv(&["claim", "8f3a", "--ttl"])).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("--ttl"), "{}", err.message);
        assert_eq!(err.hint.as_deref(), Some("ank claim --ttl <value>"));

        // A global flag is not a special case.
        let err = parse(&argv(&["check", "--repo"])).unwrap_err();
        assert!(err.message.contains("--repo"), "{}", err.message);
    }

    #[test]
    fn json_is_accepted_on_every_command_without_exception() {
        // Invariant of the specification: full scriptability is not an option.
        // The test walks the table, so a verb added without --json would make
        // it fail.
        for spec in COMMANDS {
            let mut a = vec![spec.name.to_string()];
            if let Some(sub) = spec.subcommands.first() {
                a.push(sub.to_string());
            }
            a.push("--json".to_string());
            let inv = parse(&a)
                .unwrap_or_else(|e| panic!("--json refused on {}: {}", spec.name, e.render()));
            assert!(inv.json(), "--json not retained on {}", spec.name);
        }
        // A count, so that a verb added without a thought about `--json` fails
        // here. What §4 lists and what this table holds is a stronger question
        // and has its own test, in `tests/skill.rs`, which reads §4 rather than
        // counting.
        assert_eq!(
            COMMANDS.len(),
            21,
            "every verb of §4, plus init and help from §9. The surface is \
             complete, so this number moves only when §4 does"
        );
    }

    #[test]
    fn an_extra_positional_is_refused_never_ignored() {
        let err = parse(&argv(&["show", "8f3a", "51c2"])).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("51c2"), "{}", err.message);
        assert_eq!(err.hint.as_deref(), Some("ank show <id>"));

        // `log` accepts two: [<id>] <message>.
        assert_eq!(ok(&["log", "8f3a", "message"]).positionals.len(), 2);
        assert!(parse(&argv(&["log", "8f3a", "message", "extra"])).is_err());
    }

    #[test]
    fn an_unknown_command_lists_the_commands() {
        let err = parse(&argv(&["claimm"])).unwrap_err();
        assert!(err.message.contains("claimm"), "{}", err.message);
        let hint = err.hint.unwrap();
        assert!(hint.contains("claim") && hint.contains("context"), "{hint}");
    }

    #[test]
    fn new_requires_its_subcommand() {
        let err = parse(&argv(&["new"])).unwrap_err();
        assert_eq!(err.hint.as_deref(), Some("ank new <task|adr>"));

        let err = parse(&argv(&["new", "epic"])).unwrap_err();
        assert!(err.message.contains("epic"), "{}", err.message);

        let inv = ok(&["new", "task", "--title", "T", "--scope", "src/**"]);
        assert_eq!(inv.subcommand.as_deref(), Some("task"));
    }

    #[test]
    fn scope_is_repeatable_and_the_others_are_not() {
        let inv = ok(&[
            "new", "task", "--scope", "src/**", "--scope", "tests/**", "--title", "A", "--title",
            "B",
        ]);
        assert_eq!(inv.values("--scope"), ["src/**", "tests/**"]);
        // A non-repeatable flag keeps the last value, without accumulating.
        assert_eq!(inv.values("--title"), ["B"]);
    }

    #[test]
    fn a_switch_refuses_an_attached_value() {
        let err = parse(&argv(&["check", "--json=yes"])).unwrap_err();
        assert!(err.message.contains("takes no value"), "{}", err.message);
    }

    #[test]
    fn unimplemented_verbs_name_their_task() {
        for spec in COMMANDS.iter().filter(|c| c.owner_task.is_some()) {
            let err = not_implemented(spec);
            assert_eq!(err.code, 1);
            let hint = err.hint.unwrap();
            assert!(hint.contains("TASK-"), "{}: {hint}", spec.name);
        }
    }

    #[test]
    fn a_routed_verb_carries_no_owner_task() {
        // `owner_task` is what `not_implemented` names, so leaving it set on a
        // verb dispatch reaches would advertise an implementation as missing
        // while it runs. Exactly the drift this task existed to fix, in the
        // opposite direction: `init`, `claim` and `context` are the verbs
        // routed today, and all must be clear of it.
        for routed in [
            "init", "help", "config", "claim", "context", "done", "log", "release", "new", "find",
            "attest", "amend", "show", "edit",
        ] {
            assert_eq!(
                spec_of(routed).unwrap().owner_task,
                None,
                "{routed} is routed by dispatch"
            );
        }
        // Every verb of the surface is routed now, so none may carry the
        // field any more. The day one is added, it arrives unrouted and this
        // fails until its arm exists.
        assert!(
            COMMANDS.iter().all(|c| c.owner_task.is_none()),
            "an unrouted verb must name its task, a routed one must not"
        );
    }

    #[test]
    fn the_foundation_is_resolved_before_the_verb_runs() {
        // A wrong --repo is named as such and never disguised as a broken
        // environment, which is why `startup` resolves the repository before
        // checking git.
        let mut out = Vec::new();
        let code = run(
            &argv(&["check", "--repo", "/path/that/does/not/exist"]),
            std::path::Path::new("."),
            &mut out,
            crate::style::PLAIN,
        );
        assert_eq!(code, 1);

        // A valid --repo crosses the foundation and reaches the verb. This
        // used to assert that `check` was unimplemented; every verb of the
        // surface is routed now, so what it asserts instead is that the verb
        // answered — 0 or 8 from `check`, never the 1 of a rejection.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let mut out = Vec::new();
        let code = run(
            &argv(&["check", "--repo", root.to_str().unwrap(), "--quiet"]),
            std::path::Path::new("."),
            &mut out,
            crate::style::PLAIN,
        );
        assert!(code == 0 || code == 8, "check answered with {code}");
    }

    #[test]
    fn init_does_not_go_through_the_foundation() {
        // init precedes the existence of the repository: requiring a `.ank/`
        // beforehand would make the command useless for what it is for.
        let inv = ok(&["init"]);
        assert_eq!(inv.command, "init");
        assert!(spec_of("init").unwrap().owner_task.is_none());
    }

    fn help_out(args: &[&str]) -> String {
        let inv = ok(args);
        let mut out = Vec::new();
        let code = help(&inv, &mut out).unwrap_or_else(|e| panic!("{args:?}: {}", e.render()));
        assert_eq!(code, 0, "{args:?}");
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn help_lists_every_verb_of_the_table() {
        // Walks COMMANDS rather than a written-out list: a verb added later
        // that the renderer forgets fails here, which is the whole reason the
        // listing is derived from the table instead of maintained beside it.
        //
        // The flags are asserted on the per-verb page and no longer in the
        // listing, which is where they moved when the description took their
        // place (§9, TASK-fe130d2b732c). The guarantee is unchanged — a flag
        // the renderer forgets still fails here — and it now watches the
        // surface that actually carries them, with their value placeholders.
        let text = help_out(&["help"]);
        // Whitespace collapsed, because a description too long for the column
        // is folded across lines: the fold is presentation, and asserting on
        // the text means asserting on the words rather than on where they broke.
        let flowed = text.split_whitespace().collect::<Vec<_>>().join(" ");
        for spec in COMMANDS {
            assert!(
                text.contains(&usage(spec)),
                "{} missing from the listing:\n{text}",
                spec.name
            );
            assert!(
                flowed.contains(
                    &spec
                        .summary
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                "{} is listed without its description:\n{text}",
                spec.name
            );
            let page = help_out(&["help", spec.name]);
            for f in listed_flags(spec) {
                assert!(
                    page.contains(f.name),
                    "{} of {} missing from its page:\n{page}",
                    f.name,
                    spec.name
                );
            }
        }
    }

    #[test]
    fn the_listing_follows_the_table_and_puts_nothing_above_it() {
        // With the headings gone, the order of COMMANDS is the only structure
        // the listing has (ADR-c656cbcc33a9) -- so it is asserted rather than
        // assumed. The test above passes just as well on a renderer that sorts
        // alphabetically, which would bury the loop in the middle.
        let text = help_out(&["help"]);
        let mut at = 0usize;
        for spec in COMMANDS {
            let u = usage(spec);
            let found = text[at..]
                .find(&u)
                .unwrap_or_else(|| panic!("{} out of order or missing:\n{text}", spec.name));
            at += found + u.len();
        }
        assert!(
            text.starts_with(&usage(&COMMANDS[0])),
            "the first line is not the first verb, so something groups or \
             titles the listing:\n{text}"
        );
    }

    #[test]
    fn help_for_one_verb_answers_about_that_verb_alone() {
        let text = help_out(&["help", "claim"]);
        assert!(text.contains("ank claim <id>"), "{text}");
        // The flags carry their value placeholder here, which is the detail §9
        // moves out of SKILL.md and into this command.
        assert!(text.contains("--ttl <v>"), "{text}");
        assert!(text.contains("--criteria <v>"), "{text}");
        assert!(
            !text.contains("audience"),
            "the audience line is what ADR-c656cbcc33a9 removes:\n{text}"
        );
        // One verb means one verb: no other usage line rides along.
        assert!(!text.contains("ank accept"), "{text}");

        // A repeatable flag is shown as repeatable.
        let text = help_out(&["help", "new"]);
        assert!(text.contains("--scope <v>..."), "{text}");
    }

    #[test]
    fn an_unknown_verb_passed_to_help_is_a_two_and_never_the_listing() {
        let inv = ok(&["help", "clam"]);
        let mut out = Vec::new();
        let err = help(&inv, &mut out).unwrap_err();
        assert_eq!(err.code, 2, "entity not found, per the table of §4");
        assert!(err.message.contains("clam"), "{}", err.message);
        assert_eq!(err.hint.as_deref(), Some("ank help"));
        assert!(
            out.is_empty(),
            "a silent fallback to the general listing: {}",
            String::from_utf8_lossy(&out)
        );
    }

    #[test]
    fn help_speaks_json_in_both_of_its_forms() {
        let all = help_out(&["help", "--json"]);
        assert!(all.starts_with("{\"verbs\":["), "{all}");
        for spec in COMMANDS {
            assert!(
                all.contains(&format!("\"name\":\"{}\"", spec.name)),
                "{all}"
            );
        }
        assert!(
            !all.contains("audience"),
            "the audience key carried the grouping into the scripted \
             output:\n{all}"
        );
        assert!(all.contains("\"takes_value\":false"), "{all}");

        let one = help_out(&["help", "claim", "--json"]);
        assert!(one.contains("\"name\":\"claim\""), "{one}");
        assert!(!one.contains("\"name\":\"accept\""), "{one}");
        // --json carries the globals too: they are part of what the verb takes.
        assert!(one.contains("\"name\":\"--repo\""), "{one}");
    }

    #[test]
    fn quiet_help_says_nothing_and_still_exits_zero() {
        assert_eq!(help_out(&["help", "--quiet"]), "");
        assert_eq!(help_out(&["help", "claim", "--quiet"]), "");
    }

    #[test]
    fn error_rendering_follows_the_shape_in_the_spec() {
        let err = CliError::new(7, "TASK-51c2 has no done_criteria")
            .with_hint("ank claim 51c2 --criteria \"<verifiable criterion>\"");
        assert_eq!(
            err.render(),
            "error[7]: TASK-51c2 has no done_criteria\n  -> ank claim 51c2 --criteria \"<verifiable criterion>\""
        );
        assert_eq!(
            CliError::new(1, "no next step").render(),
            "error[1]: no next step"
        );
    }
}
