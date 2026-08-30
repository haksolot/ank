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
//! Three claims about the surface are asserted here and all three come from the
//! table rather than from a list written beside it. The advertised tools
//! **equal** `COMMANDS`, so
//! a verb added there reaches this surface with no edit in this crate and a verb
//! removed stops being advertised. A refused call carries **the code the table
//! declares for that refusal**, read out of `spec.refuses` rather than typed as
//! a number, so the assertion cannot drift from what `ank help --json`
//! publishes. And the document a call answers with is **the document the binary
//! prints**, compared against a direct run of the same verb, because the whole
//! of what the fold left alone is that a call is still a run of `ank`.
//!
//! A fourth claim is about a number rather than about the table: the version a
//! client is told, and the one in the identity a claim is written under, is the
//! one `ank --version` prints (TASK-ae64d1c5678d). It is read out of the process
//! for the same reason everything else here is.

mod scratch;

use ank_contract::{ExitCode, COMMANDS};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// A corpus with one task, built through the binary rather than by writing files.
fn corpus() -> PathBuf {
    corpus_titled("A task to find", None)
}

/// The same, with the task's title chosen and the reader's home named.
///
/// Both are what the multi-corpus suite needs and no other test does. A title
/// tells two corpora apart in an answer, which is how "neither names the other's
/// task" is asserted on content rather than on an id alone; a home is what makes
/// the reader's declarations the test's and not the machine's.
fn corpus_titled(title: &str, home: Option<&Path>) -> PathBuf {
    let root = scratch::dir("corpus");
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
    // Maintenance off, because git is otherwise free to repack a fixture
    // between two reads of it (TASK-fc6bef21e268).
    git(&["config", "gc.auto", "0"]);
    git(&["config", "maintenance.auto", "false"]);

    let run = |args: &[&str]| {
        let out = ank_at(&root, home, args);
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
        title,
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
    ank_at(repo, None, args)
}

/// The same run, with the reader's home named.
///
/// `None` leaves the machine's, which is what every single-corpus test wants:
/// the corpora it builds are keyed on root commits nobody has declared anywhere,
/// so the reader's real map answers nothing about them either way.
fn ank_at(repo: &Path, home: Option<&Path>, args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new(ANK);
    cmd.args(args)
        .current_dir(repo)
        .env("ANK_AGENT", "test@ank.local");
    if let Some(home) = home {
        let (key, value) = reader_home(home);
        cmd.env(key, value);
    }
    cmd.output().expect("the binary must have been built")
}

/// The environment variable that names where a reader's declarations live, and
/// the value to give it for `home`.
///
/// The rule is `ank_contract::events::user_dir`'s, read from the outside: this
/// is a test of the file the surface actually opens, so it must name it the way
/// the surface resolves it and not the way one platform spells it.
fn reader_home(home: &Path) -> (&'static str, PathBuf) {
    match cfg!(windows) {
        true => ("APPDATA", home.to_path_buf()),
        false => ("XDG_CONFIG_HOME", home.to_path_buf()),
    }
}

/// Sends every request, closes stdin, and returns the reply lines in order.
///
/// **No `ANK_BIN` and no path to an `ank` beside anything.** The server is the
/// binary, so the process a call runs is the process under test by construction
/// -- which is what the verb bought over the sibling that had to go looking
/// (ADR-fd98f4bc6dea).
fn talk(repo: &Path, requests: &[&str]) -> Vec<String> {
    talk_at(repo, None, requests)
}

/// The same session, with the reader's home named: one server, addressed with
/// one corpus at startup, reaching whatever that home declares and nothing else.
fn talk_at(repo: &Path, home: Option<&Path>, requests: &[&str]) -> Vec<String> {
    let mut cmd = Command::new(ANK);
    cmd.arg("mcp")
        .arg("--repo")
        .arg(repo)
        .env("ANK_AGENT", "test@ank.local")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(home) = home {
        let (key, value) = reader_home(home);
        cmd.env(key, value);
    }
    let mut child = cmd.spawn().expect("the binary must have been built");
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

/// A session under no named identity, which is the only way to see the one the
/// server gives itself.
///
/// `ANK_AGENT` is *removed* rather than left alone: every other session here
/// sets it, and a machine that happened to have it exported would otherwise
/// make the typed identity untestable exactly where it matters -- an agent's
/// own shell, which is where this suite runs.
fn talk_unnamed(repo: &Path, requests: &[&str]) -> Vec<String> {
    let mut child = Command::new(ANK)
        .arg("mcp")
        .arg("--repo")
        .arg(repo)
        .env_remove("ANK_AGENT")
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

/// The version out of `serverInfo`, and not merely the first `"version"` in the
/// reply.
///
/// [`field`] would find the right one today, since `protocolVersion` is spelled
/// with a capital and nothing else in the handshake carries the key. That is a
/// fact about how the reply is written rather than about what is being asserted,
/// so the object is sliced first: a key added above it must not quietly turn
/// this into a test of something else.
fn server_info_version(reply: &str) -> Option<String> {
    let at = reply.find("\"serverInfo\"")?;
    field(&reply[at..], "version")
}

/// A fragment as it reads once the surface has escaped a document into a `text`
/// block.
///
/// The same escaper both sides share (ADR-6fd69efb629c), applied to a fragment
/// instead of a whole document, with the quotes it wraps a string in taken back
/// off. Escaping by hand here would be a second escaper to keep in step with the
/// one under test.
fn escaped(fragment: &str) -> String {
    let quoted = ank_contract::json::string(fragment);
    quoted[1..quoted.len() - 1].to_string()
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

// ---------------------------------------------------------------------------
// Several corpora, and never a merged one (ADR-fd98f4bc6dea, TASK-2f31789f6af2)
// ---------------------------------------------------------------------------
//
// **The clause under test is a distinction, so the test is about what does not
// happen.** Multiplexing is permitted: one server may address several corpora,
// each on its own, the way `--repo` addresses one. Merging is forbidden in the
// same words the superseded decision used -- no merged claim space, no claim
// held on a client's behalf, no arbitration across clones. Two claims taken
// through one server is the permitted half; the forbidden half is only visible
// as an absence, and the place an absence can be read is `refs/ank/claims`,
// which is per repository and is where a merged claim space would have had to
// leave a trace.
//
// **A temporary home, and not the machine's.** What a server may reach is what
// the reader declared, so a test that used the developer's own `corpora.yml`
// would be asserting something about their laptop. The declaration is written
// with `ank config --user`, which is the verb that owns the file (§4,
// ADR-96174f1ac2b7): the surface then finds it by resolving the same home, so a
// disagreement about where that file lives fails here rather than in prose.

/// A reader's home, empty, with no corpus declared in it yet.
fn declaring_home() -> PathBuf {
    scratch::dir("home")
}

/// The repository identity of a corpus, read out of the binary.
///
/// `ank status --json` under `"corpus"` is where ADR-96174f1ac2b7 sends a reader
/// looking for the key, and it is what the surface's own argument documents. So
/// the value this test names a corpus with is the value a client would have
/// obtained, rather than a root commit read out of git here.
fn corpus_identity(repo: &Path, home: &Path) -> String {
    let out = ank_at(repo, Some(home), &["status", "--json"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let id = field(&text, "corpus").expect("status names the corpus it answered about");
    assert_eq!(id.len(), 40, "an identity is a root commit: {id}");
    id
}

/// Every claim ref a corpus holds, by the task each one names.
fn claimed_tasks(repo: &Path) -> Vec<String> {
    let out = Command::new("git")
        .args(["for-each-ref", "--format=%(refname)", "refs/ank/claims"])
        .current_dir(repo)
        .output()
        .expect("git is a hard dependency of this repository");
    assert!(out.status.success(), "git for-each-ref failed");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.rsplit('/').next().map(str::to_string))
        .collect()
}

/// One server, two corpora, two claims, and neither corpus carrying a trace of
/// the other.
///
/// **This is the criterion, and every assertion below is one of its clauses.**
/// A call omitting the argument reaches the corpus the process was addressed
/// with, which is the backwards compatibility clause and is asserted first
/// because every client that exists today makes exactly that call. A call naming
/// a declared identity reaches that corpus and takes a claim there. An identity
/// nobody declared is refused by name, with a §4 code and the command that
/// resolves it, and -- the half that matters -- does not quietly fall back to
/// the corpus the caller did not name. And each corpus ends holding its own
/// claim on its own `refs/ank/claims`, naming its own task and nothing of the
/// other's, which is the ban on a merged claim space made observable.
#[test]
fn two_corpora_through_one_server_land_two_claims_and_no_third() {
    let home = declaring_home();
    let one = corpus_titled("The task of the first corpus", Some(&home));
    let two = corpus_titled("The task of the second corpus", Some(&home));
    let id_two = corpus_identity(&two, &home);
    let task_one = seeded_task(&one);
    let task_two = seeded_task(&two);
    assert_ne!(task_one, task_two, "two corpora mint two ids");

    // Declared with the verb that owns the map. The startup corpus is
    // deliberately *not* declared: it is reachable because it is the startup
    // corpus, and asserting that is asserting the set is "what the reader
    // declared plus that one" rather than "what the reader declared".
    let declare = ank_at(
        &two,
        Some(&home),
        &[
            "config",
            "--user",
            &format!("corpora.{id_two}"),
            &two.display().to_string(),
        ],
    );
    assert!(
        declare.status.success(),
        "the declaration must be written: {}",
        String::from_utf8_lossy(&declare.stderr)
    );

    // A root commit of the right shape that no corpus has and nobody declared.
    let nobodys = "0".repeat(40);

    let replies = talk_at(
        &one,
        Some(&home),
        &[
            &format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"ank_claim","arguments":{{"arguments":["{task_one}"]}}}}}}"#
            ),
            &format!(
                r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"ank_claim","arguments":{{"arguments":["{task_two}"],"corpus":"{id_two}"}}}}}}"#
            ),
            &format!(
                r#"{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"ank_find","arguments":{{"corpus":"{nobodys}"}}}}}}"#
            ),
            &format!(
                r#"{{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{{"name":"ank_find","arguments":{{"corpus":"{id_two}"}}}}}}"#
            ),
            r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"ank_find","arguments":{}}}"#,
        ],
    );
    assert_eq!(replies.len(), 5, "{replies:?}");

    // 1. No corpus named: the corpus the process was addressed with, unchanged
    //    from every client that existed before the argument did.
    assert_eq!(
        field(&replies[0], "exitCode").as_deref(),
        Some("0"),
        "a call naming no corpus must reach the startup corpus: {}",
        replies[0]
    );
    assert!(
        replies[0].contains(&task_one),
        "the claim was taken on the startup corpus's own task: {}",
        replies[0]
    );

    // 2. A declared identity: that corpus, addressed on its own.
    assert_eq!(
        field(&replies[1], "exitCode").as_deref(),
        Some("0"),
        "a declared corpus must be reachable by name: {}",
        replies[1]
    );
    assert!(
        replies[1].contains(&task_two),
        "the claim was taken on the named corpus's own task: {}",
        replies[1]
    );

    // 3. An identity nobody declared: refused by name, and nothing runs.
    //    The code is what settles that no fallback happened -- a call that had
    //    quietly gone to the startup corpus would have answered that corpus's
    //    listing with exit 0.
    assert_eq!(
        field(&replies[2], "exitCode").as_deref(),
        Some(ExitCode::Environment.code().to_string().as_str()),
        "an undeclared corpus is refused with the code §4 declares for an \
         environment to repair: {}",
        replies[2]
    );
    assert_eq!(
        field(&replies[2], "isError").as_deref(),
        Some("true"),
        "a refusal is an error result: {}",
        replies[2]
    );
    assert!(
        replies[2].contains(&nobodys),
        "refused by name, so the name is in the refusal: {}",
        replies[2]
    );
    assert!(
        replies[2].contains(&format!("ank config --user corpora.{nobodys}")),
        "every refusal names the command that resolves it (§4): {}",
        replies[2]
    );
    assert!(
        !replies[2].contains("\"contract\""),
        "the refused call answered a document, so something ran: {}",
        replies[2]
    );

    // 4 and 5. Each corpus answers about itself, and about nothing else. This is
    // the aggregation ADR-621a7fd96ce1 permits: two readings presented one after
    // the other, never one corpus with two sources.
    assert!(
        replies[3].contains("The task of the second corpus")
            && !replies[3].contains("The task of the first corpus"),
        "the named corpus answered about the other one: {}",
        replies[3]
    );
    assert!(
        replies[4].contains("The task of the first corpus")
            && !replies[4].contains("The task of the second corpus"),
        "the startup corpus answered about the other one: {}",
        replies[4]
    );

    // The claim spaces, which are the refs and are per repository. A merged one
    // is what ADR-fd98f4bc6dea forbids in the same words the decision it
    // supersedes used, and this is where it would have had to leave a trace.
    assert_eq!(
        claimed_tasks(&one),
        vec![task_one.clone()],
        "the first corpus must hold its own claim and only its own"
    );
    assert_eq!(
        claimed_tasks(&two),
        vec![task_two.clone()],
        "the second corpus must hold its own claim and only its own"
    );

    let _ = std::fs::remove_dir_all(&one);
    let _ = std::fs::remove_dir_all(&two);
    let _ = std::fs::remove_dir_all(&home);
}

/// The startup corpus is reachable by its own name too, and a path is reachable
/// by no name at all.
///
/// **The set is "what the reader declared plus the startup corpus"**, so the
/// startup corpus has to answer to its identity and not only to the absence of
/// one -- otherwise a client looping over the corpora it can see would be able
/// to name every one of them except the one it is talking to.
///
/// **And a corpus is named by an identity, never by a path.** That is the clause
/// that keeps a declared set from becoming a merged one: a caller who could put
/// a path here would reach every corpus on the machine, which is `--repo` back
/// under another name. The refusal names the command that prints a real one.
#[test]
fn the_startup_corpus_answers_to_its_own_identity_and_a_path_answers_to_nothing() {
    let home = declaring_home();
    let repo = corpus_titled("The only task", Some(&home));
    let id = corpus_identity(&repo, &home);
    let path = repo.display().to_string().replace('\\', "/");

    let replies = talk_at(
        &repo,
        Some(&home),
        &[
            &format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"ank_find","arguments":{{"corpus":"{id}"}}}}}}"#
            ),
            &format!(
                r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"ank_find","arguments":{{"corpus":"{path}"}}}}}}"#
            ),
        ],
    );
    assert_eq!(replies.len(), 2, "{replies:?}");

    assert_eq!(
        field(&replies[0], "exitCode").as_deref(),
        Some("0"),
        "the corpus the server was addressed with is in the set it may reach, \
         and is reachable by its own identity: {}",
        replies[0]
    );
    assert!(
        replies[0].contains("The only task"),
        "and it answered about itself: {}",
        replies[0]
    );

    assert_eq!(
        field(&replies[1], "isError").as_deref(),
        Some("true"),
        "a path is not a corpus name: {}",
        replies[1]
    );
    assert!(
        replies[1].contains("is not a repository identity")
            && replies[1].contains("ank status --json"),
        "the refusal says what a name is and names the command that prints one: \
         {}",
        replies[1]
    );

    let _ = std::fs::remove_dir_all(&repo);
    let _ = std::fs::remove_dir_all(&home);
}

/// Every tool advertises the argument, asserted through the process.
///
/// The unit test in `crates/ank-mcp` asserts the same property of the generated
/// schema; this asserts it of the bytes that leave the server, which is the rule
/// CLAUDE.md states and this file's own header repeats: what a protocol promises
/// is what leaves the process. A schema that was right in the function and
/// absent from the reply would be exactly the failure the golden harness exists
/// to catch elsewhere.
#[test]
fn every_advertised_tool_carries_the_corpus_argument() {
    let repo = corpus();
    let replies = talk(
        &repo,
        &[r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#],
    );
    let listed = advertised(&replies[0]);
    assert_eq!(listed.len(), COMMANDS.len(), "{listed:?}");
    // One tool is one object, and every one of them has to carry the property.
    // Counting is the assertion: a subset would still contain the substring.
    assert_eq!(
        replies[0]
            .matches("\"corpus\":{\"type\":\"string\"")
            .count(),
        COMMANDS.len(),
        "the corpus argument is on every tool or it is a curated subset of the \
         corpora a client can reach per verb (ADR-fd98f4bc6dea): {}",
        replies[0]
    );
    let _ = std::fs::remove_dir_all(&repo);
}

/// The version a client is told is the version the binary prints, in both places
/// one leaves this surface (TASK-ae64d1c5678d).
///
/// **Read out of the process, never out of `CARGO_PKG_VERSION`.** A test that
/// compared the constant with itself would pass on the very day the surface went
/// back to reporting the library's number, which is the defect this closes: the
/// crate's version used to reach a client here, and the gate that held it to the
/// release tag went away with the second executable it was written for
/// (ADR-1ea31c2f3c5a). So `ank --version` is run, and what it says is the
/// expectation. CLAUDE.md states the rule this is an instance of.
///
/// **Two places and one number.** `serverInfo` is what a client reads at the
/// handshake; `ank-mcp/<version>` is the typed identity every claim taken
/// through this surface is written under, and it outlives the session in the
/// ref. They are asserted against the same value, because two numbers reaching a
/// reader are two numbers that can drift and only one of them would ever be
/// looked at again.
///
/// The claim is taken through the surface rather than read off a struct: the
/// identity is an environment variable handed to a spawned process, so what
/// proves it is a record the corpus wrote.
#[test]
fn the_surface_reports_the_version_the_binary_prints_and_writes_under_it() {
    let repo = corpus();
    let id = seeded_task(&repo);

    // `ank <version> (<commit>, skill <revision>)` -- the second word, and the
    // only part of that line this is about.
    let printed = ank(&repo, &["--version"]);
    assert!(
        printed.status.success(),
        "ank --version must answer: {}",
        String::from_utf8_lossy(&printed.stderr)
    );
    let line = String::from_utf8_lossy(&printed.stdout).trim().to_string();
    let version = line
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_string();
    assert!(
        !version.is_empty() && version.chars().next().is_some_and(|c| c.is_ascii_digit()),
        "the shape of `ank --version` moved and this test is reading the wrong \
         word of it: {line}"
    );

    let claim = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"ank_claim","arguments":{{"arguments":["{id}"]}}}}}}"#
    );
    let replies = talk_unnamed(
        &repo,
        &[
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{}}}"#,
            &claim,
        ],
    );
    assert_eq!(replies.len(), 2, "{replies:?}");

    assert_eq!(
        server_info_version(&replies[0]).as_deref(),
        Some(version.as_str()),
        "a client is told a version the binary does not answer with. The number \
         in serverInfo is the executable's, handed down by the dispatch, and the \
         release gates that one against the tag: {}",
        replies[0]
    );

    assert_eq!(
        field(&replies[1], "exitCode").as_deref(),
        Some("0"),
        "the claim must have been taken for its holder to say anything: {}",
        replies[1]
    );
    assert!(
        replies[1].contains(&escaped(&format!(r#""holder":"ank-mcp/{version}""#))),
        "the claim was written under an identity carrying a version the binary \
         does not answer with. It is the same number serverInfo reports, and it \
         is in a ref that outlives this session: {}",
        replies[1]
    );

    let _ = std::fs::remove_dir_all(&repo);
}

/// Every git repository inside a fixture, found rather than listed.
///
/// A directory holding a `HEAD` file and an `objects` directory is one,
/// whether it is the `.git` beside a working tree or a bare corpus. Found,
/// because a list would have to be maintained, and what is being guarded
/// against is exactly a repository nobody remembered to enrol.
fn repositories_under(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if dir.join("HEAD").is_file() && dir.join("objects").is_dir() {
            found.push(dir);
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(e.path());
            }
        }
    }
    found
}

/// What git answers for one key of one repository's own configuration, `None`
/// when the key is unset -- which is the state this asserts against, since an
/// unset `maintenance.auto` means maintenance is on.
///
/// `--local`, so what comes back is the fixture's answer and never the
/// machine's: a contributor carrying `gc.auto` in a global configuration would
/// otherwise read a pass out of a repository that sets nothing.
fn config_of(git_dir: &std::path::Path, key: &str) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(git_dir)
        .args(["config", "--local", "--get", key])
        .output()
        .expect("git must be installed: it is a hard dependency");
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Asserts that every repository under `root` is one git will not maintain.
///
/// Read back out of a freshly built fixture rather than grepped out of this
/// file: what is under test is the configuration `git init` was actually
/// followed by, and a grep passes on a comment and fails on a refactor.
fn assert_unmaintained(root: &std::path::Path) {
    let repos = repositories_under(root);
    assert!(
        !repos.is_empty(),
        "no repository found under {}: this asserts nothing",
        root.display()
    );
    for git_dir in repos {
        let at = git_dir.display();
        assert_eq!(
            config_of(&git_dir, "gc.auto").as_deref(),
            Some("0"),
            "gc.auto at {at}"
        );
        assert_eq!(
            config_of(&git_dir, "maintenance.auto").as_deref(),
            Some("false"),
            "maintenance.auto at {at}"
        );
    }
}

/// A fixture repository is not maintained under the test.
///
/// Measured on 2026-08-30 in run 33284185681: git repacked a fixture between
/// two fingerprints of it -- `objects/maintenance.lock`, a `tmp_pack` and six
/// loose objects in the first, a multi-pack-index, two packs and `info/refs`
/// in the second -- and a test asserting that a read writes nothing failed on
/// one platform of three. Ank had written nothing (TASK-fc6bef21e268).
///
/// The repositories are found by walking the fixture and not named here, so a
/// second one grown under it later is held to this without anyone remembering
/// to enrol it.
#[test]
fn a_fixture_repository_is_not_maintained_under_the_test() {
    let repo = corpus();
    assert_unmaintained(&repo);
    let _ = std::fs::remove_dir_all(&repo);
}
