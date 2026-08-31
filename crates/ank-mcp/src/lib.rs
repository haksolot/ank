//! The Ank protocol surface: every verb, over MCP, generated from one table.
//!
//! ADR-fd98f4bc6dea permits this and permits nothing else: a full-surface
//! passthrough or no surface at all. It supersedes an earlier refusal, kept
//! whole as the shape of what is now allowed. The proposal that refusal rejected
//! exposed four verbs out of twenty-two, which rebuilt under a
//! second protocol the agent-surface split that had been abolished once already,
//! and rebuilt the worse half of it: a caller reached through the protocol could
//! not amend, could not propose an ADR, could not check the corpus.
//!
//! **Nothing in this crate names a verb.** [`tools`] walks
//! [`ank_contract::COMMANDS`], and [`call`] spawns the binary. What this surface
//! has is what the CLI dispatches, and it cannot be otherwise without an edit to
//! the table both consume.
//!
//! **A call names its corpus, and a server never merges two.** Every tool
//! carries an optional `corpus` argument holding the repository identity of
//! ADR-621a7fd96ce1; absent, the call goes to the corpus the process was
//! addressed with at startup, the way `--repo` addresses one. The set a server
//! may reach is that corpus plus what the reader declared in `corpora.yml`, and
//! nothing is discovered: an identity nobody declared is refused by name.
//! [`corpora`] is the whole of it, and it hands [`call`] one path.
//!
//! **Multiplexing is what was permitted; merging is what stays forbidden**, in
//! the same words the superseded decision used. There is no merged claim space,
//! no claim held on a client's behalf, and no pooling of clients under one
//! identity — because `refs/ank/*` is per repository, and a server that merged
//! two claim spaces would be inventing an arbitration the refs cannot carry.
//! Nothing in this crate reasons about two corpora at once: a resolution returns
//! one path, and every call is `ank --repo <one corpus> <verb> --json`.
//!
//! **This is a library, and the surface is the verb `ank mcp`** (ADR-fd98f4bc6dea).
//! It was a sibling executable, and the file folded into the one binary every
//! route carries for the reason ADR-8bd76e8d7c4e gave for `tui`: a separate file
//! is invisible to precisely the people it exists for, and has to be
//! distributed, documented and discovered as a third thing.
//!
//! **One executable is not one process, and that is what the fold left alone.**
//! [`call`] still spawns `ank <verb> --repo <corpus> --json` per call. Linking
//! the CLI's dispatch in here would re-derive every refusal, and anything
//! re-derived can differ; spawning inherits them by construction. So what folded
//! is the file and never the dispatch, and `crates/ank-mcp/tests/dependencies.rs`
//! reads that back out of the build rather than trusting this paragraph.
//!
//! **The version a client is told is the binary's, and this crate's own number
//! reaches nobody.** Both places a version leaves this surface -- `serverInfo`
//! at the handshake and the `ank-mcp/<version>` identity a call writes under --
//! read [`Address::version`], which the dispatch hands down. The argument is on
//! that field.
//!
//! **This does not make a protocol the preferred route.** §2's common denominator
//! is shell, that is still what the skill teaches, and this exists for the client
//! that has no shell at all.

mod call;
mod corpora;
mod tools;

use ank_contract::json::{string, Obj};
use std::io::{BufRead, Write};
use std::path::PathBuf;

/// The MCP revision this server implements.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// Where the surface reaches, resolved by the dispatch and never here.
///
/// Every field is the caller's foundation rather than this crate's: the verb
/// resolves the corpus the way every other verb resolves it, so a missing
/// `.ank/` is the refusal it already is instead of a JSON-RPC error a client
/// would have to decode, and it names the binary so that this crate has one
/// road out of the process and no search to get it wrong.
pub struct Address {
    /// The binary a call runs. `std::env::current_exe()` of the process serving
    /// the verb -- see the note on [`call`].
    pub exe: PathBuf,
    /// The version that binary answers `--version` with, and the only version
    /// this surface ever tells anyone (TASK-ae64d1c5678d).
    ///
    /// **This is the choice, and it is made here.** `crates/ank-mcp` carries a
    /// version of its own, as every crate must, and until now that number was
    /// what a client read in `serverInfo` and what landed in the
    /// `ank-mcp/<version>` identity every claim taken through this surface is
    /// written under. It was held to the release tag by
    /// `.github/scripts/check-version.sh`, and the reason that gate gave was
    /// that the release shipped `ank-mcp` as a file of its own. It does not any
    /// more (ADR-1ea31c2f3c5a): the surface is a verb, so the file went and the
    /// gate went with it -- correctly, by the reason as written. What the reason
    /// never covered is that the number went on reaching a client anyway.
    ///
    /// Two repairs were available. Gate this crate's version against the tag
    /// again: that restores a rule whose stated justification is gone, and
    /// leaves two numbers that agree only because somebody remembered -- one
    /// more of the literals `check-version.sh` exists because nobody gets right
    /// forever. Or stop the crate's number reaching a client at all, which is
    /// this: what a client is told is the version of the executable it is
    /// talking to, and `crates/ank-mcp/Cargo.toml`'s literal becomes what a
    /// crate version is for, a number cargo resolves the workspace with.
    ///
    /// **Handed down, because it cannot honestly be computed here.** This crate
    /// must not link `ank-cli` -- that is ADR-fd98f4bc6dea and
    /// `tests/dependencies.rs` reads it back out of the build -- so
    /// `CARGO_PKG_VERSION` here can only ever name this crate. The dispatch
    /// already hands over what the surface cannot work out for itself, and this
    /// is one more of those. It is exact rather than approximately right:
    /// [`Address::exe`] is `current_exe()`, the process serving the verb, so the
    /// version compiled into that process *is* the version it prints. There is
    /// nothing to parse and nothing to keep in step.
    pub version: String,
    /// The corpus the process was addressed with, which is where a call goes
    /// when it names none.
    ///
    /// **The startup corpus, and not the only one a server can reach.** A call
    /// may name another by the identity of ADR-621a7fd96ce1, resolved against
    /// what the reader declared ([`corpora`]). This one is the default and is
    /// resolved by the dispatch, so a client that names nothing is served
    /// exactly as it was before a call could name anything.
    pub repo: PathBuf,
}

/// Serves the session: one message in, at most one out, until the client stops
/// writing.
///
/// **Flushed per reply, and that is not tidiness.** A client speaks to this over
/// a pipe and waits for an answer before it sends the next request, so a reply
/// still sitting in a buffer is a session that has deadlocked rather than one
/// that is slow.
///
/// A line that will not parse is answered and the session continues: JSON-RPC
/// reserves a code for exactly that, and a server that hung up on one would take
/// the client's other work with it. The loop ends on end of input, which is what
/// a client closing its side means, and on nothing else.
pub fn serve(address: &Address, input: &mut dyn BufRead, out: &mut dyn Write) {
    // Built once and per session, because what it resolves is per session: the
    // reader's declarations and the startup corpus's own name do not change
    // under a running server, and reading them per call would ask the same
    // question of the same file for every message a client sends. It resolves
    // nothing until a call names a corpus.
    let reach = corpora::Reach::new(address);
    for line in input.lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = handle(&line, &reach) {
            let _ = writeln!(out, "{reply}");
            let _ = out.flush();
        }
    }
}

/// One JSON-RPC message in, at most one out.
///
/// `None` for a notification, which is a message with no `id` and must never be
/// answered: a reply to one is a protocol error on our side, and the client that
/// sent `notifications/initialized` is waiting for nothing.
///
/// **Read as JSON, because it is JSON** (TASK-1bc1186ad9e7). This was
/// `serde_yaml::from_str`, on the manifest's claim that YAML 1.2 is a superset
/// of JSON. The grammar is; the resolver is not. RFC 8259 spells a non-BMP code
/// point as a surrogate pair and YAML's `\u` escape is a single 16-bit unit
/// that does not pair, so `{"id":"\ud83d\ude00-alpha"}` -- which is what
/// `json.dumps` emits by default -- was answered `-32700` with `id: null`, and
/// a repeated key was refused where JSON takes the last one. Both were measured
/// through the binary and both are held there, by
/// `crates/ank-cli/tests/mcp.rs`.
///
/// **A parse failure still answers `id: null`, and that is not the same
/// silence.** The reply below is for a line no reader can attribute, which after
/// this change means a line that is not JSON at all rather than a JSON document
/// this reader happened not to accept. The client is told which of its requests
/// failed by the fact that one did, and there is nothing else honest to say.
fn handle(line: &str, reach: &corpora::Reach) -> Option<String> {
    let request: serde_json::Value = match serde_json::from_str(line) {
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
                // **`ank-mcp` and not `ank`**, although the executable is now
                // `ank`. This names the *server* a client configured, not the
                // file it launched, and a client's own configuration keys off
                // it: renaming it would rename every entry in every client
                // that already talks to this surface, to say something the
                // command line beside it already says.
                //
                // The version beside it is the binary's, not this crate's, and
                // is the same value `call::identity` writes into the process
                // identity: a client reads a version here and an agent reads
                // one off a claim record, and the two are one number or they
                // are a drift nobody can see (see [`Address::version`]).
                .obj(
                    "serverInfo",
                    Obj::new()
                        .str("name", "ank-mcp")
                        .str("version", &reach.address().version),
                )
                .str(
                    "instructions",
                    "Every verb the ank CLI dispatches is a tool here, generated \
                     from one table. Start with ank_context: it answers what binds \
                     this perimeter and what is claimable. Every tool takes an \
                     optional corpus argument, the root commit ank status --json \
                     prints under \"corpus\": absent, a call goes to the corpus this \
                     server was addressed with at startup; given, it goes to that \
                     corpus alone, and only corpora the reader declared can be \
                     named. Each corpus is addressed on its own and none is merged \
                     with another: a claim is taken in one repository, by the refs \
                     of that repository.",
                )
                .finish(),
        )),
        "ping" => Some(result(id_json, &Obj::new().finish())),
        "tools/list" => Some(result(
            id_json,
            &Obj::new().raw("tools", &tools::list()).finish(),
        )),
        "tools/call" => Some(call_tool(id_json, request.get("params"), reach)),
        other => Some(error(id_json, -32601, &format!("no such method '{other}'"))),
    }
}

/// `tools/call`, which is the only method that reaches a corpus.
///
/// **The corpus is settled before anything runs, and exactly once.** A call
/// names one or names none, [`corpora::Reach::resolve`] turns that into one
/// path, and a name it cannot resolve is refused with a §4 code and the command
/// that resolves it -- the verb is not spawned, no corpus is opened, and nothing
/// falls back to the corpus the caller did not ask for. Falling back would be
/// the worst answer available here: a claim taken in a corpus the client did not
/// name, reported as success.
fn call_tool(
    id: Option<String>,
    params: Option<&serde_json::Value>,
    reach: &corpora::Reach,
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
    let mut named: Option<String> = None;
    if let Some(map) = params.get("arguments").and_then(|a| a.as_object()) {
        for (key, value) in map {
            let key = key.as_str();
            // The one argument that is the surface's rather than the verb's, so
            // it is taken out before the table is consulted: the table knows
            // nothing about it and would refuse it by name, which is the right
            // answer for every key except this one.
            if key == corpora::ARGUMENT {
                named = Some(scalar(value));
                continue;
            }
            if key == "arguments" {
                if let Some(list) = value.as_array() {
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
                    &format!(
                        "{flag} belongs to the server: name a corpus with the \
                         {} argument, by the identity ank status --json prints, \
                         never by a path",
                        corpora::ARGUMENT
                    ),
                );
            }
            let values = match (known.takes_value, value.as_array()) {
                (false, _) => vec![String::new()],
                (true, Some(list)) => list.iter().map(scalar).collect(),
                (true, None) => vec![scalar(value)],
            };
            args.flags.push((key.to_string(), values));
        }
    }

    // A corpus this server cannot reach is a refusal and not a protocol error:
    // the request is well formed and the answer is no, which is the line
    // `Outcome::refused` already draws. It reaches the client through the same
    // renderer a refusal from the binary does, so the two shapes cannot drift.
    let corpus = match reach.resolve(named.as_deref()) {
        Ok(corpus) => corpus,
        Err(refusal) => return result(id, &refusal.outcome().to_result()),
    };

    match call::run(spec, reach.address(), &corpus, &args) {
        Ok(outcome) => result(id, &outcome.to_result()),
        // The binary itself could not be run. That is the environment and not the
        // corpus, and §4 keeps the two apart on purpose.
        Err(e) => error(
            id,
            -32603,
            &format!("cannot run {}: {e}", reach.address().exe.display()),
        ),
    }
}

/// A scalar argument as the string the command line will carry.
fn scalar(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// The request id, rendered back exactly as JSON-RPC requires: a string stays a
/// string and a number stays a number, because a client matching replies by
/// identity would not recognise a retyped one.
fn render_id(id: &serde_json::Value) -> String {
    match id {
        serde_json::Value::String(s) => string(s),
        serde_json::Value::Number(n) => n.to_string(),
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
