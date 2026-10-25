//! An ADR scope no commit ever carried is awaited while an open task covers it,
//! and owed once that task ends (TASK-afc96307cb81, ADR-91f486072d68).
//!
//! Every case through the binary, on a proposed ADR and on an accepted one,
//! each scoping `src/new.rs`, a path the fixture never commits until the case
//! that commits it. What each case varies is the task: none, an open one whose
//! scope is that path or `src/**`, one gone done, one gone closed. The death
//! git can name, a deletion, is pinned to the words ADR-3094538d831e gives it,
//! so a rule that lowered more than it was given would turn it red.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "human:marie";
const NEW: &str = "src/new.rs";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("dead-scope-awaited-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n\
             [user]\n\tname = t\n\temail = t@example.com\n\tsigningkey =\n\
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
        .env_remove("EDITOR")
        .env_remove("VISUAL")
        .env_remove("GIT_TRACE")
        .env("ANK_AGENT", AGENT);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

/// One finding `check` printed about the ADR, with the kind it was filed under.
#[derive(Debug, PartialEq)]
struct Finding {
    level: String,
    kind: String,
    message: String,
    note: Vec<String>,
}

fn finding(level: &str, kind: &str, message: &str, note: &[&str]) -> Finding {
    Finding {
        level: level.into(),
        kind: kind.into(),
        message: message.into(),
        note: note.iter().map(|n| n.to_string()).collect(),
    }
}

/// The findings `check --kind <kind>` printed about `id`: a
/// `level: subject: message` line, and the note hung beneath it.
fn findings(printed: &str, id: &str, kind: &str) -> Vec<Finding> {
    let mut found: Vec<Finding> = Vec::new();
    let mut ours = false;
    for line in printed.lines() {
        if let Some(note) = line
            .strip_prefix("└── ")
            .or_else(|| line.strip_prefix("    "))
        {
            if ours {
                found.last_mut().unwrap().note.push(note.to_string());
            }
            continue;
        }
        let mut parts = line.splitn(3, ": ");
        let (Some(level), Some(subject), Some(message)) =
            (parts.next(), parts.next(), parts.next())
        else {
            ours = false;
            continue;
        };
        // A fault is printed as `error`, as a compiler prints one.
        let level = match level {
            "error" => "fault",
            other => other,
        };
        ours = subject == id && matches!(level, "signal" | "fault");
        if ours {
            found.push(finding(level, kind, message, &[]));
        }
    }
    found
}

struct Repo(PathBuf);

impl Repo {
    fn new(what: &str) -> Repo {
        let dir = scratch::dir(what);
        fs::create_dir_all(dir.join(".ank/entities")).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        let r = Repo(dir);
        r.git(&["init", "-q", "-b", "main"]);
        fs::write(
            r.0.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(r.0.join(".gitignore"), ".ank/index.db*\n").unwrap();
        r.commit("seed");
        r
    }

    fn git(&self, args: &[&str]) -> String {
        let out = spawn("git")
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
        text(&out.stdout)
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", message, "--allow-empty"]);
    }

    fn ank(&self, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .arg("--repo")
            .arg(&self.0)
            .current_dir(scratch::root())
            .output()
            .unwrap()
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.ank(args);
        assert_eq!(
            code(&out),
            0,
            "ank {args:?}: {}{}",
            text(&out.stdout),
            text(&out.stderr)
        );
        text(&out.stdout)
    }

    fn created(&self, args: &[&str]) -> String {
        self.ok(args)
            .split_whitespace()
            .nth(1)
            .expect("created <id> <slug>")
            .to_string()
    }

    /// An ADR scoping `src/new.rs` and nothing else, committed, and ratified
    /// when `accepted`.
    fn adr(&self, accepted: bool) -> String {
        let id = self.created(&[
            "new",
            "adr",
            "--title",
            "A rule for a file to come",
            "--constraint",
            "A binding rule.",
            "--scope",
            NEW,
        ]);
        self.commit("propose");
        if accepted {
            self.ok(&["accept", &id]);
            self.commit("ratify");
        }
        id
    }

    /// An open task scoped `scope`, committed.
    fn task(&self, scope: &str) -> String {
        let id = self.created(&[
            "new",
            "task",
            "--title",
            "Write the file",
            "--scope",
            scope,
            "--criteria",
            "The file exists.",
            "--no-verify",
        ]);
        self.commit("plan");
        id
    }

    /// `check`: its exit code, and every scope finding about `id`, each read
    /// from `check --kind <kind>` so it carries the kind it was filed under.
    fn check(&self, id: &str) -> (i32, Vec<Finding>) {
        let whole = self.ank(&["check"]);
        assert!(
            [0, 8].contains(&code(&whole)),
            "check: {}{}",
            text(&whole.stdout),
            text(&whole.stderr)
        );
        let mut found = Vec::new();
        for kind in ["scope-not-yet", "dead-scope"] {
            let out = self.ank(&["check", "--kind", kind]);
            assert!([0, 8].contains(&code(&out)), "{}", text(&out.stderr));
            found.extend(findings(&text(&out.stdout), id, kind));
        }
        (code(&whole), found)
    }

    /// The kinds `check --json` files the findings about `id` under, in order.
    fn kinds(&self, id: &str) -> Vec<String> {
        let out = self.ank(&["check", "--json"]);
        let doc = text(&out.stdout);
        let tail = format!("\",\"subject\":\"{id}\"");
        doc.match_indices(&tail)
            .map(|(at, _)| {
                let head = &doc[..at];
                let open = head
                    .rfind("\"kind\":\"")
                    .expect("a finding carries its kind");
                head[open + "\"kind\":\"".len()..].to_string()
            })
            .collect()
    }

    /// Every git process one `check` starts, as git's own trace names them:
    /// one `trace: built-in:` line per process, on all three platforms.
    fn git_processes(&self) -> Vec<String> {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let log = scratch::root().join(format!(
            "dead-scope-awaited-trace-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let out = spawn(ANK)
            .args(["check", "--repo"])
            .arg(&self.0)
            .current_dir(scratch::root())
            .env("GIT_TRACE", &log)
            .output()
            .unwrap();
        assert!([0, 8].contains(&code(&out)), "{}", text(&out.stderr));
        let trace = fs::read_to_string(&log).unwrap_or_default();
        let _ = fs::remove_file(&log);
        let started: Vec<String> = trace
            .lines()
            .filter_map(|l| l.split_once("trace: built-in: git "))
            .map(|(_, argv)| {
                // The fixtures differ by path and by commit, and the processes
                // are what is compared.
                argv.trim()
                    .replace(&self.0.display().to_string(), "<repo>")
                    .split(|c: char| c == '\'' || c == ' ' || c == '^')
                    .map(
                        |w| match w.len() == 40 && w.bytes().all(|b| b.is_ascii_hexdigit()) {
                            true => "<sha>",
                            false => w,
                        },
                    )
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        assert!(
            !started.is_empty(),
            "git wrote no trace: nothing was measured"
        );
        started
    }
}

/// The kinds about a scope, which is where a new one would have been added.
fn scope_kinds(kinds: Vec<String>) -> Vec<String> {
    kinds.into_iter().filter(|k| k.contains("scope")).collect()
}

fn awaited(task: &str) -> Finding {
    finding(
        "signal",
        "scope-not-yet",
        &format!("scope '{NEW}' matches no file yet: awaited by {task}, which is still open"),
        &[],
    )
}

fn owed(task: &str, ended: &str, what: &str) -> Finding {
    finding(
        "fault",
        "dead-scope",
        &format!("dead scope '{NEW}': no file matches it, and {task}, which covered it, is {ended}: {what}"),
        &[],
    )
}

fn unawaited() -> Finding {
    finding(
        "fault",
        "dead-scope",
        &format!("dead scope '{NEW}': no file matches it"),
        &[],
    )
}

#[test]
fn an_open_task_scoping_the_path_or_a_glob_over_it_makes_the_scope_awaited() {
    for accepted in [false, true] {
        for scope in [NEW, "src/**"] {
            let r = Repo::new("dead-scope-awaited-open");
            let adr = r.adr(accepted);
            let task = r.task(scope);
            assert_eq!(
                r.check(&adr),
                (0, vec![awaited(&task)]),
                "accepted: {accepted}, task scope {scope}"
            );
            assert_eq!(
                scope_kinds(r.kinds(&adr)),
                ["scope-not-yet"],
                "--json files it under a kind that already existed"
            );
        }
    }
}

#[test]
fn with_no_task_covering_it_the_scope_is_the_fault_it_was() {
    for accepted in [false, true] {
        let r = Repo::new("dead-scope-awaited-none");
        let adr = r.adr(accepted);
        // A task elsewhere is no promise about this path.
        r.task("src/main.rs");
        assert_eq!(
            r.check(&adr),
            (8, vec![unawaited()]),
            "accepted: {accepted}"
        );
        assert_eq!(scope_kinds(r.kinds(&adr)), ["dead-scope"]);
    }
}

#[test]
fn the_last_covering_task_ending_brings_the_fault_back_naming_it() {
    for accepted in [false, true] {
        let r = Repo::new("dead-scope-awaited-done");
        let adr = r.adr(accepted);
        let task = r.task(NEW);
        r.ok(&["claim", &task]);
        let sha = r.git(&["rev-parse", "HEAD"]);
        r.ok(&["done", &task, "--proof", &format!("commit:{}", sha.trim())]);
        r.commit("done");
        assert_eq!(
            r.check(&adr),
            (8, vec![owed(&task, "done", "its promise was not kept")]),
            "accepted: {accepted}"
        );

        let r = Repo::new("dead-scope-awaited-closed");
        let adr = r.adr(accepted);
        let task = r.task("src/**");
        r.ok(&["close", &task, "--reason", "not worth it"]);
        r.commit("closed");
        assert_eq!(
            r.check(&adr),
            (8, vec![owed(&task, "closed", "it withdrew the promise")]),
            "accepted: {accepted}"
        );
    }
}

#[test]
fn once_the_file_is_committed_the_scope_has_no_finding() {
    for accepted in [false, true] {
        let r = Repo::new("dead-scope-awaited-born");
        let adr = r.adr(accepted);
        r.task(NEW);
        fs::write(r.0.join(NEW), "pub fn new() {}\n").unwrap();
        r.commit("write it");
        assert_eq!(r.check(&adr), (0, vec![]), "accepted: {accepted}");
    }
}

#[test]
fn a_scope_git_records_deleted_is_not_awaited_whatever_task_covers_it() {
    for accepted in [false, true] {
        let r = Repo::new("dead-scope-awaited-deleted");
        fs::write(r.0.join(NEW), "pub fn new() {}\n").unwrap();
        r.commit("write it");
        let adr = r.adr(accepted);
        r.git(&["rm", "-q", NEW]);
        r.commit("drop it");
        let sha = r.git(&["rev-parse", "--short=12", "HEAD"]);
        r.task(NEW);
        assert_eq!(
            r.check(&adr),
            (
                0,
                vec![finding(
                    "signal",
                    "dead-scope",
                    &format!("dead scope '{NEW}': no file matches it"),
                    &[&format!("git records {NEW} deleted in {}", sha.trim())],
                )]
            ),
            "accepted: {accepted}"
        );
    }
}

#[test]
fn the_covering_task_starts_no_git_process_of_its_own() {
    for accepted in [false, true] {
        let elsewhere = Repo::new("dead-scope-awaited-trace-without");
        elsewhere.adr(accepted);
        elsewhere.task("src/main.rs");
        let covering = Repo::new("dead-scope-awaited-trace-with");
        covering.adr(accepted);
        covering.task(NEW);
        let (without, with) = (elsewhere.git_processes(), covering.git_processes());
        eprintln!(
            "accepted: {accepted}: {} git processes without the covering task, {} with it",
            without.len(),
            with.len()
        );
        assert_eq!(
            with, without,
            "accepted: {accepted}: the covering task is read from the index the run holds"
        );
    }
}
