//! What `ank init` writes into `.gitignore` covers the index's WAL siblings,
//! and a repository carrying the previous line is brought to the new one
//! (TASK-574dee03ba56).
//!
//! Measured on 2026-09-22: `ank find` killed with `SIGKILL` mid-run left
//! `.ank/index.db-wal` and `.ank/index.db-shm`, and `git status --porcelain
//! -uall` offered both, because the line `init` wrote was the literal
//! `.ank/index.db`. Killing a process at the right microsecond is not a test
//! anyone can rerun, so the siblings are written here the way the kill leaves
//! them: present, beside the database, with nothing holding them open. What is
//! under test is git's verdict on them, and that is the same either way.
//!
//! **Through the binary, and read back from git.** The line is the one the
//! binary wrote, and the verdict is `git status`, not a string comparison
//! against the constant: a glob that git reads differently from how it looks
//! would pass the second and fail the first.

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// Every file SQLite may leave beside the database after a crash.
const SIBLINGS: [&str; 3] = ["index.db-wal", "index.db-shm", "index.db-journal"];

fn spawn(program: &str, dir: &Path) -> Command {
    let mut c = Command::new(program);
    c.current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
        .env("ANK_AGENT", "wal@fixture");
    c
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = spawn("git", dir).args(args).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn ank_init(dir: &Path) {
    let out = spawn(ANK, dir).arg("init").output().unwrap();
    assert!(out.status.success(), "ank init: {out:?}");
}

fn repository(what: &str) -> PathBuf {
    let repo = scratch::dir(what);
    git(&repo, &["init", "-q", "."]);
    git(&repo, &["config", "commit.gpgsign", "false"]);
    repo
}

/// What `git status` offers under `.ank/`, after the corpus is committed and
/// every sibling a crash leaves has been written beside the database.
fn offered_after_a_crash(repo: &Path) -> Vec<String> {
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-q", "-m", "init"]);
    for s in SIBLINGS {
        fs::write(repo.join(".ank").join(s), b"left by a killed verb").unwrap();
    }
    git(repo, &["status", "--porcelain", "-uall"])
        .lines()
        .filter(|l| l.contains(".ank/"))
        .map(str::to_string)
        .collect()
}

#[test]
fn a_fresh_init_ignores_the_index_siblings() {
    let repo = repository("wal-fresh");
    ank_init(&repo);
    let offered = offered_after_a_crash(&repo);
    assert!(offered.is_empty(), "offered for commit: {offered:?}");
}

#[test]
fn an_old_ignore_line_is_brought_to_the_new_one() {
    let repo = repository("wal-legacy");
    fs::write(
        repo.join(".gitignore"),
        "/target\n.ank/index.db\nnode_modules/\n",
    )
    .unwrap();
    ank_init(&repo);

    let gi = fs::read_to_string(repo.join(".gitignore")).unwrap();
    let ours: Vec<&str> = gi
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with(".ank/index.db"))
        .collect();
    assert_eq!(ours.len(), 1, "both halves left in: {gi:?}");
    assert!(
        gi.starts_with("/target\n") && gi.contains("node_modules/\n"),
        "the user's own rules were not kept: {gi:?}"
    );

    let offered = offered_after_a_crash(&repo);
    assert!(offered.is_empty(), "offered for commit: {offered:?}");

    // A second init has nothing left to bring up to date.
    ank_init(&repo);
    assert_eq!(fs::read_to_string(repo.join(".gitignore")).unwrap(), gi);
}
