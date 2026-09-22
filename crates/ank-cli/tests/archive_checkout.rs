//! `find --all` lists the archive after a checkout took it away and brought it
//! back (TASK-ef4dac167955).
//!
//! The archive is committed on one branch and absent on the other, where the
//! same entity is hot. A read there that does not ask for the archive indexed
//! the hot copy, which moved the entity's one row off its archived path and
//! left the `files` row of that path where it was, unseen by an open that does
//! not walk the archive. Back on the first branch the archived file matched
//! that stale `files` row, counted as unchanged and was never reindexed, and
//! the removal of the hot path deleted the only row the entity had: measured
//! before the fix, `total 6 archived 1` became `total 5 archived 0`, and a
//! deleted `index.db` gave `6` and `1` again.
//!
//! Through the binary, because the criterion is about what the binary lists,
//! and across two real checkouts, because the defect lives in the index a tree
//! keeps from one of them to the next.

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@archive-checkout";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("archive-checkout-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n[core]\n\tautocrlf = false\n",
        )
        .unwrap();
        p
    })
    .as_path()
}

fn spawn(program: &str, dir: &Path) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    c.env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", AGENT)
        .current_dir(dir);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn run(program: &str, dir: &Path, args: &[&str]) -> Output {
    let out = spawn(program, dir)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("{program}: {e}"));
    assert!(
        out.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn git(dir: &Path, args: &[&str]) -> String {
    String::from_utf8_lossy(&run("git", dir, args).stdout)
        .trim()
        .to_string()
}

fn ank(dir: &Path, args: &[&str]) -> String {
    String::from_utf8_lossy(&run(ANK, dir, args).stdout).to_string()
}

/// `(total, archived)` as `find --all --json` states them: the total it
/// declares, and the results that carry `"archived":true`.
fn listed(dir: &Path) -> (u64, usize) {
    let out = ank(dir, &["find", "--all", "--json"]);
    let total = out
        .split("\"total\":")
        .nth(1)
        .and_then(|rest| {
            rest.split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or_else(|| panic!("find --json states no total: {out}"));
    (total, out.matches("\"archived\":true").count())
}

#[test]
fn find_all_lists_the_archive_after_a_checkout_took_it_away_and_brought_it_back() {
    let dir = scratch::dir("archive-checkout");
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["commit", "-q", "--allow-empty", "-m", "init"]);
    ank(&dir, &["init"]);
    let mut ids = Vec::new();
    for title in ["cold one", "cold two", "hot"] {
        let out = ank(
            &dir,
            &[
                "new",
                "task",
                "--title",
                title,
                "--scope",
                "src/**",
                "--criteria",
                "a criterion",
                "--no-verify",
            ],
        );
        let id = out
            .split_whitespace()
            .find(|w| w.starts_with("TASK-"))
            .unwrap_or_else(|| panic!("new task names no id: {out}"))
            .to_string();
        ids.push(id);
    }
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "seed"]);

    // The archive, committed on a branch of its own; `main` keeps both hot.
    git(&dir, &["switch", "-q", "-c", "archived"]);
    let cold = dir.join(".ank/archive/entities");
    fs::create_dir_all(&cold).unwrap();
    for id in &ids[..2] {
        let name = format!("{id}.md");
        git(
            &dir,
            &[
                "mv",
                &format!(".ank/entities/{name}"),
                &format!(".ank/archive/entities/{name}"),
            ],
        );
    }
    git(&dir, &["commit", "-q", "-m", "archive"]);

    let before = listed(&dir);
    assert_eq!(before.1, 2, "the archive is listed before any checkout");

    // A read on the other branch that does not ask for the archive, then back.
    git(&dir, &["switch", "-q", "main"]);
    ank(&dir, &["find", "cold"]);
    git(&dir, &["switch", "-q", "archived"]);
    let after = listed(&dir);

    // What a fresh index gives, from the same tree.
    for entry in fs::read_dir(dir.join(".ank")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if name.starts_with("index.db") {
            fs::remove_file(&path).unwrap();
        }
    }
    let fresh = listed(&dir);
    assert_eq!(fresh, before, "a fresh index answers as the first did");
    assert_eq!(
        after, fresh,
        "(total, archived) after the round trip against a fresh index"
    );
}
