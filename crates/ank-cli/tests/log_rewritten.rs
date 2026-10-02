//! `check` signals a log entry the default branch holds and this checkout has
//! changed (TASK-0275c2ea648c, under ADR-4004eb9be5e9).
//!
//! The case that motivated it is PR #523: the `author` of LOG-090ad90eb071 was
//! rewritten by hand on a branch, and `check` exited 0 saying nothing. An entry
//! is written once (ADR-25f977377fa0); a correction is a new entry naming it.
//!
//! **A signal and never a fault**, because a format migration rewrites entries
//! on purpose, so every case below that finds one still exits 0.
//!
//! **Through the binary**, every case. Process counts are read from
//! `GIT_TRACE` at an absolute path and never from a wall clock (ADR-cc65f1388a71).

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@log-rewritten";

/// What git prefixes the argument list of every git it runs, under `GIT_TRACE`.
const MARK: &str = "trace: built-in: git ";

/// The words every rewritten-entry signal carries, and no other finding does.
const SAID: &str = "the default branch holds";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("log-rewritten-gitconfig");
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

fn spawn(program: &str) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    c.env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env_remove("GIT_TRACE")
        .env_remove("EDITOR")
        .env_remove("VISUAL")
        .env("NO_COLOR", "1")
        .env("ANK_AGENT", AGENT);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

struct Repo {
    dir: PathBuf,
    /// The entries main holds, in the order they were written.
    entries: Vec<String>,
}

impl Repo {
    /// A repository whose `main` holds `n` log entries, checked out on a branch
    /// cut from it. `default_branch` is written to the config only when asked:
    /// the indeterminable case is the same repository without it.
    fn new(what: &str, n: usize, default_branch: bool) -> Repo {
        let dir = scratch::dir(what);
        fs::create_dir_all(dir.join(".ank/entities")).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        let mut r = Repo {
            dir,
            entries: Vec::new(),
        };
        r.git(&["init", "-q", "-b", "main"]);
        let config = if default_branch {
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n"
        } else {
            "schema: 1\nclaim_ttl_max: 2h\n"
        };
        fs::write(r.dir.join(".ank/config.yml"), config).unwrap();
        fs::write(r.dir.join(".gitignore"), ".ank/index.db*\n").unwrap();
        r.commit("seed");
        let task = r
            .ok(&[
                "new",
                "task",
                "--title",
                "Work that gets logged",
                "--scope",
                "src/**",
                "--criteria",
                "Something holds.",
                "--no-verify",
            ])
            .split_whitespace()
            .nth(1)
            .expect("created <id> <slug>")
            .to_string();
        r.commit("plan");
        r.ok(&["claim", &task]);
        for i in 0..n {
            r.entries.push(r.log(&format!("entry number {i}")));
        }
        r.commit("entries");
        r.git(&["checkout", "-q", "-b", "work"]);
        r
    }

    fn git(&self, args: &[&str]) -> String {
        let out = spawn("git")
            .args(args)
            .current_dir(&self.dir)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
        text(&out.stdout)
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", message, "--allow-empty"]);
    }

    fn ank(&self, args: &[&str], trace: Option<&Path>) -> Output {
        let mut c = spawn(ANK);
        c.args(args)
            .arg("--repo")
            .arg(&self.dir)
            .current_dir(scratch::root());
        if let Some(t) = trace {
            c.env("GIT_TRACE", t);
        }
        c.output().unwrap()
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.ank(args, None);
        assert_eq!(
            out.status.code(),
            Some(0),
            "ank {args:?}: {}{}",
            text(&out.stdout),
            text(&out.stderr)
        );
        text(&out.stdout)
    }

    /// An entry about the claimed task, and the id the corpus gave it.
    fn log(&self, message: &str) -> String {
        let before = self.logs();
        self.ok(&["log", message]);
        let after = self.logs();
        let new: Vec<&String> = after.iter().filter(|id| !before.contains(id)).collect();
        assert_eq!(new.len(), 1, "one entry per ank log: {new:?}");
        new[0].clone()
    }

    fn logs(&self) -> Vec<String> {
        let mut ids: Vec<String> = fs::read_dir(self.dir.join(".ank/entities"))
            .unwrap()
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.strip_prefix("LOG-")
                    .and_then(|rest| rest.strip_suffix(".md"))
                    .map(|stem| format!("LOG-{stem}"))
            })
            .collect();
        ids.sort();
        ids
    }

    fn file(&self, id: &str) -> PathBuf {
        self.dir.join(format!(".ank/entities/{id}.md"))
    }

    /// The hand edit of PR #523: one field of a written entry, rewritten.
    fn rewrite(&self, id: &str) {
        let path = self.file(id);
        let before = fs::read_to_string(&path).unwrap();
        let after = before.replace(&format!("author: {AGENT}"), "author: human:somebody-else");
        assert_ne!(
            before, after,
            "the entry carries the author it is written with"
        );
        fs::write(&path, after).unwrap();
    }

    /// `check`, its exit code and its standard output.
    fn check(&self) -> (Option<i32>, String) {
        let out = self.ank(&["check"], None);
        (out.status.code(), text(&out.stdout) + &text(&out.stderr))
    }

    /// `check` under `GIT_TRACE`; the git command lines it started.
    fn traced_check(&self, name: &str) -> Vec<String> {
        let trace = self.dir.with_extension(format!("{name}.trace"));
        let _ = fs::remove_file(&trace);
        let out = self.ank(&["check"], Some(&trace));
        assert_eq!(out.status.code(), Some(0), "check: {}", text(&out.stdout));
        fs::read_to_string(&trace)
            .unwrap_or_default()
            .lines()
            .filter_map(|l| l.split_once(MARK).map(|(_, args)| args.to_string()))
            .collect()
    }
}

/// The lines of `check` naming a rewritten entry.
fn rewritten(output: &str) -> Vec<&str> {
    output.lines().filter(|l| l.contains(SAID)).collect()
}

#[test]
fn a_corpus_with_no_entry_changed_reports_nothing_of_the_kind() {
    let r = Repo::new("log-rewritten-none", 3, true);
    let (code, out) = r.check();
    assert_eq!(code, Some(0), "{out}");
    assert!(rewritten(&out).is_empty(), "{out}");
}

#[test]
fn an_entry_edited_in_the_working_tree_is_a_signal_naming_it_and_the_correction() {
    let r = Repo::new("log-rewritten-tree", 3, true);
    let x = r.entries[1].clone();
    r.rewrite(&x);
    let (code, out) = r.check();
    assert_eq!(code, Some(0), "a signal, never a fault: {out}");
    let lines = rewritten(&out);
    assert_eq!(lines.len(), 1, "one line for the one entry: {out}");
    let line = lines[0];
    assert!(line.contains(&x), "names the entry: {line}");
    assert!(line.contains("restore it"), "names the restore: {line}");
    assert!(
        line.contains("write a new entry naming it"),
        "names the correction ADR-25f977377fa0 prescribes: {line}"
    );
    for other in [&r.entries[0], &r.entries[2]] {
        assert!(!line.contains(other.as_str()), "an untouched entry: {line}");
    }
}

#[test]
fn an_entry_edited_in_a_commit_on_the_branch_reports_the_same() {
    let r = Repo::new("log-rewritten-commit", 3, true);
    let x = r.entries[0].clone();
    r.rewrite(&x);
    r.commit("rewrite an entry");
    let (code, out) = r.check();
    assert_eq!(code, Some(0), "{out}");
    let lines = rewritten(&out);
    assert_eq!(lines.len(), 1, "{out}");
    assert!(lines[0].contains(&x), "{out}");
    assert!(lines[0].contains("write a new entry naming it"), "{out}");
}

#[test]
fn an_entry_created_on_the_branch_is_new_and_not_reported() {
    let r = Repo::new("log-rewritten-new", 2, true);
    let fresh = r.log("written on the branch");
    r.commit("an entry of the branch's own");
    // And corrected before it lands, which is the window where that is cheap.
    r.rewrite(&fresh);
    let (code, out) = r.check();
    assert_eq!(code, Some(0), "{out}");
    assert!(rewritten(&out).is_empty(), "{out}");
}

#[test]
fn the_git_process_count_is_the_same_for_one_and_for_forty_entries_changed() {
    let r = Repo::new("log-rewritten-cost", 40, true);
    let level = r.traced_check("level");

    r.rewrite(&r.entries[7].clone());
    let one = r.traced_check("one");
    let (_, out) = r.check();
    assert_eq!(rewritten(&out).len(), 1, "{out}");

    for id in r.entries.clone() {
        if id != r.entries[7] {
            r.rewrite(&id);
        }
    }
    let forty = r.traced_check("forty");
    let (_, out) = r.check();
    assert_eq!(rewritten(&out).len(), 40, "{out}");

    eprintln!(
        "git processes started by check: level {}, 1 changed {}, 40 changed {}",
        level.len(),
        one.len(),
        forty.len()
    );
    assert!(!one.is_empty(), "the trace was written");
    assert_eq!(
        one.len(),
        forty.len(),
        "one process per entry would grow the count:\n1: {one:#?}\n40: {forty:#?}"
    );
}

#[test]
fn with_no_default_branch_the_comparison_is_skipped_in_one_line_without_a_fault() {
    let r = Repo::new("log-rewritten-nobranch", 2, false);
    r.rewrite(&r.entries[0].clone());
    let (code, out) = r.check();
    assert_eq!(code, Some(0), "skipped, not a fault: {out}");
    assert!(rewritten(&out).is_empty(), "{out}");
    let skipped: Vec<&str> = out
        .lines()
        .filter(|l| l.contains("log entries") && l.contains("not compared"))
        .collect();
    assert_eq!(skipped.len(), 1, "exactly one line says so: {out}");
}
