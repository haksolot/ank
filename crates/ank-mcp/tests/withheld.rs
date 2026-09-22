//! A flag this server keeps is refused with the reason it keeps *that* flag,
//! driven through `ank mcp` over stdio (TASK-308ce062f427).
//!
//! **Through the process, because the claim is about the process.** The
//! criterion says what a `tools/call` comes back with, and what a `tools/call`
//! comes back with is a line on a pipe. `tools::withheld` is unit-tested beside
//! itself in `src/tools.rs` and that proves the function; this proves the
//! surface, which is what a client without a shell talks to. CLAUDE.md states
//! the rule and this workspace has paid for it twice.
//!
//! **Why the binary is found rather than named.** `CARGO_BIN_EXE_ank` is
//! defined only for the package that declares the binary, and that is
//! `ank-cli`. So it is looked for beside the test executable, where cargo puts
//! it, exactly as `crates/ank-tui/tests/terminal/mod.rs` does for the same
//! reason -- and for the same reason neither crate may simply link the
//! dispatch: `tests/dependencies.rs` forbids this one `ank-cli` outright
//! (ADR-fd98f4bc6dea), and a `Command` in a test is not a link.
//!
//! The fixture is a corpus rather than a bare directory because the server
//! refuses to start without one: `ank mcp --repo <a directory with no .ank/>`
//! exits 1 with `no .ank/ found`, before any request is read.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// The `ank` this suite drives, built before it is driven.
///
/// Cargo puts an integration test in `<target>/<profile>/deps/` and a binary in
/// `<target>/<profile>/`, so the binary is one or two pops away.
///
/// **And it is built here rather than only looked for, which is not
/// belt-and-braces.** Measured on 2026-09-20 while this suite was being written
/// red: with the fix reverted in `src/lib.rs`, `cargo test -p ank-mcp --test
/// withheld` passed 2 of 2 -- that invocation builds this crate and never
/// `ank-cli`'s binary, so the suite drove the `ank` left in `target/` from
/// before the revert and reported on a process that no longer existed in the
/// tree. `cargo build -p ank-cli --bin ank` and the same command then failed at
/// the assertion it was written for. A suite whose subject is a file somebody
/// else's command produces is green over a stale one by default, and green over
/// a stale one is the failure mode a regression test exists to not have.
///
/// `--offline` because a test must not reach the network: the lockfile is
/// complete by the time anything here is compiled (`tests/dependencies.rs` runs
/// `cargo tree` on the same terms). Under `cargo test --workspace`, which is
/// what `verifiers.cargo-test.run` declares, the binary is already current and
/// this is a no-op that takes the build lock and gives it back.
fn ank() -> PathBuf {
    static BUILT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    BUILT
        .get_or_init(|| {
            let mut at = std::env::current_exe().expect("a test executable has a path");
            at.pop();
            if at.file_name().is_some_and(|n| n == "deps") {
                at.pop();
            }
            let profile = at
                .file_name()
                .and_then(|n| n.to_str())
                .expect("a target directory is named after its profile")
                .to_string();
            let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
            let mut build = Command::new(cargo);
            build.args(["build", "-p", "ank-cli", "--bin", "ank", "--offline"]);
            // The profile is read off the path rather than assumed: this suite is
            // run under `--release` by nothing today and would be silently driving
            // the debug binary if it ever were.
            if profile == "release" {
                build.arg("--release");
            }
            let out = build
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .output()
                .expect("cargo must be runnable: it is what built this test");
            assert!(
                out.status.success(),
                "cargo could not build the binary this suite drives:\n{}",
                String::from_utf8_lossy(&out.stderr)
            );
            let binary = at.join(if cfg!(windows) { "ank.exe" } else { "ank" });
            assert!(
                binary.is_file(),
                "cargo reported success and there is no binary at {}",
                binary.display()
            );
            binary
        })
        .clone()
}

/// A corpus nothing else uses, made through the binary rather than by writing
/// files into `.ank/`, and removed when the test ends.
struct Corpus(PathBuf);

impl Drop for Corpus {
    /// Stated over this run's own root and never over whatever path the field
    /// happens to hold, for the reason `crates/ank-tui/tests/terminal/mod.rs`
    /// records: a `Drop` that removes what it is handed removes a working tree
    /// the first time somebody hands it one.
    fn drop(&mut self) {
        if self.0.starts_with(scratch::root()) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

impl Corpus {
    fn new() -> Corpus {
        let corpus = Corpus(scratch::dir("withheld"));
        corpus.git(&["init", "--initial-branch=main"]);
        corpus.git(&["config", "user.email", "suite@example.invalid"]);
        corpus.git(&["config", "user.name", "The Suite"]);
        // A throwaway corpus inherits whatever the machine has globally
        // otherwise, and a suite whose commits are signed on one machine and
        // not the next is a suite that reports the machine.
        corpus.git(&["config", "commit.gpgsign", "false"]);
        // A repository git repacks behind the suite is a corpus that moves
        // under whatever reads it (TASK-fc6bef21e268).
        corpus.git(&["config", "gc.auto", "0"]);
        corpus.git(&["config", "maintenance.auto", "false"]);
        std::fs::create_dir_all(corpus.0.join("src")).unwrap();
        std::fs::write(corpus.0.join("src/lib.rs"), "// code\n").unwrap();
        corpus.ank(&["init"]);
        corpus.git(&["add", "-A"]);
        corpus.git(&["commit", "-qm", "a corpus"]);
        corpus
    }

    fn git(&self, args: &[&str]) {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("git is a hard dependency of this repository");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn ank(&self, args: &[&str]) {
        let mut cmd = Command::new(ank());
        cmd.args(args)
            .current_dir(&self.0)
            .env("ANK_AGENT", "suite@example.invalid");
        in_this_runs_root(&mut cmd);
        let out = cmd.output().expect("the binary must have been built");
        assert!(
            out.status.success(),
            "ank {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// Sends every request to one `ank mcp` over stdio, closes stdin, and
    /// returns the reply lines in order.
    fn talk(&self, requests: &[&str]) -> Vec<String> {
        let mut cmd = Command::new(ank());
        cmd.arg("mcp")
            .arg("--repo")
            .arg(&self.0)
            .env("ANK_AGENT", "suite@example.invalid")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        in_this_runs_root(&mut cmd);
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
            .map(str::to_string)
            .collect()
    }
}

/// Tell a child of this suite that "temporary" means this run's own root
/// (TASK-ec85b1561855). All three names, because `std::env::temp_dir` reads
/// `TMP` and `TEMP` on Windows and `TMPDIR` everywhere else.
fn in_this_runs_root(command: &mut Command) {
    let root = scratch::root();
    command
        .env("TMPDIR", root)
        .env("TMP", root)
        .env("TEMP", root);
}

/// One `tools/call` on `ank_find`, carrying one withheld flag.
fn call(id: u32, key: &str, value: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"ank_find","arguments":{{"{key}":{value}}}}}}}"#
    )
}

/// The three flags this server keeps, each refused with its own reason.
///
/// **The assertion is that the three answers differ**, and then that each one
/// is about its own flag. Measured on 2026-09-20, before the fix: all three
/// came back `"<flag> belongs to the server: name a corpus with the corpus
/// argument, by the identity ank status --json prints, never by a path"` --
/// `--repo`'s reason, handed to a caller that had asked about `--json` and to
/// one that had asked about `--quiet`. Each was refused, so a test that only
/// looked for a refusal was green over it.
#[test]
fn json_quiet_and_repo_are_each_refused_with_their_own_reason() {
    let corpus = Corpus::new();
    let replies = corpus.talk(&[
        &call(1, "json", "true"),
        &call(2, "quiet", "true"),
        &call(3, "repo", "\"/somewhere/else\""),
    ]);
    assert_eq!(replies.len(), 3, "{replies:?}");

    for (reply, flag) in replies.iter().zip(["--json", "--quiet", "--repo"]) {
        assert!(
            reply.contains(&format!("{flag} belongs to the server")),
            "the refusal must name the flag it is about: {reply}"
        );
    }

    // `--repo` keeps the message it has always had, which is the half of the
    // criterion that is about not changing anything.
    assert!(
        replies[2].contains("name a corpus with the corpus argument")
            && replies[2].contains("never by a path"),
        "--repo's reason is the one it had: {}",
        replies[2]
    );

    // And the other two no longer carry it. This is the defect, stated as the
    // one sentence that must not appear where it does not belong.
    for reply in &replies[..2] {
        assert!(
            !reply.contains("name a corpus"),
            "a caller that did not ask about a corpus is told to name one, \
             which is --repo's reason on another flag's refusal: {reply}"
        );
        assert!(
            !reply.contains("never by a path"),
            "a caller that passed no path is told not to pass one: {reply}"
        );
    }

    // Three flags, three sentences: the fix is not one new sentence shared by
    // the two that were wrong.
    let reasons: std::collections::BTreeSet<&String> = replies.iter().collect();
    assert_eq!(
        reasons.len(),
        3,
        "two of the three refusals are the same sentence: {replies:?}"
    );
}

/// `--worktree` is the server's too, refused in the shape the other three are
/// and with a reason of its own (TASK-7cb77bc870b1).
///
/// Measured on 2026-09-20, before the fix: `{"worktree":"/tmp"}` on `ank_find`
/// ran and came back with a document, and `{"worktree":"/no/such/dir"}` on
/// `ank_status` reached the CLI and was refused there -- the caller's path was
/// written into the address (ADR-9e56318631f3) the server holds, and a
/// verifier `ank done` runs has its working directory in that tree.
///
/// The three already withheld are asked again in the same session, because
/// the other half of the criterion is that they keep the reasons they have:
/// four flags, four sentences, none of them the new one.
#[test]
fn worktree_is_refused_by_name_and_the_three_keep_their_reasons() {
    let corpus = Corpus::new();
    let replies = corpus.talk(&[
        &call(1, "worktree", "\"/somewhere/else\""),
        &call(2, "json", "true"),
        &call(3, "quiet", "true"),
        &call(4, "repo", "\"/somewhere/else\""),
    ]);
    assert_eq!(replies.len(), 4, "{replies:?}");

    let worktree = &replies[0];
    assert!(
        worktree.contains(r#""id":1"#) && worktree.contains(r#""code":-32602"#),
        "--worktree is refused as the other withheld flags are, an invalid \
         parameter on the caller's own id: {worktree}"
    );
    assert!(
        !worktree.contains(r#""result""#),
        "the call ran with the caller's work tree: {worktree}"
    );
    assert!(
        worktree.contains("--worktree belongs to the server"),
        "the refusal must name the flag it is about: {worktree}"
    );

    for (reply, flag) in replies[1..].iter().zip(["--json", "--quiet", "--repo"]) {
        assert!(
            reply.contains(&format!("{flag} belongs to the server")),
            "the refusal must name the flag it is about: {reply}"
        );
    }
    assert!(
        replies[3].contains("name a corpus with the corpus argument")
            && replies[3].contains("never by a path"),
        "--repo's reason is the one it had: {}",
        replies[3]
    );

    let reasons: std::collections::BTreeSet<&String> = replies.iter().collect();
    assert_eq!(
        reasons.len(),
        4,
        "two of the four refusals are the same sentence: {replies:?}"
    );
}

/// No tool advertises `worktree`: the schema hides exactly what the refusal
/// has a reason for, so a client is never offered an argument it would be
/// refused for passing.
#[test]
fn no_tool_advertises_worktree() {
    let corpus = Corpus::new();
    let replies = corpus.talk(&[r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#]);
    assert_eq!(replies.len(), 1, "{replies:?}");
    assert!(
        replies[0].contains(r#""ank_find""#),
        "the list came back without the tools: {}",
        replies[0]
    );
    assert!(
        !replies[0].contains(r#""worktree""#),
        "a tool still advertises the flag every call refuses"
    );
}

/// The refusal reaches the client as a JSON-RPC error on the request's own id,
/// and never as a call that quietly ran without the flag.
///
/// The second half is what makes the first worth asserting: a surface that
/// dropped `"json": true` and ran `ank find` would answer `result` with a
/// document, and the caller would never learn that what it asked for was not
/// what it got.
#[test]
fn a_withheld_flag_is_an_error_on_the_callers_own_id_and_never_a_silent_run() {
    let corpus = Corpus::new();
    let replies = corpus.talk(&[&call(7, "quiet", "true")]);
    assert_eq!(replies.len(), 1, "{replies:?}");
    let reply = &replies[0];
    assert!(
        reply.contains(r#""id":7"#),
        "a client matches replies by identity: {reply}"
    );
    assert!(
        reply.contains(r#""code":-32602"#),
        "a flag that is not the caller's is an invalid parameter: {reply}"
    );
    assert!(
        !reply.contains(r#""result""#),
        "the call ran anyway, with the flag dropped: {reply}"
    );
}

/// One directory per test process, and the next run sweeps what a killed one
/// left (TASK-553740e7af11).
///
/// The reasoning is written out once, where the original lives, in
/// `crates/ank-cli/tests/scratch/mod.rs`. In short: a `Drop` cannot run on
/// `SIGKILL`, so the run that cleans up is the next one, and a root's `.owner`
/// lock is free exactly when its owner is gone.
mod scratch {
    use std::fs::{self, File};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::OnceLock;

    const PREFIX: &str = "ank-it-";
    const OWNER: &str = ".owner";
    static HELD: OnceLock<File> = OnceLock::new();

    pub fn dir(what: &str) -> PathBuf {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let p = root().join(format!("{what}-{}", SEQ.fetch_add(1, Ordering::Relaxed)));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).expect("the root must be writable");
        p
    }

    pub fn root() -> &'static Path {
        static ROOT: OnceLock<PathBuf> = OnceLock::new();
        ROOT.get_or_init(|| {
            let base = std::env::temp_dir();
            let mine = base.join(format!("{PREFIX}{}", std::process::id()));
            let _ = fs::remove_dir_all(&mine);
            fs::create_dir_all(&mine).expect("the temporary directory must be writable");
            let lock = File::create(mine.join(OWNER)).expect("the root takes its own lock");
            lock.try_lock()
                .expect("a fresh root cannot already be held");
            let _ = HELD.set(lock);
            sweep(&base, &mine);
            mine
        })
        .as_path()
    }

    fn sweep(base: &Path, mine: &Path) {
        let Ok(entries) = fs::read_dir(base) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let named = p
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(PREFIX));
            if p == mine || !named || !p.is_dir() {
                continue;
            }
            let Ok(file) = File::options().read(true).write(true).open(p.join(OWNER)) else {
                continue;
            };
            if file.try_lock().is_ok() {
                let _ = file.unlock();
                drop(file);
                let _ = fs::remove_dir_all(&p);
            }
        }
    }
}
