//! The server, driven as a process.
//!
//! Driven rather than called, for the reason the CLI's own golden harness gives:
//! what a protocol promises is what leaves the process. A test that called
//! `tools::list()` would prove the function and not the surface, and the surface
//! is what a client without a shell talks to.
//!
//! Two claims are asserted here and both come from the table rather than from a
//! list written beside it. The advertised tools **equal** `COMMANDS`, so a verb
//! added there reaches this surface with no edit in this crate and a verb removed
//! stops being advertised. And a refused call carries **the code the table
//! declares for that refusal**, read out of `spec.refuses` rather than typed as a
//! number, so the assertion cannot drift from what `ank help --json` publishes.

use ank_contract::{ExitCode, COMMANDS};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

fn bin(name: &str) -> PathBuf {
    // The sibling rule the server itself follows: `ank` sits beside `ank-mcp`
    // because ADR-372b82af1ec7 ships them together. In a test tree that is
    // `target/<profile>/`, which is also where cargo puts both.
    let mcp = PathBuf::from(env!("CARGO_BIN_EXE_ank-mcp"));
    let dir = mcp.parent().expect("a built binary has a directory");
    let exe = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    dir.join(exe)
}

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

    let ank = bin("ank");
    assert!(
        ank.is_file(),
        "the CLI must be built beside the server: cargo build --workspace"
    );
    let run = |args: &[&str]| {
        let out = Command::new(&ank)
            .args(args)
            .current_dir(&root)
            .env("ANK_AGENT", "test@ank.local")
            .output()
            .expect("the binary must have been built");
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

/// Sends every request, closes stdin, and returns the reply lines in order.
fn talk(repo: &Path, requests: &[&str]) -> Vec<String> {
    let mut child = Command::new(bin("ank-mcp"))
        .arg("--repo")
        .arg(repo)
        .env("ANK_BIN", bin("ank"))
        .env("ANK_AGENT", "test@ank.local")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the server must have been built");
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

#[test]
fn the_surface_it_advertises_is_the_table_and_nothing_else() {
    let repo = corpus();
    let replies = talk(
        &repo,
        &[
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{}}}"#,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        ],
    );

    // The notification must not have been answered: two requests carried an id,
    // so two replies is the whole of what a correct server sends.
    assert_eq!(
        replies.len(),
        2,
        "a notification was answered, which is a protocol error on our side: {replies:?}"
    );
    assert_eq!(field(&replies[0], "name").as_deref(), Some("ank-mcp"));

    let listed: Vec<String> = replies[1]
        .split("\"name\":\"ank_")
        .skip(1)
        .map(|s| s.split('"').next().unwrap_or("").to_string())
        .collect();
    let table: Vec<String> = COMMANDS.iter().map(|c| c.name.to_string()).collect();
    assert_eq!(
        listed, table,
        "the advertised surface is not the table, in the table's order. \
         ADR-372b82af1ec7 allows a full-surface passthrough and nothing else, so \
         any difference here is either a curated subset or a list somebody wrote \
         by hand"
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
        "--repo must be refused by name: one process speaks for one corpus \
         (ADR-372b82af1ec7): {}",
        replies[0]
    );
    assert!(
        replies[1].contains("takes no --nonsense"),
        "a flag the verb does not take is refused by name, against the table: {}",
        replies[1]
    );
    let _ = std::fs::remove_dir_all(&repo);
}
