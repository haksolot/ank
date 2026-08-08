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
}

const fn flag(name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        takes_value: true,
        repeatable: false,
    }
}

const fn switch(name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        takes_value: false,
        repeatable: false,
    }
}

const fn multi(name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        takes_value: true,
        repeatable: true,
    }
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
    /// Mandatory subcommands, as in `new task` / `new adr`.
    pub subcommands: &'static [&'static str],
    pub max_positionals: usize,
    pub positional_help: &'static str,
    pub flags: &'static [FlagSpec],
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
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[flag("--limit")],
        owner_task: None,
    },
    CommandSpec {
        name: "claim",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--criteria"), flag("--ttl")],
        owner_task: None,
    },
    CommandSpec {
        name: "show",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "log",
        subcommands: &[],
        max_positionals: 2,
        // Both optional, and what is given decides which of the two things the
        // verb does: an id alone reads, a message writes (§4).
        positional_help: "[<id>] [<message>]",
        flags: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "done",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<id>]",
        flags: &[flag("--proof")],
        owner_task: None,
    },
    CommandSpec {
        name: "release",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<id>]",
        flags: &[flag("--reason")],
        owner_task: None,
    },
    CommandSpec {
        name: "new",
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
        owner_task: None,
    },
    CommandSpec {
        name: "find",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<query>",
        flags: &[flag("--type"), flag("--status"), flag("--scope")],
        owner_task: None,
    },
    // After `find` and before `review`, which is where §4 puts it. Placing it
    // beside `graph` instead read as tidy and was wrong; `tests/skill.rs`
    // refused the commit until it moved (TASK-15336a0012d5).
    CommandSpec {
        name: "status",
        subcommands: &[],
        max_positionals: 0,
        positional_help: "",
        flags: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "review",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "accept",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "close",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--reason")],
        owner_task: None,
    },
    CommandSpec {
        name: "amend",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[
            multi("--blocked-by"),
            multi("--drop-blocked-by"),
            multi("--scope"),
            multi("--drop-scope"),
            // Declared only so it can be refused by name. The parser's "unknown
            // flag" would list the valid ones and leave the caller to work out
            // why the obvious one is missing; §4 wants the exact command to run
            // next, and for a frozen criterion that command is `ank release`.
            flag("--criteria"),
        ],
        owner_task: None,
    },
    CommandSpec {
        name: "attest",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--proof")],
        owner_task: None,
    },
    // After `attest` and before `graph`: §4's order, and the last gap in it.
    // `tests/skill.rs` is what holds this to §4 rather than to memory.
    CommandSpec {
        name: "edit",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "graph",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "scope",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<path>",
        flags: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "check",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "init",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        owner_task: None,
    },
    CommandSpec {
        name: "help",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<verb>]",
        flags: &[],
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
/// The flat listing does not, and stays byte for byte what it was: it buys its
/// token economy by being short, and spending it on a second spelling of every
/// flag would spend exactly what the split saves.
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

/// Bare names, for the general listing. §9 buys its token economy by keeping the
/// overview short and sending the detail to `ank help <verb>`; spelling out every
/// value placeholder in the listing would spend exactly what the split saves.
fn flag_names(spec: &CommandSpec) -> String {
    spec.flags
        .iter()
        .map(|f| f.name)
        .collect::<Vec<_>>()
        .join(" ")
}

fn globals_line(with_short: bool) -> String {
    GLOBAL_FLAGS
        .iter()
        .map(|f| flag_display(f, with_short))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Minimal JSON string escaping. The strings rendered here are `&'static str`
/// literals and none of them needs it today, which is precisely why it is
/// written rather than assumed: the day a verb or a flag carries a quote, the
/// output stays parseable instead of becoming a bug in whatever consumes it.
fn json_str(s: &str) -> String {
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
            let flags: Vec<String> = spec
                .flags
                .iter()
                .chain(GLOBAL_FLAGS.iter())
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
                "{{\"name\":{},\"usage\":{},\"flags\":[{}]}}",
                json_str(spec.name),
                json_str(&usage(spec)),
                flags.join(",")
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
        if !spec.flags.is_empty() {
            let flags: Vec<String> = spec.flags.iter().map(|f| flag_display(f, true)).collect();
            let _ = writeln!(out, "  flags:    {}", flags.join(" "));
        }
        let _ = writeln!(out, "  global:   {}", globals_line(true));
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

    // One column for the usage, so the flags line up and the shape of the
    // surface is readable at a glance rather than one verb at a time.
    let width = COMMANDS.iter().map(|c| usage(c).len()).max().unwrap_or(0);
    for spec in COMMANDS {
        let names = flag_names(spec);
        if names.is_empty() {
            let _ = writeln!(out, "{}", usage(spec));
        } else {
            let _ = writeln!(out, "{:width$}  {names}", usage(spec));
        }
    }
    let _ = writeln!(out, "\nglobal: {}", globals_line(false));
    let _ = writeln!(out, "ank help <verb> for one verb");
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
        .with_hint(format!("see .ank/tasks/{task}.md"))
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
}

fn startup(inv: &Invocation, cwd: &std::path::Path) -> Result<Startup> {
    let repo = crate::repo::resolve(inv.repo(), cwd)?;
    // git is a hard dependency, and its version is checked at startup
    // (ADR-b8884edcebe3).
    crate::git::ensure_usable(&repo.root)?;
    let config = crate::config::load(&repo.config_path())?;
    let identity = crate::identity::resolve();
    Ok(Startup {
        repo,
        config,
        identity,
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
    // and three verbs print an unconditional non-JSON line onto stdout while
    // it is set — `done`'s `running:`, and the takeover warnings of `log` and
    // `amend`. Suppressing color at each printing site would be three chances
    // to forget one; suppressing it here makes "no escape sequence under
    // --json" a property of the invocation rather than of the discipline.
    inv.style = if inv.json() {
        crate::style::PLAIN
    } else {
        style
    };

    // Two verbs run without the foundation. `init` precedes the existence of
    // the repository. `help` describes the surface rather than acting on it,
    // and the caller most in need of it is the one whose environment is wrong:
    // making `ank help` demand a `.ank/`, a git of 2.34, and a readable
    // `config.yml` would withhold the explanation exactly when it is needed.
    if inv.command == "init" {
        return crate::init::run(&inv, cwd, out);
    }
    if inv.command == "help" {
        return help(&inv, out);
    }

    let s = startup(&inv, cwd)?;
    match inv.command {
        "context" => crate::context::run(&inv, &s.repo, &s.config, &s.identity, out),
        "done" => crate::done::run(&inv, &s.repo, &s.config, &s.identity, out),
        "claim" => crate::claim::run(&inv, &s.repo, &s.config, &s.identity, out),
        "new" => crate::commands::new(&inv, &s.repo, &s.config, &s.identity, out),
        "find" => crate::commands::find(&inv, &s.repo, &s.config, &s.identity, out),
        "status" => crate::status::run(&inv, &s.repo, &s.config, &s.identity, out),
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
            20,
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
            "init", "help", "claim", "context", "done", "log", "release", "new", "find", "attest",
            "amend", "show", "edit",
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
        let text = help_out(&["help"]);
        for spec in COMMANDS {
            assert!(
                text.contains(&usage(spec)),
                "{} missing from the listing:\n{text}",
                spec.name
            );
            for f in spec.flags {
                assert!(
                    text.contains(f.name),
                    "{} of {} missing:\n{text}",
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
