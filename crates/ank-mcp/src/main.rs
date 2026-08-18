//! The Ank protocol surface: every verb, over MCP, generated from one table.
//!
//! ADR-372b82af1ec7 permits this and permits nothing else: a full-surface
//! passthrough or no surface at all. It supersedes ADR-1713af205186, whose
//! refusal is kept whole as the shape of what is now allowed. The proposal that
//! ADR rejected exposed four verbs out of twenty-two, which rebuilt under a
//! second protocol the agent-surface split that had been abolished once already,
//! and rebuilt the worse half of it: a caller reached through the protocol could
//! not amend, could not propose an ADR, could not check the corpus.
//!
//! **Nothing in this crate names a verb.** [`tools`] walks
//! [`ank_contract::COMMANDS`], and [`call`] spawns the binary. What this surface
//! has is what the CLI dispatches, and it cannot be otherwise without an edit to
//! the table both consume.
//!
//! **One process speaks for one corpus.** Addressed once, at startup, the way
//! `--repo` addresses one. A per-call repository would make one server several
//! corpora and invent an arbitration `refs/ank/*` cannot carry, which is the one
//! thing ADR-372b82af1ec7 spells out at length: claims stay per repository, and a
//! deployment over several is several servers.
//!
//! **This does not make a protocol the preferred route.** §2's common denominator
//! is shell, that is still what the skill teaches, and this exists for the client
//! that has no shell at all.

mod call;
mod tools;

use ank_contract::json::{string, Obj};
use ank_contract::ExitCode;
use std::io::{BufRead, Write};
use std::path::PathBuf;

/// The MCP revision this server implements.
const PROTOCOL_VERSION: &str = "2025-06-18";

fn main() {
    let repo = match corpus() {
        Ok(path) => path,
        Err(message) => {
            // The one refusal that happens before any client is listening, so it
            // goes where a person will see it and carries §4's environment code.
            eprintln!("error[{}]: {message}", ExitCode::Environment);
            eprintln!("  -> ank-mcp --repo <path to a directory holding .ank/>");
            std::process::exit(ExitCode::Environment.code());
        }
    };

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = handle(&line, &repo) {
            let _ = writeln!(stdout, "{reply}");
            let _ = stdout.flush();
        }
    }
}

/// The corpus this process speaks for, from `--repo` or the working directory.
///
/// Resolved once and never again: the value is what every call is given, so a
/// server cannot drift between corpora while a client holds a claim in one.
fn corpus() -> Result<PathBuf, String> {
    let mut args = std::env::args().skip(1);
    let mut given: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo" | "-r" => match args.next() {
                Some(path) => given = Some(PathBuf::from(path)),
                None => return Err("--repo needs a path".to_string()),
            },
            "--version" | "-V" => {
                println!("ank-mcp {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    let root = match given {
        Some(path) => path,
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };
    match root.join(".ank").is_dir() {
        true => Ok(root),
        false => Err(format!("no .ank/ under {}", root.display())),
    }
}

/// One JSON-RPC message in, at most one out.
///
/// `None` for a notification, which is a message with no `id` and must never be
/// answered: a reply to one is a protocol error on our side, and the client that
/// sent `notifications/initialized` is waiting for nothing.
fn handle(line: &str, repo: &std::path::Path) -> Option<String> {
    let request: serde_yaml::Value = match serde_yaml::from_str(line) {
        Ok(value) => value,
        // Unparseable and therefore unattributable: no id to answer against, so
        // the only honest reply is the one JSON-RPC reserves for it.
        Err(e) => return Some(error(None, -32700, &format!("parse error: {e}"))),
    };
    let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = request.get("id").cloned().filter(|v| !v.is_null());
    let id_json = id.as_ref().map(render_id);

    if id_json.is_none() {
        // A notification. `initialized` is the only one that matters and it wants
        // silence, not acknowledgement.
        return None;
    }

    match method {
        "initialize" => Some(result(
            id_json,
            &Obj::new()
                .str("protocolVersion", PROTOCOL_VERSION)
                .obj("capabilities", Obj::new().obj("tools", Obj::new()))
                .obj(
                    "serverInfo",
                    Obj::new()
                        .str("name", "ank-mcp")
                        .str("version", env!("CARGO_PKG_VERSION")),
                )
                .str(
                    "instructions",
                    "Every verb the ank CLI dispatches is a tool here, generated \
                     from one table. Start with ank_context: it answers what binds \
                     this perimeter and what is claimable. This server speaks for \
                     one corpus, fixed when it started.",
                )
                .finish(),
        )),
        "ping" => Some(result(id_json, &Obj::new().finish())),
        "tools/list" => Some(result(
            id_json,
            &Obj::new().raw("tools", &tools::list()).finish(),
        )),
        "tools/call" => Some(call_tool(id_json, request.get("params"), repo)),
        other => Some(error(id_json, -32601, &format!("no such method '{other}'"))),
    }
}

/// `tools/call`, which is the only method that reaches the corpus.
fn call_tool(
    id: Option<String>,
    params: Option<&serde_yaml::Value>,
    repo: &std::path::Path,
) -> String {
    let params = match params {
        Some(p) => p,
        None => return error(id, -32602, "tools/call needs params"),
    };
    let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let Some(spec) = tools::verb_of(name) else {
        // Not a refusal on state and not a curated subset: a name this surface
        // never advertised. The table is what it advertised.
        return error(id, -32602, &format!("no such tool '{name}'"));
    };

    let mut args = call::Arguments::default();
    if let Some(map) = params.get("arguments").and_then(|a| a.as_mapping()) {
        for (key, value) in map {
            let Some(key) = key.as_str() else { continue };
            if key == "arguments" {
                if let Some(list) = value.as_sequence() {
                    for item in list {
                        args.positionals.push(scalar(item));
                    }
                } else {
                    args.positionals.push(scalar(value));
                }
                continue;
            }
            let flag = format!("--{key}");
            // **Validated against the table, and refused by name when it is not
            // there.** A flag the verb does not take would otherwise reach the
            // CLI and be refused there, which is the same answer arriving less
            // usefully: the client asked this surface, so this surface says the
            // name is wrong.
            let Some(known) = ank_contract::find_flag(spec, &flag) else {
                return error(id, -32602, &format!("{} takes no {flag}", spec.name));
            };
            if !tools::client_flag(&flag) {
                return error(
                    id,
                    -32602,
                    &format!("{flag} belongs to the server: one process, one corpus"),
                );
            }
            let values = match (known.takes_value, value.as_sequence()) {
                (false, _) => vec![String::new()],
                (true, Some(list)) => list.iter().map(scalar).collect(),
                (true, None) => vec![scalar(value)],
            };
            args.flags.push((key.to_string(), values));
        }
    }

    match call::run(spec, repo, &args) {
        Ok(outcome) => result(id, &outcome.to_result()),
        // The binary itself could not be run. That is the environment and not the
        // corpus, and §4 keeps the two apart on purpose.
        Err(e) => error(
            id,
            -32603,
            &format!("cannot run {}: {e}", call::ank_binary().display()),
        ),
    }
}

/// A scalar argument as the string the command line will carry.
fn scalar(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        other => serde_yaml::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    }
}

/// The request id, rendered back exactly as JSON-RPC requires: a string stays a
/// string and a number stays a number, because a client matching replies by
/// identity would not recognise a retyped one.
fn render_id(id: &serde_yaml::Value) -> String {
    match id {
        serde_yaml::Value::String(s) => string(s),
        serde_yaml::Value::Number(n) => n.to_string(),
        other => string(&scalar(other)),
    }
}

fn result(id: Option<String>, payload: &str) -> String {
    let mut doc = Obj::new().str("jsonrpc", "2.0");
    if let Some(id) = id {
        doc = doc.raw("id", &id);
    }
    doc.raw("result", payload).finish()
}

fn error(id: Option<String>, code: i32, message: &str) -> String {
    let mut doc = Obj::new().str("jsonrpc", "2.0");
    match id {
        Some(id) => doc = doc.raw("id", &id),
        None => doc = doc.null("id"),
    }
    doc.obj(
        "error",
        Obj::new().num("code", code).str("message", message),
    )
    .finish()
}
