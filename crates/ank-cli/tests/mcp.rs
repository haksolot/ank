//! `ank mcp` through the binary (TASK-e655d28c83cb, ADR-fd98f4bc6dea).
//!
//! Driven as a process rather than called as a function, for the reason the
//! CLI's own golden harness gives and CLAUDE.md states as a rule: what a
//! protocol promises is what leaves the process. A test that called
//! `tools::list()` would prove the function and not the surface, and the surface
//! is what a client without a shell talks to.
//!
//! **It lives in `ank-cli` now, and the move is the task.** The suite used to
//! sit in `crates/ank-mcp` and spawn `ank-mcp`, because there was a second
//! executable to spawn. There is not: the surface is a verb, `crates/ank-mcp` is
//! a library linked into the one binary, and `CARGO_BIN_EXE_ank` is defined only
//! for the package that declares that binary -- which is the same mechanical
//! reason `tests/tui.rs` gives for sitting here. A suite that could not name the
//! binary would be back to testing the function instead of the process.
//!
//! Three claims are asserted here and all three come from the table rather than
//! from a list written beside it. The advertised tools **equal** `COMMANDS`, so
//! a verb added there reaches this surface with no edit in this crate and a verb
//! removed stops being advertised. A refused call carries **the code the table
//! declares for that refusal**, read out of `spec.refuses` rather than typed as
//! a number, so the assertion cannot drift from what `ank help --json`
//! publishes. And the document a call answers with is **the document the binary
//! prints**, compared against a direct run of the same verb, because the whole
//! of what the fold left alone is that a call is still a run of `ank`.

use ank_contract::{ExitCode, COMMANDS};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// A corpus with one task, built through the binary rather than by writing files.
fn corpus() -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "ank-mcp-it-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();

    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .expect("git is a hard dependency of this repository");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@ank.local"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "commit.gpgsign", "false"]);

    let run = |args: &[&str]| {
        let out = ank(&root, args);
        assert!(
            out.status.success(),
            "ank {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    run(&["init"]);
    run(&["config", "default_branch", "main"]);
    run(&[
        "new",
        "task",
        "--title",
        "A task to find",
        "--scope",
        "src/**",
        "--criteria",
        "A verifiable criterion.",
    ]);
    git(&["add", "-A"]);
    git(&["commit", "-qm", "a corpus"]);
    root
}

/// The binary, run directly, in the corpus and under the identity the server
/// gives its own children.
///
/// The environment matters here rather than being incidental: what the direct
/// run is for is comparing a document byte for byte against the one that came
/// back through the surface, and a call that ran under another identity or from
/// another directory would be a different call.
fn ank(repo: &Path, args: &[&str]) -> std::process::Output {
    Command::new(ANK)
        .args(args)
        .current_dir(repo)
        .env("ANK_AGENT", "test@ank.local")
        .output()
        .expect("the binary must have been built")
}

/// Sends every request, closes stdin, and returns the reply lines in order.
///
/// **No `ANK_BIN` and no path to an `ank` beside anything.** The server is the
/// binary, so the process a call runs is the process under test by construction
/// -- which is what the verb bought over the sibling that had to go looking
/// (ADR-fd98f4bc6dea).
fn talk(repo: &Path, requests: &[&str]) -> Vec<String> {
    let mut child = Command::new(ANK)
        .arg("mcp")
        .arg("--repo")
        .arg(repo)
        .env("ANK_AGENT", "test@ank.local")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary must have been built");
    {
        let stdin = child.stdin.as_mut().expect("piped");
        for request in requests {
            writeln!(stdin, "{request}").expect("the server must accept a request");
        }
    }
    let out = child.wait_with_output().expect("the server must finish");
    assert!(
        out.status.success(),
        "the server exited {:?}: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect()
}

/// A crude field read, deliberately: pulling one value out of a reply needs no
/// parser, and a parser here would be a second reading of the surface under test.
fn field(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let rest = rest.trim_start();
    if let Some(stripped) = rest.strip_prefix('"') {
        let end = stripped.find('"')?;
        return Some(stripped[..end].to_string());
    }
    let end = rest
        .find(|c: char| c == ',' || c == '}')
        .unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

/// The tool names a `tools/list` reply advertises, in the order it advertises
/// them.
fn advertised(reply: &str) -> Vec<String> {
    reply
        .split("\"name\":\"ank_")
        .skip(1)
        .map(|s| s.split('"').next().unwrap_or("").to_string())
        .collect()
}

/// The id of the task `corpus` seeded, read back out of the binary.
fn seeded_task(repo: &Path) -> String {
    let out = ank(repo, &["find", "--type", "task", "--json"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    text.split("\"id\":\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("the corpus was seeded with one task")
        .to_string()
}

/// The whole of what the verb has to serve: the handshake, the surface, and one
/// call that reaches the corpus.
///
/// **The three assertions are the criterion, in its order.** `initialize`
/// answers a server. `tools/list` advertises the table, all of it, in its order
/// -- so the count is `COMMANDS.len()` because the list *is* `COMMANDS`, and
/// asserting the equality says that and says which verb went missing when it
/// stops being true. And `tools/call` answers with the document the binary
/// prints for the same verb, compared against a direct run of it rather than
/// against a shape written down here: the fold left the dispatch alone, so the
/// two are the same bytes or the surface has grown a second one.
#[test]
fn the_verb_serves_the_handshake_the_table_and_the_binarys_own_document() {
    let repo = corpus();
    let id = seeded_task(&repo);
    let call = format!(
        r#"{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"ank_show","arguments":{{"arguments":["{id}"]}}}}}}"#
    );
    let replies = talk(
        &repo,
        &[
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{}}}"#,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
            &call,
        ],
    );

    // The notification must not have been answered: three requests carried an
    // id, so three replies is the whole of what a correct server sends.
    assert_eq!(
        replies.len(),
        3,
        "a notification was answered, which is a protocol error on our side: {replies:?}"
    );
    assert_eq!(
        field(&replies[0], "protocolVersion").as_deref(),
        Some("2025-06-18")
    );
    assert_eq!(field(&replies[0], "name").as_deref(), Some("ank-mcp"));

    let listed = advertised(&replies[1]);
    let table: Vec<String> = COMMANDS.iter().map(|c| c.name.to_string()).collect();
    assert_eq!(
        listed.len(),
        COMMANDS.len(),
        "the surface advertises {} tools and the table carries {}: a count that \
         disagrees with the table is either a curated subset or a list somebody \
         wrote by hand, and ADR-fd98f4bc6dea allows neither",
        listed.len(),
        COMMANDS.len()
    );
    assert_eq!(
        listed, table,
        "the advertised surface is not the table, in the table's order. \
         ADR-fd98f4bc6dea allows a full-surface passthrough and nothing else, so \
         any difference here is either a curated subset or a list somebody wrote \
         by hand"
    );
    // The verb is a tool of its own surface, and that is the rule working rather
    // than an oversight: a table walked whole cannot skip the row it is being
    // walked for, and skipping it would be the curation the ADR forbids.
    assert!(listed.contains(&"mcp".to_string()), "{:?}", listed);

    // The document, against the binary's own. `ank show --json` run directly in
    // the same corpus, under the same identity, is what a call is: the reply
    // carries those bytes escaped by the one escaper both sides use
    // (ADR-6fd69efb629c), so the comparison is of the document and not of a
    // rendering of it.
    let direct = ank(
        &repo,
        &["show", &id, "--repo", &repo.display().to_string(), "--json"],
    );
    assert!(direct.status.success(), "the direct run must succeed");
    let printed = String::from_utf8_lossy(&direct.stdout).trim().to_string();
    assert!(
        !printed.is_empty(),
        "the direct run printed nothing, so there is no document to compare"
    );
    assert!(
        replies[2].contains(&ank_contract::json::string(&printed)),
        "the call did not answer the document the binary prints. What folded is \
         the file and never the dispatch (ADR-fd98f4bc6dea), so these are the \
         same bytes.\nreply: {}\nbinary: {printed}",
        replies[2]
    );
    assert_eq!(
        field(&replies[2], "isError").as_deref(),
        Some("false"),
        "a call that succeeded is not an error result: {}",
        replies[2]
    );
    assert_eq!(
        field(&replies[2], "exitCode").as_deref(),
        Some("0"),
        "the exit code is on every result, success included: {}",
        replies[2]
    );

    let _ = std::fs::remove_dir_all(&repo);
}

#[test]
fn a_refused_call_carries_the_code_the_table_declares() {
    let repo = corpus();

    // The expectation comes out of the table, not out of this file: `show`
    // declares that it refuses an unknown id, and with which code.
    let show = ank_contract::spec_of("show").expect("show is a verb");
    let declared = show
        .refuses
        .iter()
        .find(|r| r.code == ExitCode::NotFound)
        .expect("show declares a refusal for an entity that is not there");

    let replies = talk(
        &repo,
        &[
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"ank_show","arguments":{"arguments":["TASK-000000000000"]}}}"#,
        ],
    );
    assert_eq!(replies.len(), 1, "{replies:?}");
    let reply = &replies[0];

    assert_eq!(
        field(reply, "exitCode").as_deref(),
        Some(declared.code.code().to_string().as_str()),
        "a refusal must carry the code the table declares for it ({}): {reply}",
        declared.when
    );
    assert_eq!(
        field(reply, "isError").as_deref(),
        Some("true"),
        "a refusal is an error result and not a silent success: {reply}"
    );
    // The refusal is the CLI's own, inherited rather than re-derived: the
    // self-correcting hint travels with it.
    assert!(
        reply.contains("error[2]") && reply.contains("ank find"),
        "the CLI's own error and its hint must survive the passthrough: {reply}"
    );

    let _ = std::fs::remove_dir_all(&repo);
}

#[test]
fn the_server_refuses_a_flag_that_would_make_it_two_corpora() {
    let repo = corpus();
    let replies = talk(
        &repo,
        &[
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"ank_find","arguments":{"repo":"/somewhere/else"}}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ank_find","arguments":{"nonsense":"x"}}}"#,
        ],
    );
    assert!(
        replies[0].contains("belongs to the server"),
        "--repo must be refused by name: the corpus is the server's to choose \
         and never the caller's (ADR-fd98f4bc6dea): {}",
        replies[0]
    );
    assert!(
        replies[1].contains("takes no --nonsense"),
        "a flag the verb does not take is refused by name, against the table: {}",
        replies[1]
    );
    let _ = std::fs::remove_dir_all(&repo);
}
