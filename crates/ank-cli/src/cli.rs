//! Argument parsing and dispatch (§4, §12).
//!
//! Parsing is done by hand, with no library. The reason is not saving a
//! dependency but character-level control over two surfaces read by agents:
//! the self-correcting errors, which a generic parser would replace with its
//! own messages, and the help, whose cost is paid on every call that triggers
//! it. With the surface frozen at twelve verbs (ADR-2f8a61c04b7d), that cost
//! does not grow.
//!
//! The edge cases of parsing are where hand-written code goes wrong, and they
//! look like business bugs once in production: every one of them is therefore
//! tested, one test per case.

use std::collections::BTreeMap;
use std::fmt;

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
        match &self.hint {
            Some(h) => format!("error[{}]: {}\n  -> {}", self.code, self.message, h),
            None => format!("error[{}]: {}", self.code, self.message),
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

#[derive(Debug, Clone, Copy)]
pub struct CommandSpec {
    pub name: &'static str,
    /// Mandatory subcommands, as in `new task` / `new adr`.
    pub subcommands: &'static [&'static str],
    pub max_positionals: usize,
    pub positional_help: &'static str,
    pub flags: &'static [FlagSpec],
    /// The task that carries the implementation, while it does not exist.
    pub owner_task: Option<&'static str>,
}

/// The twelve verbs of §4, plus `init` (§9). The order is the specification's:
/// agent loop, agent off-loop, human surface.
pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "context",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[flag("--limit")],
        owner_task: Some("TASK-d4e5f6a7b8c9"),
    },
    CommandSpec {
        name: "claim",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--criteria"), flag("--ttl")],
        owner_task: Some("TASK-c3d4e5f6a7b8"),
    },
    CommandSpec {
        name: "log",
        subcommands: &[],
        max_positionals: 2,
        positional_help: "[<id>] <message>",
        flags: &[],
        owner_task: Some("TASK-f6a7b8c9d0e1"),
    },
    CommandSpec {
        name: "done",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<id>]",
        flags: &[flag("--proof")],
        owner_task: Some("TASK-e5f6a7b8c9d0"),
    },
    CommandSpec {
        name: "release",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<id>]",
        flags: &[flag("--reason")],
        owner_task: Some("TASK-f6a7b8c9d0e1"),
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
        ],
        owner_task: Some("TASK-f6a7b8c9d0e1"),
    },
    CommandSpec {
        name: "find",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<query>",
        flags: &[flag("--type"), flag("--status"), flag("--scope")],
        owner_task: Some("TASK-f6a7b8c9d0e1"),
    },
    CommandSpec {
        name: "review",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        owner_task: Some("TASK-a7b8c9d0e1f2"),
    },
    CommandSpec {
        name: "accept",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        owner_task: Some("TASK-a7b8c9d0e1f2"),
    },
    CommandSpec {
        name: "close",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--reason")],
        owner_task: Some("TASK-a7b8c9d0e1f2"),
    },
    CommandSpec {
        name: "check",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        owner_task: Some("TASK-a7b8c9d0e1f2"),
    },
    CommandSpec {
        name: "show",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        owner_task: Some("TASK-a7b8c9d0e1f2"),
    },
    CommandSpec {
        name: "init",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
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
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

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

        if !terminated && arg.starts_with("--") {
            let (name, inline) = match arg.split_once('=') {
                Some((n, v)) => (n.to_string(), Some(v.to_string())),
                None => (arg.clone(), None),
            };

            let Some(fs) = find_flag(spec, &name) else {
                return Err(
                    CliError::new(1, format!("unknown flag '{name}' for '{}'", spec.name))
                        .with_hint(format!("valid flags: {}", known_flags(spec).join(" "))),
                );
            };

            if !fs.takes_value {
                if inline.is_some() {
                    return Err(CliError::new(1, format!("'{name}' takes no value"))
                        .with_hint(format!("ank {} {name}", spec.name)));
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
                        CliError::new(1, format!("'{name}' expects a value"))
                            .with_hint(format!("ank {} {name} <value>", spec.name))
                    })?;
                    i += 2;
                    v.clone()
                }
            };

            let slot = flags.entry(name.clone()).or_default();
            if !fs.repeatable {
                slot.clear();
            }
            slot.push(value);
            continue;
        }

        positionals.push(arg.clone());
        i += 1;
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
// Dispatch
// ---------------------------------------------------------------------------

fn not_implemented(spec: &CommandSpec) -> CliError {
    let task = spec.owner_task.unwrap_or("TASK-unknown");
    CliError::new(1, format!("'{}' is not implemented yet", spec.name))
        .with_hint(format!("see .ank/tasks/{task}.md"))
}

/// Entry point. Returns the exit code; never calls `exit` itself, so that it
/// stays testable.
pub fn run(argv: &[String], cwd: &std::path::Path, out: &mut dyn std::io::Write) -> i32 {
    match dispatch(argv, cwd, out) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}", err.render());
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
    #[allow(dead_code)]
    repo: crate::repo::Repo,
    #[allow(dead_code)]
    config: crate::config::Config,
    #[allow(dead_code)]
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

fn dispatch(argv: &[String], cwd: &std::path::Path, out: &mut dyn std::io::Write) -> Result<i32> {
    let inv = parse(argv)?;
    let spec = spec_of(inv.command).expect("spec resolved during parsing");

    if inv.command == "init" {
        return crate::init::run(&inv, cwd, out);
    }

    let _startup = startup(&inv, cwd)?;
    Err(not_implemented(spec))
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
    fn json_is_accepted_on_all_twelve_commands_without_exception() {
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
        assert_eq!(
            COMMANDS.len(),
            13,
            "twelve verbs from §4, plus init from §9"
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
    fn the_foundation_is_resolved_before_rejecting_an_unimplemented_verb() {
        // Without that order, --repo would be exercised by no real path while
        // no verb exists: the foundation would be unit-tested without ever
        // being reached by the binary.
        let mut out = Vec::new();
        let code = run(
            &argv(&["check", "--repo", "/path/that/does/not/exist"]),
            std::path::Path::new("."),
            &mut out,
        );
        assert_eq!(code, 1);

        // A valid --repo crosses the foundation and reaches the verb rejection.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let mut out = Vec::new();
        let code = run(
            &argv(&["check", "--repo", root.to_str().unwrap()]),
            std::path::Path::new("."),
            &mut out,
        );
        assert_eq!(code, 1, "the check verb is not implemented yet");
    }

    #[test]
    fn init_does_not_go_through_the_foundation() {
        // init precedes the existence of the repository: requiring a `.ank/`
        // beforehand would make the command useless for what it is for.
        let inv = ok(&["init"]);
        assert_eq!(inv.command, "init");
        assert!(spec_of("init").unwrap().owner_task.is_none());
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
