//! `hooks`: the entries an agent harness calls at its own events, installed
//! into that harness's user-level configuration, taken out again, and answered
//! (ADR-89c1e7146fe5, ADR-43cbb20dcbf1).
//!
//! **The binary is the handler.** What `install` writes is an entry whose
//! command runs this executable, by its absolute path, as `hooks run <agent>
//! <event>`: no script is written beside it, so there is nothing to keep in
//! step with the binary but the binary.
//!
//! **The file is somebody else's, so it is edited and never rewritten.** Claude
//! Code's `settings.json` is kept by hand and written into by other tools, and
//! the decision asks that every byte ank did not add stays where it was. So the
//! file is never round-tripped through a serializer: it is parsed for the spans
//! of its values, and an edit is one insertion or one deletion of text at a
//! position. `remove` is the inverse of `install` and not a cleanup: it deletes
//! exactly the span `install` inserted, which is what gives the file back byte
//! for byte.
//!
//! **What install created is told apart by its form.** An event array, or the
//! `hooks` object, that `install` had to create is written on one line; an
//! entry added to a container that already existed is written across lines. A
//! container its owner left empty (`"SessionStart": []`, `"hooks": {}`) would
//! otherwise come out of `install` looking exactly like one ank created, and
//! `remove` could not know whether to take the container out with the entry.
//!
//! **One protocol decides** (ADR-b108b24202ed). `run` translates each agent's
//! payload into one call, `event`, `tool`, `paths` and `cwd`, answers it with
//! one decision, `allow`, `deny` or `context`, and translates that back into
//! the agent's own bytes. `generic` is the identity translation, which any
//! harness with a hook can call; `claude` is a translation onto it whose bytes
//! are the ones it wrote before the protocol existed.

use crate::cli::{CliError, Invocation, Result};
use crate::config::{Enforce, GuardCorpus, GuardEdit};
use crate::json::Obj;
use ank_contract::ExitCode;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// The agents this version installs for (ADR-43cbb20dcbf1): another arrives by
/// an amendment of that decision, and is a row here.
const AGENTS: &[&str] = &["claude"];

/// Claude Code's events, each with the event `run` is called with and the tools
/// its entry is matched against, if any.
///
/// `PreToolUse` is matched to the file tools, which the two file guards look
/// at, and to `Bash`, which the permissions guard looks at (ADR-ddbe024ca7f2).
/// Still a list and never every tool: a tool none of the guards reads would
/// start this binary to say nothing.
const EVENTS: &[(&str, &str, Option<&str>)] = &[
    ("SessionStart", "session-start", None),
    (
        "PreToolUse",
        "pre-tool-use",
        Some("Read|Write|Edit|MultiEdit|NotebookEdit|Glob|Grep|Bash"),
    ),
];

/// The matchers an earlier install wrote, by event: install moves one to the
/// current matcher in place, and remove recognises it as install's, so an entry
/// written before shell calls were guarded is upgraded and taken out whole
/// (ADR-43cbb20dcbf1).
const PREVIOUS_MATCHERS: &[(&str, &str)] = &[(
    "PreToolUse",
    "Read|Write|Edit|MultiEdit|NotebookEdit|Glob|Grep",
)];

/// The matchers `event`'s entry may carry and still be install's: the current
/// one first, then the earlier ones.
fn matchers_of(name: &str, current: Option<&'static str>) -> Vec<Option<&'static str>> {
    let mut all = vec![current];
    all.extend(
        PREVIOUS_MATCHERS
            .iter()
            .filter(|(n, _)| *n == name)
            .map(|(_, m)| Some(*m)),
    );
    all
}

pub fn run(inv: &Invocation, cwd: &Path, out: &mut dyn Write) -> Result<ExitCode> {
    let sub = inv.subcommand.as_deref().unwrap_or_default();
    if sub == "run" {
        if inv.json() {
            return Err(CliError::new(
                ExitCode::Generic,
                "--json: hooks run returns no document, what it prints is the harness's",
            )
            .with_hint("ank hooks run claude <event>"));
        }
        // 0 whatever it is handed, and silent wherever it has nothing to serve:
        // a hook that fails is a harness that blocks (ADR-43cbb20dcbf1).
        if let Some(served) = serve(inv, cwd) {
            let _ = writeln!(out, "{served}");
        }
        return Ok(ExitCode::Ok);
    }
    if sub == "status" {
        return status(inv, cwd, out);
    }
    let agent = agent(inv, sub)?;
    if let Some(extra) = inv.positionals.get(1) {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "extra argument '{extra}': hooks {sub} writes every event's entry and takes none"
            ),
        )
        .with_hint(format!("ank hooks {sub} {agent}")));
    }
    let path = settings_path()?;
    let before = read(&path)?;
    let command =
        |event: &str| -> Result<String> { Ok(format!("{} hooks run {agent} {event}", exe()?)) };
    let (after, events) = if sub == "install" {
        let mut commands = Vec::new();
        for (_, event, _) in EVENTS {
            commands.push(command(event)?);
        }
        let text = before.clone().unwrap_or_else(|| "{}\n".to_string());
        let (after, events) = install(&text, &commands).map_err(|e| malformed(&path, e))?;
        if before.is_none() {
            (
                Some(after),
                EVENTS.iter().map(|(name, _, _)| *name).collect(),
            )
        } else {
            (Some(after), events)
        }
    } else {
        match &before {
            None => (None, Vec::new()),
            Some(text) => {
                let (after, events) = remove(text).map_err(|e| malformed(&path, e))?;
                (Some(after), events)
            }
        }
    };
    let changed = after.is_some() && after != before;
    if changed {
        write(&path, after.as_deref().unwrap_or_default())?;
    }
    let shown = path.display().to_string();
    if inv.json() {
        let doc = Obj::document()
            .str("agent", agent)
            .str("settings", &shown)
            .strings("events", &events)
            .bool("changed", changed)
            .finish();
        let _ = writeln!(out, "{doc}");
        return Ok(ExitCode::Ok);
    }
    let _ = match (sub, changed) {
        ("install", true) => writeln!(
            out,
            "installed {agent} hooks ({}) in {shown}",
            events.join(", ")
        ),
        ("install", false) => writeln!(out, "{agent} hooks already installed in {shown}"),
        (_, true) => writeln!(
            out,
            "removed {agent} hooks ({}) from {shown}",
            events.join(", ")
        ),
        (_, false) => writeln!(out, "no {agent} hooks to remove in {shown}"),
    };
    Ok(ExitCode::Ok)
}

// ---------------------------------------------------------------------------
// status
// ---------------------------------------------------------------------------

/// What one agent's settings hold of ank's: the events carrying an entry, and
/// the binary the first of them calls.
struct Installed {
    agent: &'static str,
    settings: PathBuf,
    events: Vec<&'static str>,
    binary: Option<String>,
    current: bool,
}

/// One guard, as the corpus declares it and as it runs.
struct GuardState {
    key: &'static str,
    declared: &'static str,
    in_force: &'static str,
    variable: &'static str,
    set: Option<String>,
    ignored: Option<String>,
}

/// `hooks status` (ADR-89c1e7146fe5): which agents have ank's entries, calling
/// which binary, and which guard levels are in force here. It reads the
/// settings file and the configuration and **writes nothing**: no settings
/// file is created where there is none, and no corpus is touched.
///
/// **A corpus is optional.** Outside one the agents are still listed, and the
/// answer says no guard applies, since a guard is a corpus's declaration; an
/// address the caller wrote, `--repo` or `--worktree`, naming nothing is
/// refused as every verb refuses it.
fn status(inv: &Invocation, cwd: &Path, out: &mut dyn Write) -> Result<ExitCode> {
    if let Some(extra) = inv.positionals.first() {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "extra argument '{extra}': hooks status answers for every agent install knows, \
                 and takes none"
            ),
        )
        .with_hint("ank hooks status"));
    }
    let mut agents = Vec::new();
    for agent in AGENTS {
        agents.push(inspect(agent)?);
    }
    let corpus = corpus(inv, cwd)?;
    let guards = match &corpus {
        Some(repo) => guards(&crate::config::load(&repo.config_path())?),
        None => Vec::new(),
    };
    let corpus = corpus.map(|r| r.corpus.display().to_string());

    if inv.json() {
        let agents = agents.iter().map(|a| {
            Obj::new()
                .str("agent", a.agent)
                .str("settings", &a.settings.display().to_string())
                .bool("installed", a.events.len() == EVENTS.len())
                .strings("events", &a.events)
                .opt_str("binary", a.binary.as_deref())
                .bool("current", a.current)
                .finish()
        });
        let guards = guards.iter().map(|g| {
            Obj::new()
                .str("key", g.key)
                .str("declared", g.declared)
                .str("in_force", g.in_force)
                .str("variable", g.variable)
                .opt_str("set", g.set.as_deref())
                .opt_str("ignored", g.ignored.as_deref())
                .finish()
        });
        let doc = Obj::document()
            .array("agents", agents)
            .opt_str("corpus", corpus.as_deref())
            .array("guards", guards)
            .finish();
        let _ = writeln!(out, "{doc}");
        return Ok(ExitCode::Ok);
    }

    for a in &agents {
        let shown = a.settings.display();
        let state = if a.events.is_empty() {
            "not installed".to_string()
        } else if a.events.len() == EVENTS.len() {
            format!("installed ({})", a.events.join(", "))
        } else {
            format!("partly installed ({})", a.events.join(", "))
        };
        let _ = writeln!(out, "{} {state} in {shown}", a.agent);
        if let Some(binary) = &a.binary {
            let which = if a.current {
                "this binary"
            } else {
                "not this binary"
            };
            let _ = writeln!(out, "  calls {binary}, {which}");
        }
    }
    let _ = writeln!(
        out,
        "generic is wired by hand: point a harness's own hook at `ank hooks run generic <event>`"
    );
    match &corpus {
        None => {
            let _ = writeln!(
                out,
                "no corpus is in force here, so no guard applies: guard.corpus and guard.edit \
                 are a corpus's declaration"
            );
        }
        Some(path) => {
            let _ = writeln!(out, "corpus {path}");
            for g in &guards {
                let set = match (&g.set, &g.ignored) {
                    (Some(set), None) => format!(", {}={set}", g.variable),
                    _ => String::new(),
                };
                let _ = writeln!(
                    out,
                    "{} {}, declared {}{set}",
                    g.key, g.in_force, g.declared
                );
                if let Some(why) = &g.ignored {
                    let _ = writeln!(out, "  {why}");
                }
            }
        }
    }
    Ok(ExitCode::Ok)
}

/// The corpus `status` reads its guards from, or `None` where no walk finds
/// one; an address the caller wrote that names nothing is refused.
fn corpus(inv: &Invocation, cwd: &Path) -> Result<Option<crate::repo::Repo>> {
    let addressed = inv.repo().is_some() || inv.worktree().is_some();
    let mut notes = Vec::new();
    match crate::repo::resolve(inv.repo(), inv.worktree(), cwd, &mut notes) {
        Ok(repo) => {
            if !inv.quiet() {
                let style = inv.style().on_stderr();
                for note in notes {
                    crate::cli::warn(style, &note.to_string(), &[], None);
                }
            }
            Ok(Some(repo))
        }
        Err(e) if addressed => Err(e),
        Err(_) => Ok(None),
    }
}

/// What `agent`'s settings hold of ank's entries, read and never written.
fn inspect(agent: &'static str) -> Result<Installed> {
    let settings = settings_path()?;
    let mut found = Installed {
        agent,
        settings: settings.clone(),
        events: Vec::new(),
        binary: None,
        current: false,
    };
    let Some(text) = read(&settings)? else {
        return Ok(found);
    };
    let root = parse(&text).map_err(|e| malformed(&settings, e))?;
    let Some(hooks) = root.member("hooks") else {
        return Ok(found);
    };
    if !matches!(hooks.kind, Kind::Object(_)) {
        return Err(malformed(&settings, "its hooks is not an object".into()));
    }
    for (name, event, _) in EVENTS {
        let Some(array) = hooks.member(name) else {
            continue;
        };
        let Kind::Array(items) = &array.kind else {
            return Err(malformed(
                &settings,
                format!("its hooks.{name} is not an array"),
            ));
        };
        let Some(command) = items.iter().find_map(|i| ours(i, event)) else {
            continue;
        };
        found.events.push(*name);
        if found.binary.is_none() {
            found.binary = Some(binary_of(&command, event));
        }
    }
    if let Some(binary) = &found.binary {
        let this = std::env::current_exe().and_then(std::fs::canonicalize).ok();
        let that = std::fs::canonicalize(binary).ok();
        found.current = this.is_some() && this == that;
    }
    Ok(found)
}

/// The binary an entry's command runs: what precedes `hooks run claude
/// <event>`, its quotes taken off.
fn binary_of(command: &str, event: &str) -> String {
    let path = command
        .strip_suffix(&format!(" hooks run claude {event}"))
        .unwrap_or(command)
        .trim();
    path.strip_prefix('"')
        .and_then(|p| p.strip_suffix('"'))
        .unwrap_or(path)
        .to_string()
}

/// The two guards of `cfg`, each with what the launch environment made of it.
fn guards(cfg: &crate::config::Config) -> Vec<GuardState> {
    let set = |var: &str| {
        std::env::var(var)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    let var = ank_contract::env::ANK_GUARD_CORPUS;
    let (level, ignored) = tightened(cfg.guard.corpus, var, "guard.corpus", GuardCorpus::parse);
    let corpus = GuardState {
        key: "guard.corpus",
        declared: named(cfg.guard.corpus, GuardCorpus::parse),
        in_force: named(level, GuardCorpus::parse),
        variable: var,
        set: set(var),
        ignored,
    };
    let var = ank_contract::env::ANK_GUARD_EDIT;
    let (level, ignored) = tightened(cfg.guard.edit, var, "guard.edit", GuardEdit::parse);
    let edit = GuardState {
        key: "guard.edit",
        declared: named(cfg.guard.edit, GuardEdit::parse),
        in_force: named(level, GuardEdit::parse),
        variable: var,
        set: set(var),
        ignored,
    };
    vec![corpus, edit]
}

/// The name of a guard level, by asking the parse which name reads as it: the
/// table of names stays the config's.
fn named<L: Copy + PartialEq>(level: L, parse: fn(&str) -> Option<L>) -> &'static str {
    ["deny", "off", "warn", "claim"]
        .into_iter()
        .find(|n| parse(n) == Some(level))
        .unwrap_or_default()
}

/// The agents `run` answers for, each a translation onto the protocol
/// (ADR-b108b24202ed): `generic` is the identity translation, and an agent
/// `install` knows is one of them.
const RUNNERS: &[&str] = &["claude", "generic"];

/// The events of the protocol, under the names `run` is called with.
#[derive(Clone, Copy, PartialEq)]
enum Event {
    SessionStart,
    PreToolUse,
}

/// One call of the hook protocol (ADR-b108b24202ed): what every agent's payload
/// is translated into, and what `generic` reads as it is written.
///
/// `tool` is `read`, `search`, `edit` or `shell`; any tool but `shell` is
/// confronted with `guard.corpus` for every path it reaches, and `edit` with
/// `guard.edit` as well. `paths` are relative to `cwd`, or absolute; `command`
/// is the line a `shell` call runs.
struct Call {
    event: Event,
    tool: String,
    paths: Vec<String>,
    command: Option<String>,
    cwd: Option<PathBuf>,
}

/// What the protocol answers: the call goes on, with a message for the agent
/// or none; it is refused, with the reason; or, at session start, the text the
/// agent is served.
enum Decision {
    Allow(Option<String>),
    Deny(String),
    Context(String),
}

/// What `hooks run` prints for an event, or `None` for silence.
///
/// **Every failure is silence.** An event this version does not serve, a
/// payload that is not a JSON object, a directory no corpus answers for, a
/// status that cannot be read: each is `None`, and `None` is a 0 with nothing
/// on stdout, which Claude Code reads as "go on", and any harness wired to
/// `generic` is told to read the same way.
///
/// **One place decides.** The agent's payload is translated into a [`Call`],
/// [`decide`] answers it, and the [`Decision`] is translated back into the
/// agent's own format: no agent's adapter makes a guard decision of its own.
fn serve(inv: &Invocation, cwd: &Path) -> Option<String> {
    let [agent, event] = inv.positionals.as_slice() else {
        return None;
    };
    let event = match event.as_str() {
        "session-start" => Event::SessionStart,
        "pre-tool-use" => Event::PreToolUse,
        _ => return None,
    };
    if !RUNNERS.contains(&agent.as_str()) {
        return None;
    }
    let mut payload = Vec::new();
    std::io::stdin().read_to_end(&mut payload).ok()?;
    let payload = parse(&String::from_utf8(payload).ok()?).ok()?;
    if agent == "claude" {
        let call = claude_call(event, &payload, cwd)?;
        claude_answer(event, decide(&call, cwd)?)
    } else {
        let call = generic_call(event, &payload)?;
        Some(generic_answer(decide(&call, cwd)?))
    }
}

/// The protocol's answer to `call`, or `None` where it has nothing to say: no
/// corpus answers for the directory, or an error of its own, either of which
/// lets the call through.
fn decide(call: &Call, cwd: &Path) -> Option<Decision> {
    let base = call
        .cwd
        .clone()
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| cwd.to_path_buf());
    match call.event {
        Event::SessionStart => session_start(&base).map(Decision::Context),
        // A shell call is the permissions guard's alone: the two file guards
        // have no path to confront it with, and say nothing about it.
        Event::PreToolUse if call.tool == "shell" => shell_call(call.command.as_deref()?, &base),
        Event::PreToolUse => file_call(call, &base),
    }
}

/// A `generic` document read as the protocol writes it: a JSON object whose
/// `event`, when present, is the event `run` was called with, whose `tool`
/// and, for a shell call, `command` are strings, whose `paths` is an array of
/// strings, and whose `cwd` is a string. Anything else is `None`.
fn generic_call(event: Event, doc: &Node) -> Option<Call> {
    let text = |key: &str| -> Option<Option<String>> {
        match doc.member(key) {
            None => Some(None),
            Some(n) => n.text().map(|t| Some(t.to_string())),
        }
    };
    let named = match event {
        Event::SessionStart => "session-start",
        Event::PreToolUse => "pre-tool-use",
    };
    if text("event")?.is_some_and(|e| e != named) {
        return None;
    }
    let cwd = text("cwd")?.map(PathBuf::from);
    if event == Event::SessionStart {
        return Some(Call {
            event,
            tool: String::new(),
            paths: Vec::new(),
            command: None,
            cwd,
        });
    }
    let tool = text("tool")??;
    let paths = match doc.member("paths") {
        None => Vec::new(),
        Some(n) if matches!(n.kind, Kind::Array(_)) => n
            .items()
            .into_iter()
            .map(|p| p.text().map(str::to_string))
            .collect::<Option<Vec<_>>>()?,
        Some(_) => return None,
    };
    let command = text("command")?;
    if tool == "shell" && command.is_none() {
        return None;
    }
    Some(Call {
        event,
        tool,
        paths,
        command,
        cwd,
    })
}

/// A decision as the protocol writes it, one JSON object on one line.
fn generic_answer(decision: Decision) -> String {
    let doc = Obj::new();
    match decision {
        Decision::Allow(None) => doc.str("decision", "allow"),
        Decision::Allow(Some(message)) => doc.str("decision", "allow").str("message", &message),
        Decision::Deny(reason) => doc.str("decision", "deny").str("reason", &reason),
        Decision::Context(text) => doc.str("decision", "context").str("text", &text),
    }
    .finish()
}

/// Claude Code's payload translated into the protocol, or `None` where the
/// handler before the protocol said nothing: a tool it does not match, a
/// shell call with no command, a payload missing its tool.
///
/// **What Claude Code reads is unchanged** (ADR-b108b24202ed): the session's
/// directory is the process's at session start, as it always was, and the
/// paths a tool reaches are resolved here exactly as they were before, so
/// [`decide`] confronts the same places with the corpus.
fn claude_call(event: Event, payload: &Node, cwd: &Path) -> Option<Call> {
    if event == Event::SessionStart {
        return Some(Call {
            event,
            tool: String::new(),
            paths: Vec::new(),
            command: None,
            cwd: None,
        });
    }
    let tool = payload.member("tool_name")?.text()?;
    let input = payload.member("tool_input")?;
    let typed = payload
        .member("cwd")
        .and_then(Node::text)
        .map(PathBuf::from);
    let base = typed
        .clone()
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| cwd.to_path_buf());
    let (kind, command) = match tool {
        "Bash" => ("shell", Some(input.member("command")?.text()?.to_string())),
        "Read" => ("read", None),
        "Glob" | "Grep" => ("search", None),
        t if EDITORS.contains(&t) => ("edit", None),
        _ => return None,
    };
    let paths = targets(tool, input, &base)
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    Some(Call {
        event,
        tool: kind.to_string(),
        paths,
        command,
        cwd: typed,
    })
}

/// A decision in the bytes Claude Code reads: nothing for a call let through
/// in silence, `systemMessage` and `additionalContext` for one let through
/// with a message, `permissionDecision` for a refusal, and the session's
/// `additionalContext` at its start.
fn claude_answer(event: Event, decision: Decision) -> Option<String> {
    let name = match event {
        Event::SessionStart => "SessionStart",
        Event::PreToolUse => "PreToolUse",
    };
    match decision {
        Decision::Allow(None) => None,
        Decision::Allow(Some(message)) => {
            let output = Obj::new()
                .str("hookEventName", name)
                .str("additionalContext", &message);
            Some(
                Obj::new()
                    .str("systemMessage", &message)
                    .obj("hookSpecificOutput", output)
                    .finish(),
            )
        }
        Decision::Deny(reason) => {
            let output = Obj::new()
                .str("hookEventName", name)
                .str("permissionDecision", "deny")
                .str("permissionDecisionReason", &reason);
            Some(Obj::new().obj("hookSpecificOutput", output).finish())
        }
        Decision::Context(text) => {
            let output = Obj::new()
                .str("hookEventName", name)
                .str("additionalContext", &text);
            Some(Obj::new().obj("hookSpecificOutput", output).finish())
        }
    }
}

/// What an agent is told as its session opens: `ank status` for its identity,
/// exactly as the verb prints it, then where the contract is.
///
/// **The verb itself, not a summary of it.** The corpus, the config and the
/// identity are resolved as `startup` resolves them for every verb, and
/// `status::run` writes into a buffer: so the text is `status`'s, and so is its
/// cost in git processes, with nothing added on top. Notes the resolution
/// would print are dropped, because standard error is not the agent's.
fn session_start(cwd: &Path) -> Option<String> {
    let repo = crate::repo::resolve(None, None, cwd, &mut Vec::new()).ok()?;
    let cfg = crate::config::load(&repo.config_path()).ok()?;
    let (identity, source) = crate::identity::resolved();
    let status = crate::cli::parse(&["status".to_string()]).ok()?;
    let mut text = Vec::new();
    crate::status::run(&status, &repo, &cfg, &identity, source, &mut text).ok()?;
    let mut text = String::from_utf8(text).ok()?;
    text.push_str(CONTRACT);
    Some(text)
}

/// The pointer served after the status: where the rules of the perimeter come
/// from, and where the contract for using them is.
const CONTRACT: &str = "\n\
This repository keeps its tasks and decisions in ank: reach them through the CLI and never by opening .ank/. \
Run `ank context <path>` before touching a perimeter, for the decisions that bind it and what is claimable. \
How to work with them is the contract `ank skills ank` prints: read it before the first claim.\n";

/// The tools that change a file, which `guard.edit` looks at; every file tool
/// the entry is matched to is one `guard.corpus` looks at.
const EDITORS: &[&str] = &["Edit", "MultiEdit", "Write", "NotebookEdit"];

/// The protocol's answer before a tool that reaches files (ADR-43cbb20dcbf1),
/// or `None` where no corpus answers for the directory or the guard cannot
/// read what it needs, either of which lets the call through.
///
/// **The path is the one the tool will reach, not the one it was typed as.**
/// Relative to the session's directory, absolute, or through a symbolic link,
/// it is resolved to the real place before it is confronted with the corpus
/// directory, so `link/config.yml` is under `.ank/` when `link` points there.
/// A file that does not exist yet is resolved through its nearest ancestor
/// that does.
///
/// **What it costs does not depend on the corpus.** The corpus guard reads the
/// configuration and nothing else; the edit guard adds the one reading of the
/// claims namespace `claim::on_task` makes and the one entity file it names
/// (ADR-cc65f1388a71).
fn file_call(call: &Call, base: &Path) -> Option<Decision> {
    let repo = crate::repo::resolve(None, None, base, &mut Vec::new()).ok()?;
    let cfg = crate::config::load(&repo.config_path()).ok()?;
    let targets: Vec<PathBuf> = call.paths.iter().map(|p| real(&base.join(p))).collect();
    if targets.is_empty() {
        return Some(Decision::Allow(None));
    }
    let corpus = real(&repo.ank);
    let worktree = real(&repo.worktree);

    let (level, note) = tightened(
        cfg.guard.corpus,
        ank_contract::env::ANK_GUARD_CORPUS,
        "guard.corpus",
        GuardCorpus::parse,
    );
    let under: Vec<&PathBuf> = targets.iter().filter(|t| t.starts_with(&corpus)).collect();
    if let Some(target) = under.first() {
        if level == GuardCorpus::Off {
            return Some(Decision::Allow(None));
        }
        let shown = shown(target, &worktree);
        let reason = format!(
            "{shown} is under the corpus directory, which is reached through the ank CLI and \
             never by opening its files: `ank show <id>` prints an entity whole, `ank find \
             <query>` lists, `ank context <path>` serves what binds a perimeter.{}",
            note.map(|n| format!(" {n}")).unwrap_or_default()
        );
        return Some(Decision::Deny(reason));
    }

    if call.tool != "edit" {
        return Some(Decision::Allow(None));
    }
    let (level, note) = tightened(
        cfg.guard.edit,
        ank_contract::env::ANK_GUARD_EDIT,
        "guard.edit",
        GuardEdit::parse,
    );
    if level == GuardEdit::Off {
        return Some(Decision::Allow(None));
    }
    let (identity, _) = crate::identity::resolved();
    // Outside the work tree a path is no perimeter, and nothing can cover it.
    let paths: Vec<String> = targets
        .iter()
        .filter_map(|t| t.strip_prefix(&worktree).ok())
        .filter_map(|rel| ank_core::normalize_path(&rel.to_string_lossy()))
        .filter(|rel| !rel.is_empty())
        .collect();
    let Some(path) = paths.first() else {
        return Some(Decision::Allow(None));
    };
    if covered(&repo, &identity, path)? {
        return Some(Decision::Allow(None));
    }
    let note = note.map(|n| format!(" {n}")).unwrap_or_default();
    if level == GuardEdit::Claim {
        let reason = format!(
            "guard.edit is claim, and no live claim of {identity} covers {path}: `ank context \
             {path}` names the task that does, and `ank claim <id>` takes it.{note}"
        );
        return Some(Decision::Deny(reason));
    }
    Some(Decision::Allow(Some(format!(
        "no live claim of {identity} covers {path}: `ank context {path}` names what binds it \
         and what is claimable, and `ank claim <id>` takes the task.{note}"
    ))))
}

/// What a shell call meets under the permissions guard (ADR-ddbe024ca7f2):
/// refused under `deny` when one of the ank verbs it runs performs an act the
/// identity's role forbids, naming the entry and the command a human runs
/// instead; let through otherwise, with the note when a value the launcher
/// set was ignored.
///
/// **A command that runs no ank costs nothing.** It is split and read before
/// any corpus is resolved, so `cargo test` or `git status` starts no git and
/// reads no file. The verb the call runs is read from the invocation
/// `cli::parse` makes of it, and the acts from [`crate::cli::acts_of`], the
/// table the verb itself warns from.
///
/// **The identity is the session's**, the `ANK_AGENT` the hook runs under,
/// and never a prefix the command sets: `ANK_AGENT=human:me ank accept` is
/// judged as the agent that typed it.
fn shell_call(command: &str, base: &Path) -> Option<Decision> {
    let calls = ank_calls(command);
    if calls.is_empty() {
        return Some(Decision::Allow(None));
    }
    let repo = crate::repo::resolve(None, None, base, &mut Vec::new()).ok()?;
    let cfg = crate::config::load(&repo.config_path()).ok()?;
    let (level, note) = tightened(
        cfg.enforce,
        ank_contract::env::ANK_PERMISSIONS_ENFORCE,
        "permissions.enforce",
        Enforce::parse,
    );
    let suffix = note.as_ref().map(|n| format!(" {n}")).unwrap_or_default();
    if level == Enforce::Deny {
        let (identity, _) = crate::identity::resolved();
        for (typed, argv) in &calls {
            let Ok(inv) = crate::cli::parse(argv) else {
                continue;
            };
            for act in crate::cli::acts_of(&inv) {
                if let Some(f) = cfg.forbids(&identity, &act) {
                    return Some(Decision::Deny(format!(
                        "permissions.enforce is deny, and role '{}' of {identity} forbids \
                         {act} ({}): `{typed}` is a human's to run, from their own terminal \
                         under an identity whose role allows it.{suffix}",
                        f.role, f.entry
                    )));
                }
            }
        }
        return Some(Decision::Allow(None));
    }
    // Nothing refused, and a value the launcher set was ignored: said, as the
    // file guards say it.
    Some(Decision::Allow(note))
}

/// The ank invocations a shell command runs, each as it was typed and as the
/// argument list after the program: the command split on `&&`, `||`, `;`,
/// `|`, `&` and newlines outside quotes, each part split into words the way
/// `sh` splits them for quotes and backslashes, leading `NAME=value`
/// assignments and the wrappers `env`, `command`, `exec`, `nohup` and `time`
/// passed over, and the program kept when its last path component is `ank`.
///
/// **Not a shell.** What runs ank through another program, `sh -c`, `xargs`,
/// `$(...)`, an alias or a script, is not read here and is let through; the
/// verb still warns when it runs.
fn ank_calls(command: &str) -> Vec<(String, Vec<String>)> {
    if !command.contains("ank") {
        return Vec::new();
    }
    let mut calls = Vec::new();
    for part in shell_parts(command) {
        let words = shell_words(&part);
        let mut rest = words.iter().map(String::as_str).peekable();
        while let Some(w) = rest.peek() {
            let assignment = w.split_once('=').is_some_and(|(k, _)| {
                !k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            });
            if assignment || matches!(*w, "env" | "command" | "exec" | "nohup" | "time") {
                rest.next();
            } else {
                break;
            }
        }
        let Some(program) = rest.next() else { continue };
        let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
        if name == "ank" || name == "ank.exe" {
            calls.push((part.trim().to_string(), rest.map(str::to_string).collect()));
        }
    }
    calls
}

/// `command` cut at the separators of [`ank_calls`] that stand outside quotes.
fn shell_parts(command: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut chars = command.chars().peekable();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), c) if c == q => {
                quote = None;
                current.push(c);
            }
            (Some('"'), '\\') => {
                current.push(c);
                if let Some(n) = chars.next() {
                    current.push(n);
                }
            }
            (Some(_), c) => current.push(c),
            (None, '\'' | '"') => {
                quote = Some(c);
                current.push(c);
            }
            (None, '\\') => {
                current.push(c);
                if let Some(n) = chars.next() {
                    current.push(n);
                }
            }
            (None, ';' | '|' | '&' | '\n') => {
                if matches!(chars.peek(), Some('|' | '&')) && c != ';' && c != '\n' {
                    chars.next();
                }
                parts.push(std::mem::take(&mut current));
            }
            (None, c) => current.push(c),
        }
    }
    parts.push(current);
    parts.retain(|p| !p.trim().is_empty());
    parts
}

/// One part of a command split into words: whitespace outside quotes
/// separates, quotes are taken off, a backslash keeps the character after it.
fn shell_words(part: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quote: Option<char> = None;
    let mut chars = part.chars();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some('"'), '\\') => {
                if let Some(n) = chars.next() {
                    word.push(n);
                }
            }
            (Some(_), c) => word.push(c),
            (None, '\'' | '"') => {
                quote = Some(c);
                started = true;
            }
            (None, '\\') => {
                if let Some(n) = chars.next() {
                    word.push(n);
                }
                started = true;
            }
            (None, c) if c.is_whitespace() => {
                if started || !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
                started = false;
            }
            (None, c) => {
                word.push(c);
                started = true;
            }
        }
    }
    if started || !word.is_empty() {
        words.push(word);
    }
    words
}

/// Whether a live claim of `identity` covers `path`, matched the way `ank
/// scope` matches it; `None` when the claims or the task cannot be read, which
/// lets the call through.
fn covered(repo: &crate::repo::Repo, identity: &str, path: &str) -> Option<bool> {
    let Some(standing) = crate::claim::on_task(&repo.corpus, identity).ok()? else {
        return Some(false);
    };
    if standing.lapsed {
        return Some(false);
    }
    let loaded = crate::store::Store::new(&repo.ank)
        .load(&standing.id)
        .ok()?;
    let ank_core::Entity::Task(task) = loaded.entity else {
        return Some(false);
    };
    Some(crate::context::in_perimeter(&task.scope, Some(path)))
}

/// The places a tool call reaches, resolved: the file a file tool names, the
/// directory a search starts from, and the literal head of a `Glob` pattern
/// joined to it.
fn targets(tool: &str, input: &Node, base: &Path) -> Vec<PathBuf> {
    let at = |raw: &str| -> PathBuf { real(&base.join(raw)) };
    let mut out = Vec::new();
    match tool {
        "Read" | "Edit" | "MultiEdit" | "Write" => {
            out.extend(input.member("file_path").and_then(Node::text).map(at));
        }
        "NotebookEdit" => {
            out.extend(input.member("notebook_path").and_then(Node::text).map(at));
        }
        "Glob" | "Grep" => {
            let from = input.member("path").and_then(Node::text);
            let start = from.map_or_else(|| real(base), at);
            if tool == "Glob" {
                let head: Vec<&str> = input
                    .member("pattern")
                    .and_then(Node::text)
                    .unwrap_or_default()
                    .split(['/', '\\'])
                    .take_while(|c| !c.contains(['*', '?', '[', '{']))
                    .collect();
                if !head.is_empty() {
                    out.push(real(&start.join(head.join("/"))));
                }
            }
            out.push(start);
        }
        _ => {}
    }
    out
}

/// `path` with `.` and `..` folded, then resolved through every symbolic link
/// of its longest existing ancestor; the part that does not exist yet is joined
/// back as written.
fn real(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut folded = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                folded.pop();
            }
            c => folded.push(c),
        }
    }
    let mut existing = folded.as_path();
    let mut rest = Vec::new();
    loop {
        if let Ok(resolved) = existing.canonicalize() {
            return rest.iter().rev().fold(resolved, |p, c| p.join(c));
        }
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name.to_os_string());
                existing = parent;
            }
            _ => return folded,
        }
    }
}

/// A target as the agent would recognise it: relative to the work tree when it
/// is inside it.
fn shown(target: &Path, worktree: &Path) -> String {
    match target.strip_prefix(worktree) {
        Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
        Err(_) => target.display().to_string(),
    }
}

/// The level a guard runs at: the corpus's, or the launch environment's when
/// that one is tighter. A looser or unreadable value is ignored, and the note
/// says so, to be carried by whatever the guard answers.
fn tightened<L: Copy + Ord>(
    declared: L,
    var: &str,
    key: &str,
    parse: fn(&str) -> Option<L>,
) -> (L, Option<String>) {
    let Some(set) = std::env::var(var).ok().filter(|v| !v.trim().is_empty()) else {
        return (declared, None);
    };
    let set = set.trim();
    match parse(set) {
        Some(level) if level >= declared => (level, None),
        Some(_) => {
            let named = named(declared, parse);
            (
                declared,
                Some(format!(
                    "{var}={set} was ignored: .ank/config.yml declares {key} {named}, and the \
                     launch environment may tighten it, never loosen it."
                )),
            )
        }
        None => (
            declared,
            Some(format!(
                "{var}={set} was ignored: it is not a level of {key}."
            )),
        ),
    }
}

fn agent<'a>(inv: &'a Invocation, sub: &str) -> Result<&'a str> {
    let hint = format!("ank hooks {sub} {}", AGENTS.join("|"));
    let Some(name) = inv.positionals.first() else {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "hooks {sub} names its agent: {} is the one it installs for, and generic is \
                 wired by hand",
                AGENTS.join(", ")
            ),
        )
        .with_hint(hint));
    };
    // The protocol itself has no settings file to write: its harness wires it
    // in its own hook configuration (ADR-b108b24202ed).
    if name == "generic" {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "generic is wired by hand: it is the hook protocol itself, with no settings \
                 file for hooks {sub} to write; point the harness's own hook at `ank hooks run \
                 generic <event>`, as docs/integrating.md shows"
            ),
        )
        .with_hint("ank hooks run generic pre-tool-use"));
    }
    if !AGENTS.contains(&name.as_str()) {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "no agent named '{name}': {} is the one this version installs for, and generic \
                 is wired by hand",
                AGENTS.join(", ")
            ),
        )
        .with_hint(hint));
    }
    Ok(name)
}

/// Where Claude Code reads its user settings: `CLAUDE_CONFIG_DIR`, or `.claude`
/// under the home directory. Empty counts as unset, as it does for every other
/// directory the binary takes from the environment.
fn settings_path() -> Result<PathBuf> {
    let dir = std::env::var_os(ank_contract::env::CLAUDE_CONFIG_DIR)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::home_dir().map(|h| h.join(".claude")))
        .ok_or_else(|| {
            CliError::new(
                ExitCode::Environment,
                "no directory for Claude Code's settings: CLAUDE_CONFIG_DIR is unset and there is no home directory",
            )
            .with_hint("CLAUDE_CONFIG_DIR=<directory> ank hooks install claude")
        })?;
    Ok(dir.join("settings.json"))
}

/// The installing binary, by absolute path, quoted for the shell the harness
/// runs the command with.
fn exe() -> Result<String> {
    let exe = std::env::current_exe()
        .and_then(std::fs::canonicalize)
        .map_err(|e| {
            CliError::new(
                ExitCode::Environment,
                format!("the path of this binary cannot be read ({e})"),
            )
        })?;
    let exe = exe.display().to_string();
    let exe = if cfg!(windows) { for_shell(&exe) } else { exe };
    Ok(format!("\"{exe}\""))
}

/// A Windows path as a POSIX shell runs it. Claude Code runs a hook command
/// with Git Bash on Windows, and `fs::canonicalize` answers there in the
/// verbatim form, `\\?\C:\...`: inside double quotes bash collapses the
/// `\\` and reads no backslash as a separator, so the binary is never found
/// and every hook exits 127. The prefix comes off, `\\?\UNC\` back to the
/// `\\` it stands for, and the separators become forward slashes, which Git
/// Bash, cmd and Windows itself all accept. Pure text, so its test runs on
/// every platform; only `exe` decides it applies on Windows alone, where a
/// backslash cannot be part of a file name.
fn for_shell(path: &str) -> String {
    let path = match path.strip_prefix(r"\\?\UNC\") {
        Some(rest) => format!(r"\\{rest}"),
        None => path.strip_prefix(r"\\?\").unwrap_or(path).to_string(),
    };
    path.replace('\\', "/")
}

fn read(path: &Path) -> Result<Option<String>> {
    match std::fs::read(path) {
        Ok(bytes) => String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| malformed(path, "it is not UTF-8".into())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(CliError::new(
            ExitCode::Environment,
            format!("{} cannot be read ({e})", path.display()),
        )),
    }
}

/// Written in place rather than renamed over, so a settings file that is a
/// link into somebody's dotfiles stays one, and nothing else is ever created
/// beside it.
fn write(path: &Path, text: &str) -> Result<()> {
    let fail = |e: std::io::Error| {
        CliError::new(
            ExitCode::Environment,
            format!("{} cannot be written ({e})", path.display()),
        )
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(fail)?;
    }
    std::fs::write(path, text).map_err(fail)
}

fn malformed(path: &Path, why: String) -> CliError {
    CliError::new(
        ExitCode::Prerequisite,
        format!("{} is left alone: {why}", path.display()),
    )
    .with_hint(format!(
        "repair {} by hand, then run the command again",
        path.display()
    ))
}

// ---------------------------------------------------------------------------
// The edits
// ---------------------------------------------------------------------------

type Edit<T> = std::result::Result<T, String>;

/// `text` with an entry for every event that lacks this binary's, and the
/// events that gained one. An entry of ank's naming another path is taken out
/// first, so a binary that moved replaces what the old one wrote.
fn install(text: &str, commands: &[String]) -> Edit<(String, Vec<&'static str>)> {
    let mut text = text.to_string();
    let root = parse(&text)?;
    let Some(hooks) = root.member("hooks") else {
        let created = created_hooks(commands);
        text = insert_member(&text, &root, "hooks", &created);
        return Ok((text, EVENTS.iter().map(|(n, _, _)| *n).collect()));
    };
    if !matches!(hooks.kind, Kind::Object(_)) {
        return Err("its hooks is not an object".into());
    }
    let mut added = Vec::new();
    for ((name, event, matcher), command) in EVENTS.iter().zip(commands) {
        loop {
            let root = parse(&text)?;
            let hooks = root.member("hooks").expect("present above");
            let Some(array) = hooks.member(name) else {
                let created = format!("[{}]", compact(command, *matcher));
                text = insert_member(&text, hooks, name, &created);
                added.push(*name);
                break;
            };
            let Kind::Array(items) = &array.kind else {
                return Err(format!("its hooks.{name} is not an array"));
            };
            if let Some(entry) = items
                .iter()
                .find(|i| ours(i, event).as_deref() == Some(command))
            {
                // An earlier matcher, moved to the current one in place: the
                // one string of the entry that changed meaning.
                let stale = entry.member("matcher").filter(|m| {
                    matches!(&m.kind, Kind::Str(v) if Some(v.as_str()) != *matcher
                        && PREVIOUS_MATCHERS.contains(&(name, v.as_str())))
                });
                if let (Some(node), Some(current)) = (stale, matcher) {
                    let replaced = crate::json::string(current);
                    text = format!("{}{replaced}{}", &text[..node.start], &text[node.end..]);
                    if !added.contains(name) {
                        added.push(*name);
                    }
                }
                break;
            }
            // In place, so whatever form the entry was written in survives it,
            // and `remove` still recognises a container install created.
            if let Some(stale) = items.iter().find_map(|i| command_of(i, event)) {
                let replaced = crate::json::string(command);
                text = format!("{}{replaced}{}", &text[..stale.start], &text[stale.end..]);
                added.push(*name);
                // Once more, for the matcher the moved entry may also carry.
                continue;
            }
            let indent = child_indent(&text, array.start);
            let entry = pretty(command, *matcher, &indent, &unit(&text), eol(&text));
            text = insert_item(&text, array, &entry);
            added.push(*name);
            break;
        }
    }
    Ok((text, added))
}

/// `text` with every entry of ank's taken out, and the containers `install`
/// created with them; the events that lost one.
fn remove(text: &str) -> Edit<(String, Vec<&'static str>)> {
    let mut text = text.to_string();
    let root = parse(&text)?;
    let Some(hooks) = root.member("hooks") else {
        return Ok((text, Vec::new()));
    };
    if !matches!(hooks.kind, Kind::Object(_)) {
        return Err("its hooks is not an object".into());
    }
    if let Some(commands) = created_by_install(hooks) {
        let found = &text[hooks.start..hooks.end];
        if created_hooks(&commands) == found || created_hooks_earlier(&commands) == found {
            let index = root.index_of("hooks").expect("present above");
            text = remove_item(&text, &root, index);
            return Ok((text, EVENTS.iter().map(|(n, _, _)| *n).collect()));
        }
    }
    let mut removed = Vec::new();
    for (name, event, matcher) in EVENTS {
        loop {
            let root = parse(&text)?;
            let hooks = root.member("hooks").expect("present above");
            let Some(array) = hooks.member(name) else {
                break;
            };
            let Kind::Array(items) = &array.kind else {
                return Err(format!("its hooks.{name} is not an array"));
            };
            let Some(index) = items.iter().position(|i| ours(i, event).is_some()) else {
                break;
            };
            let command = ours(&items[index], event).unwrap_or_default();
            if items.len() == 1
                && matchers_of(name, *matcher)
                    .iter()
                    .any(|m| text[array.start..array.end] == format!("[{}]", compact(&command, *m)))
            {
                let index = hooks.index_of(name).expect("present above");
                text = remove_item(&text, hooks, index);
            } else {
                text = remove_item(&text, array, index);
            }
            if !removed.contains(name) {
                removed.push(*name);
            }
        }
    }
    Ok((text, removed))
}

/// The commands of a `hooks` object holding exactly the events install writes,
/// each with one entry of ank's, in install's order: the shape it creates.
fn created_by_install(hooks: &Node) -> Option<Vec<String>> {
    let Kind::Object(members) = &hooks.kind else {
        return None;
    };
    if members.len() != EVENTS.len() {
        return None;
    }
    let mut commands = Vec::new();
    for (member, (name, event, _)) in members.iter().zip(EVENTS) {
        if member.key != *name {
            return None;
        }
        let Kind::Array(items) = &member.value.kind else {
            return None;
        };
        if items.len() != 1 {
            return None;
        }
        commands.push(ours(&items[0], event)?);
    }
    Some(commands)
}

/// The command of an entry of ank's for `event`: one hook, a command running
/// `hooks run claude <event>`, whatever path names the binary.
fn ours(entry: &Node, event: &str) -> Option<String> {
    match &command_of(entry, event)?.kind {
        Kind::Str(command) => Some(command.clone()),
        _ => None,
    }
}

/// The string node holding that command.
fn command_of<'a>(entry: &'a Node, event: &str) -> Option<&'a Node> {
    let hooks = entry.member("hooks")?;
    let Kind::Array(list) = &hooks.kind else {
        return None;
    };
    let [hook] = list.as_slice() else { return None };
    let node = hook.member("command")?;
    let Kind::Str(command) = &node.kind else {
        return None;
    };
    command
        .ends_with(&format!(" hooks run claude {event}"))
        .then_some(node)
}

/// The `hooks` object install creates when the file has none: one line, the
/// form that tells it apart from one its owner left empty.
fn created_hooks(commands: &[String]) -> String {
    let members: Vec<String> = EVENTS
        .iter()
        .zip(commands)
        .map(|((name, _, matcher), command)| {
            format!(
                "{}: [{}]",
                crate::json::string(name),
                compact(command, *matcher)
            )
        })
        .collect();
    format!("{{{}}}", members.join(", "))
}

/// [`created_hooks`] as an earlier install wrote it, with the matchers of
/// [`PREVIOUS_MATCHERS`].
fn created_hooks_earlier(commands: &[String]) -> String {
    let members: Vec<String> = EVENTS
        .iter()
        .zip(commands)
        .map(|((name, _, matcher), command)| {
            let m = matchers_of(name, *matcher).last().copied().flatten();
            format!("{}: [{}]", crate::json::string(name), compact(command, m))
        })
        .collect();
    format!("{{{}}}", members.join(", "))
}

fn hook(command: &str) -> String {
    format!(
        "{{\"type\": \"command\", \"command\": {}}}",
        crate::json::string(command)
    )
}

fn compact(command: &str, matcher: Option<&str>) -> String {
    let matcher = matcher
        .map(|m| format!("\"matcher\": {}, ", crate::json::string(m)))
        .unwrap_or_default();
    format!("{{{matcher}\"hooks\": [{}]}}", hook(command))
}

fn pretty(command: &str, matcher: Option<&str>, indent: &str, unit: &str, eol: &str) -> String {
    let inner = format!("{indent}{unit}");
    let mut s = format!("{{{eol}");
    if let Some(m) = matcher {
        s.push_str(&format!(
            "{inner}\"matcher\": {},{eol}",
            crate::json::string(m)
        ));
    }
    s.push_str(&format!(
        "{inner}\"hooks\": [{eol}{inner}{unit}{}{eol}{inner}]{eol}{indent}}}",
        hook(command)
    ));
    s
}

/// `value` as the last member of `object`, under `key`.
fn insert_member(text: &str, object: &Node, key: &str, value: &str) -> String {
    let member = format!("{}: {value}", crate::json::string(key));
    insert_item(text, object, &member)
}

/// `item` as the last element of `container`, on a line of its own.
///
/// After the last element, the insertion is a comma and the item, which is
/// exactly what [`remove_item`] deletes from the element before it. Into an
/// empty container, it is the item and the closing line, which is what
/// [`remove_item`] deletes after the bracket.
fn insert_item(text: &str, container: &Node, item: &str) -> String {
    let eol = eol(text);
    let indent = child_indent(text, container.start);
    let (at, piece) = match container.items().last() {
        Some(last) => (last.end, format!(",{eol}{indent}{item}")),
        None => (
            container.start + 1,
            format!(
                "{eol}{indent}{item}{eol}{}",
                line_indent(text, container.start)
            ),
        ),
    };
    format!("{}{piece}{}", &text[..at], &text[at..])
}

/// `container` without its element at `index`, and what [`insert_item`] put
/// around it.
fn remove_item(text: &str, container: &Node, index: usize) -> String {
    let items = container.items();
    let item = &items[index];
    let (from, to) = if items.len() == 1 {
        let closing = format!("{}{}", eol(text), line_indent(text, container.start));
        let to = if text[item.end..].starts_with(&closing) {
            item.end + closing.len()
        } else {
            item.end
        };
        (container.start + 1, to)
    } else if index > 0 {
        (items[index - 1].end, item.end)
    } else {
        (item.start, items[1].start)
    };
    format!("{}{}", &text[..from], &text[to..])
}

fn eol(text: &str) -> &'static str {
    if text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// The spaces and tabs opening the line `at` is on.
fn line_indent(text: &str, at: usize) -> String {
    let start = text[..at].rfind('\n').map_or(0, |i| i + 1);
    text[start..]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect()
}

fn child_indent(text: &str, at: usize) -> String {
    format!("{}{}", line_indent(text, at), unit(text))
}

/// The file's own indentation step: the first line indented deeper than the
/// one before it says by how much. Two spaces where nothing says.
fn unit(text: &str) -> String {
    let mut previous = 0usize;
    for line in text.lines() {
        let width = line.len() - line.trim_start_matches([' ', '\t']).len();
        if line.trim().is_empty() {
            continue;
        }
        if width > previous {
            return line[previous..width].to_string();
        }
        previous = width;
    }
    "  ".to_string()
}

// ---------------------------------------------------------------------------
// The parse: spans, and the strings decoded
// ---------------------------------------------------------------------------

/// A value and the bytes it spans, `start` its first and `end` one past its
/// last.
#[derive(Debug)]
struct Node {
    start: usize,
    end: usize,
    kind: Kind,
}

#[derive(Debug)]
enum Kind {
    Object(Vec<Member>),
    Array(Vec<Node>),
    Str(String),
    Scalar,
}

/// A member spans from its key to the end of its value.
#[derive(Debug)]
struct Member {
    key: String,
    span: Node,
    value: Node,
}

impl Node {
    fn member(&self, key: &str) -> Option<&Node> {
        match &self.kind {
            Kind::Object(members) => members.iter().find(|m| m.key == key).map(|m| &m.value),
            _ => None,
        }
    }

    fn text(&self) -> Option<&str> {
        match &self.kind {
            Kind::Str(s) => Some(s),
            _ => None,
        }
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        match &self.kind {
            Kind::Object(members) => members.iter().position(|m| m.key == key),
            _ => None,
        }
    }

    /// The elements, or the members as the spans they cover.
    fn items(&self) -> Vec<&Node> {
        match &self.kind {
            Kind::Object(members) => members.iter().map(|m| &m.span).collect(),
            Kind::Array(items) => items.iter().collect(),
            _ => Vec::new(),
        }
    }
}

fn parse(text: &str) -> Edit<Node> {
    let mut p = Parser {
        b: text.as_bytes(),
        i: 0,
    };
    p.ws();
    let root = p.value()?;
    p.ws();
    if p.i != p.b.len() {
        return Err(format!("it carries text after its value, at byte {}", p.i));
    }
    if !matches!(root.kind, Kind::Object(_)) {
        return Err("it is not a JSON object".into());
    }
    Ok(root)
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn bad<T>(&self, what: &str) -> Edit<T> {
        if self.i >= self.b.len() {
            Err(format!("it is not valid JSON: {what}, and the file ends"))
        } else {
            Err(format!("it is not valid JSON: {what} at byte {}", self.i))
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        if self.b.get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn value(&mut self) -> Edit<Node> {
        let start = self.i;
        match self.b.get(self.i) {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => {
                let s = self.string()?;
                Ok(Node {
                    start,
                    end: self.i,
                    kind: Kind::Str(s),
                })
            }
            Some(_) => {
                for word in ["true", "false", "null"] {
                    if self.b[self.i..].starts_with(word.as_bytes()) {
                        self.i += word.len();
                        return Ok(Node {
                            start,
                            end: self.i,
                            kind: Kind::Scalar,
                        });
                    }
                }
                self.number()?;
                Ok(Node {
                    start,
                    end: self.i,
                    kind: Kind::Scalar,
                })
            }
            None => self.bad("a value is expected"),
        }
    }

    fn number(&mut self) -> Edit<()> {
        let start = self.i;
        self.eat(b'-');
        let digits = |p: &mut Self| {
            let from = p.i;
            while p.b.get(p.i).is_some_and(u8::is_ascii_digit) {
                p.i += 1;
            }
            p.i > from
        };
        if !digits(self) {
            self.i = start;
            return self.bad("a value is expected");
        }
        if self.eat(b'.') && !digits(self) {
            return self.bad("a digit is expected");
        }
        if self.eat(b'e') || self.eat(b'E') {
            let _ = self.eat(b'+') || self.eat(b'-');
            if !digits(self) {
                return self.bad("a digit is expected");
            }
        }
        Ok(())
    }

    fn string(&mut self) -> Edit<String> {
        self.i += 1;
        let mut out = String::new();
        loop {
            let Some(&c) = self.b.get(self.i) else {
                return self.bad("a string is not closed");
            };
            match c {
                b'"' => {
                    self.i += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.i += 1;
                    let Some(&e) = self.b.get(self.i) else {
                        return self.bad("an escape is not finished");
                    };
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hex = self
                                .b
                                .get(self.i..self.i + 4)
                                .and_then(|h| std::str::from_utf8(h).ok())
                                .and_then(|h| u32::from_str_radix(h, 16).ok());
                            let Some(code) = hex else {
                                return self.bad("a \\u escape is not four hex digits");
                            };
                            self.i += 4;
                            out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                        }
                        _ => return self.bad("an unknown escape"),
                    }
                }
                c if c < 0x20 => return self.bad("a control character inside a string"),
                _ => {
                    // The text is a `&str`, so a multi-byte character is whole:
                    // copy up to the next byte that means something here.
                    let from = self.i;
                    while self
                        .b
                        .get(self.i)
                        .is_some_and(|&c| c != b'"' && c != b'\\' && c >= 0x20)
                    {
                        self.i += 1;
                    }
                    out.push_str(std::str::from_utf8(&self.b[from..self.i]).unwrap_or_default());
                }
            }
        }
    }

    fn array(&mut self) -> Edit<Node> {
        let start = self.i;
        self.i += 1;
        let mut items = Vec::new();
        self.ws();
        if !self.eat(b']') {
            loop {
                self.ws();
                items.push(self.value()?);
                self.ws();
                if self.eat(b']') {
                    break;
                }
                if !self.eat(b',') {
                    return self.bad("a comma or a closing bracket is expected");
                }
            }
        }
        Ok(Node {
            start,
            end: self.i,
            kind: Kind::Array(items),
        })
    }

    fn object(&mut self) -> Edit<Node> {
        let start = self.i;
        self.i += 1;
        let mut members = Vec::new();
        self.ws();
        if !self.eat(b'}') {
            loop {
                self.ws();
                let key_start = self.i;
                if self.b.get(self.i) != Some(&b'"') {
                    return self.bad("a key is expected");
                }
                let key = self.string()?;
                self.ws();
                if !self.eat(b':') {
                    return self.bad("a colon is expected");
                }
                self.ws();
                let value = self.value()?;
                members.push(Member {
                    key,
                    span: Node {
                        start: key_start,
                        end: value.end,
                        kind: Kind::Scalar,
                    },
                    value,
                });
                self.ws();
                if self.eat(b'}') {
                    break;
                }
                if !self.eat(b',') {
                    return self.bad("a comma or a closing brace is expected");
                }
            }
        }
        Ok(Node {
            start,
            end: self.i,
            kind: Kind::Object(members),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commands() -> Vec<String> {
        EVENTS
            .iter()
            .map(|(_, e, _)| format!("\"/bin/ank\" hooks run claude {e}"))
            .collect()
    }

    #[test]
    fn a_verbatim_drive_path_reaches_the_shell_without_its_prefix_and_with_slashes() {
        assert_eq!(
            for_shell(r"\\?\C:\Users\me\AppData\Roaming\npm\bin\ank.exe"),
            "C:/Users/me/AppData/Roaming/npm/bin/ank.exe"
        );
    }

    #[test]
    fn a_verbatim_unc_path_becomes_the_share_it_names() {
        assert_eq!(
            for_shell(r"\\?\UNC\server\share\bin\ank.exe"),
            "//server/share/bin/ank.exe"
        );
    }

    #[test]
    fn a_path_without_the_prefix_only_changes_separators() {
        assert_eq!(for_shell(r"C:\tools\ank.exe"), "C:/tools/ank.exe");
        assert_eq!(for_shell("C:/tools/ank.exe"), "C:/tools/ank.exe");
        assert_eq!(for_shell("/usr/local/bin/ank"), "/usr/local/bin/ank");
    }

    #[test]
    fn a_moved_binary_replaces_the_entry_the_old_one_wrote() {
        let (once, _) = install("{}\n", &commands()).unwrap();
        let moved: Vec<String> = commands()
            .iter()
            .map(|c| c.replace("/bin/ank", "/opt/ank"))
            .collect();
        let (twice, events) = install(&once, &moved).unwrap();
        assert_eq!(events, ["SessionStart", "PreToolUse"]);
        assert!(!twice.contains("/bin/ank"), "{twice}");
        assert_eq!(twice.matches("/opt/ank").count(), 2, "{twice}");
        let (back, _) = remove(&twice).unwrap();
        assert!(!back.contains("hooks run"), "{back}");
    }

    #[test]
    fn an_entry_somebody_merged_a_hook_into_is_not_ank_s() {
        let text = r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"x hooks run claude session-start"},{"type":"command","command":"y"}]}]}}"#;
        let (after, events) = remove(text).unwrap();
        assert_eq!(after, text);
        assert!(events.is_empty());
    }
}
