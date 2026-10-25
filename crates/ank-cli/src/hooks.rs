//! `hooks`: the entries an agent harness calls at its own events, installed
//! into that harness's user-level configuration, taken out again, and answered
//! (ADR-24efa7543a16, ADR-43cbb20dcbf1).
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

use crate::cli::{CliError, Invocation, Result};
use crate::config::{GuardCorpus, GuardEdit};
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
/// `PreToolUse` is matched to the file tools: they are what the two guards look
/// at, and a shell line is out of the decision's scope, so an entry matching
/// every tool would start this binary on every shell call to say nothing.
const EVENTS: &[(&str, &str, Option<&str>)] = &[
    ("SessionStart", "session-start", None),
    (
        "PreToolUse",
        "pre-tool-use",
        Some("Read|Write|Edit|MultiEdit|NotebookEdit|Glob|Grep"),
    ),
];

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

/// What `hooks run` prints for an event, or `None` for silence.
///
/// **Every failure is silence.** An event this version does not serve, a
/// payload that is not a JSON object, a directory no corpus answers for, a
/// status that cannot be read: each is `None`, and `None` is a 0 with nothing
/// on stdout, which Claude Code reads as "go on".
fn serve(inv: &Invocation, cwd: &Path) -> Option<String> {
    let [agent, event] = inv.positionals.as_slice() else {
        return None;
    };
    if agent != "claude" || !matches!(event.as_str(), "session-start" | "pre-tool-use") {
        return None;
    }
    let mut payload = Vec::new();
    std::io::stdin().read_to_end(&mut payload).ok()?;
    let payload = parse(&String::from_utf8(payload).ok()?).ok()?;
    if event == "pre-tool-use" {
        return pre_tool_use(&payload, cwd);
    }
    let context = session_start(cwd)?;
    let output = Obj::new()
        .str("hookEventName", "SessionStart")
        .str("additionalContext", &context);
    Some(Obj::new().obj("hookSpecificOutput", output).finish())
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
How to work with them is the ank skill; `ank skills --install` installs it if it is missing.\n";

/// The tools that change a file, which `guard.edit` looks at; every file tool
/// the entry is matched to is one `guard.corpus` looks at.
const EDITORS: &[&str] = &["Edit", "MultiEdit", "Write", "NotebookEdit"];

/// What Claude Code is told before a file tool runs, or `None` to let it run
/// in silence (ADR-43cbb20dcbf1).
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
fn pre_tool_use(payload: &Node, cwd: &Path) -> Option<String> {
    let tool = payload.member("tool_name")?.text()?;
    let input = payload.member("tool_input")?;
    let base = payload
        .member("cwd")
        .and_then(Node::text)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| cwd.to_path_buf());
    let repo = crate::repo::resolve(None, None, &base, &mut Vec::new()).ok()?;
    let cfg = crate::config::load(&repo.config_path()).ok()?;
    let targets = targets(tool, input, &base);
    if targets.is_empty() {
        return None;
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
            return None;
        }
        let shown = shown(target, &worktree);
        let reason = format!(
            "{shown} is under the corpus directory, which is reached through the ank CLI and \
             never by opening its files: `ank show <id>` prints an entity whole, `ank find \
             <query>` lists, `ank context <path>` serves what binds a perimeter.{}",
            note.map(|n| format!(" {n}")).unwrap_or_default()
        );
        return Some(deny(&reason));
    }

    if !EDITORS.contains(&tool) {
        return None;
    }
    let (level, note) = tightened(
        cfg.guard.edit,
        ank_contract::env::ANK_GUARD_EDIT,
        "guard.edit",
        GuardEdit::parse,
    );
    if level == GuardEdit::Off {
        return None;
    }
    let (identity, _) = crate::identity::resolved();
    // Outside the work tree a path is no perimeter, and nothing can cover it.
    let paths: Vec<String> = targets
        .iter()
        .filter_map(|t| t.strip_prefix(&worktree).ok())
        .filter_map(|rel| ank_core::normalize_path(&rel.to_string_lossy()))
        .filter(|rel| !rel.is_empty())
        .collect();
    let path = paths.first()?;
    if covered(&repo, &identity, path)? {
        return None;
    }
    let note = note.map(|n| format!(" {n}")).unwrap_or_default();
    if level == GuardEdit::Claim {
        let reason = format!(
            "guard.edit is claim, and no live claim of {identity} covers {path}: `ank context \
             {path}` names the task that does, and `ank claim <id>` takes it.{note}"
        );
        return Some(deny(&reason));
    }
    let message = format!(
        "no live claim of {identity} covers {path}: `ank context {path}` names what binds it \
         and what is claimable, and `ank claim <id>` takes the task.{note}"
    );
    let output = Obj::new()
        .str("hookEventName", "PreToolUse")
        .str("additionalContext", &message);
    Some(
        Obj::new()
            .str("systemMessage", &message)
            .obj("hookSpecificOutput", output)
            .finish(),
    )
}

/// The answer that refuses the tool call, with the reason Claude Code shows the
/// agent.
fn deny(reason: &str) -> String {
    let output = Obj::new()
        .str("hookEventName", "PreToolUse")
        .str("permissionDecision", "deny")
        .str("permissionDecisionReason", reason);
    Obj::new().obj("hookSpecificOutput", output).finish()
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
            // The name of the declared level, by asking the parse which name
            // reads as it: the table of names stays the config's.
            let named = ["deny", "off", "warn", "claim"]
                .into_iter()
                .find(|n| parse(n) == Some(declared))
                .unwrap_or_default();
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
                "hooks {sub} names its agent, and {} is the only one",
                AGENTS.join(", ")
            ),
        )
        .with_hint(hint));
    };
    if !AGENTS.contains(&name.as_str()) {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "no agent named '{name}': {} is the only one this version installs for",
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
    Ok(format!("\"{}\"", exe.display()))
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
            if items
                .iter()
                .any(|i| ours(i, event).as_deref() == Some(command))
            {
                break;
            }
            // In place, so whatever form the entry was written in survives it,
            // and `remove` still recognises a container install created.
            if let Some(stale) = items.iter().find_map(|i| command_of(i, event)) {
                let replaced = crate::json::string(command);
                text = format!("{}{replaced}{}", &text[..stale.start], &text[stale.end..]);
                added.push(*name);
                break;
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
        if created_hooks(&commands) == text[hooks.start..hooks.end] {
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
                && text[array.start..array.end] == format!("[{}]", compact(&command, *matcher))
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
