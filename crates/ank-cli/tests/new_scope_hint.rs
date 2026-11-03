//! `ank new task` names the files finished tasks declared beside the scope it
//! is given (TASK-cc8bdc66a2ec).
//!
//! Through the binary, in a scratch corpus where three done tasks declare both
//! `src/a.rs` and `tests/main.rs`, one declares `src/a.rs` alone and one
//! declares `src/a.rs` and `docs/x.md`. A second family of seven done tasks
//! declares `src/b.rs` beside six other entries at six different frequencies,
//! which is what the cap of five and the order are measured against.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "human:marie";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("new-scope-hint-gitconfig");
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
        .env_remove("EDITOR")
        .env_remove("VISUAL")
        .env_remove("GIT_TRACE")
        .env("ANK_AGENT", AGENT);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

struct Repo(PathBuf);

impl Repo {
    fn new(what: &str) -> Repo {
        let dir = scratch::dir(what);
        fs::create_dir_all(dir.join(".ank/entities")).unwrap();
        for f in [
            "src/a.rs",
            "src/b.rs",
            "src/c.rs",
            "tests/main.rs",
            "docs/x.md",
        ] {
            let p = dir.join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, "\n").unwrap();
        }
        let r = Repo(dir);
        r.git(&["init", "-q", "-b", "main"]);
        fs::write(
            r.0.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(r.0.join(".gitignore"), ".ank/index.db*\n").unwrap();
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "seed"]);
        r
    }

    fn git(&self, args: &[&str]) {
        let out = spawn("git")
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", stderr(&out));
    }

    fn ank_with(&self, args: &[&str], env: &[(&str, &Path)]) -> Output {
        let mut c = spawn(ANK);
        for (k, v) in env {
            c.env(k, v);
        }
        c.args(args)
            .arg("--repo")
            .arg(&self.0)
            .current_dir(scratch::root())
            .output()
            .unwrap()
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.ank_with(args, &[]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "ank {args:?}: {}{}",
            stdout(&out),
            stderr(&out)
        );
        stdout(&out)
    }

    /// A task through `new`, then marked done in its file: the fixture needs
    /// the finished state and none of the road to it.
    fn done_task(&self, scope: &[&str]) {
        let mut args = vec!["new", "task", "--title", "Finished work", "--no-verify"];
        for s in scope {
            args.extend_from_slice(&["--scope", s]);
        }
        let id = self
            .ok(&args)
            .split_whitespace()
            .nth(1)
            .expect("created <id> <title>")
            .to_string();
        let file = find_file(&self.0.join(".ank"), &format!("{id}.md")).expect("the entity file");
        let text = fs::read_to_string(&file).unwrap();
        assert!(text.contains("status: open"), "{text}");
        fs::write(&file, text.replace("status: open", "status: done")).unwrap();
    }

    /// The scope of a task `new` printed, read back through `show --json`.
    fn scope_of(&self, id: &str) -> String {
        self.ok(&["show", id, "--json"])
    }
}

fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for e in fs::read_dir(dir).ok()?.flatten() {
        let p = e.path();
        if p.is_dir() {
            if let Some(found) = find_file(&p, name) {
                return Some(found);
            }
        } else if p.file_name().is_some_and(|n| n == name) {
            return Some(p);
        }
    }
    None
}

/// The corpus the criterion describes, plus the `src/b.rs` family.
fn corpus(what: &str) -> Repo {
    let r = Repo::new(what);
    for _ in 0..3 {
        r.done_task(&["src/a.rs", "tests/main.rs"]);
    }
    r.done_task(&["src/a.rs"]);
    r.done_task(&["src/a.rs", "docs/x.md"]);
    // Seven tasks on src/b.rs: tests/z.rs in all seven, y in six, ... u in two.
    // Reverse alphabetical, so the most frequent first is not an accident of
    // sorting by path. Six entries reach two, one more than the cap.
    let beside = [
        ("tests/z.rs", 7),
        ("tests/y.rs", 6),
        ("tests/x.rs", 5),
        ("tests/w.rs", 4),
        ("tests/v.rs", 3),
        ("tests/u.rs", 2),
    ];
    for i in 0..7 {
        let mut scope = vec!["src/b.rs"];
        for (path, n) in beside {
            if i < n {
                scope.push(path);
            }
        }
        r.done_task(&scope);
    }
    r
}

fn created_id(said: &str) -> String {
    said.split_whitespace()
        .nth(1)
        .expect("created <id> <title>")
        .to_string()
}

#[test]
fn new_task_names_what_finished_tasks_declared_beside_its_scope() {
    let r = corpus("scope-hint-text");
    let said = r.ok(&["new", "task", "--title", "Next", "--scope", "src/a.rs"]);

    // The hint suggests and adds nothing: the scope is exactly what was given.
    let show = r.scope_of(&created_id(&said));
    assert!(
        show.contains("scope:\\n  - src/a.rs\\n") && !show.contains("  - tests/main.rs"),
        "the task's scope must be exactly src/a.rs: {show}"
    );

    assert!(
        said.contains("tests/main.rs") && said.contains("3 of 5"),
        "the hint must name tests/main.rs, 3 of the 5 finished tasks declaring src/a.rs: {said}"
    );
    assert!(
        !said.contains("docs/x.md"),
        "a file one finished task declared is a coincidence, not a pattern: {said}"
    );
}

#[test]
fn the_hint_names_five_entries_at_most_the_most_frequent_first() {
    let r = corpus("scope-hint-cap");
    let said = r.ok(&["new", "task", "--title", "Next", "--scope", "src/b.rs"]);
    let named: Vec<&str> = [
        "tests/z.rs",
        "tests/y.rs",
        "tests/x.rs",
        "tests/w.rs",
        "tests/v.rs",
    ]
    .into_iter()
    .collect();
    let at: Vec<usize> = named
        .iter()
        .map(|p| {
            said.find(p)
                .unwrap_or_else(|| panic!("{p} must be named: {said}"))
        })
        .collect();
    assert!(
        at.windows(2).all(|w| w[0] < w[1]),
        "most frequent first, z (7) to v (3): {said}"
    );
    for (path, n) in [("tests/z.rs", 7), ("tests/v.rs", 3)] {
        assert!(
            said.contains(&format!("{path}  {n} of 7")),
            "{path} carries its count: {said}"
        );
    }
    assert!(
        !said.contains("tests/u.rs"),
        "the sixth entry is past the cap of five: {said}"
    );
}

#[test]
fn a_scope_no_finished_task_declares_prints_no_hint() {
    let r = corpus("scope-hint-none");
    let said = r.ok(&["new", "task", "--title", "Next", "--scope", "src/c.rs"]);
    assert_eq!(
        said.lines().count(),
        1,
        "only the created line, no hint: {said}"
    );
}

#[test]
fn new_json_carries_the_hint_under_a_declared_key() {
    let r = corpus("scope-hint-json");
    let said = r.ok(&[
        "new", "task", "--title", "Next", "--scope", "src/a.rs", "--json",
    ]);
    assert!(
        said.contains(r#""scope_hint":[{"path":"tests/main.rs","count":3,"of":5}]"#),
        "{said}"
    );
    let said = r.ok(&[
        "new", "task", "--title", "Next", "--scope", "src/c.rs", "--json",
    ]);
    assert!(said.contains(r#""scope_hint":[]"#), "{said}");

    // Declared in the contract table, which is what help --json renders.
    let help = r.ok(&["help", "new", "--json"]);
    assert!(help.contains(r#""scope_hint""#), "{help}");
}

/// One line per git process, written by git itself (ADR-cc65f1388a71).
fn git_processes(trace: &Path) -> usize {
    fs::read_to_string(trace)
        .unwrap_or_default()
        .lines()
        .filter(|l| l.contains("trace: built-in:") || l.contains("trace: exec:"))
        .count()
}

#[test]
fn the_hint_costs_new_no_git_process() {
    let r = corpus("scope-hint-trace");
    let hinted = r.0.with_extension("hinted.trace");
    let bare = r.0.with_extension("bare.trace");
    let out = r.ank_with(
        &["new", "task", "--title", "Next", "--scope", "src/a.rs"],
        &[("GIT_TRACE", &hinted)],
    );
    assert!(stdout(&out).contains("3 of 5"), "{}", stdout(&out));
    let out = r.ank_with(
        &["new", "task", "--title", "Next", "--scope", "src/c.rs"],
        &[("GIT_TRACE", &bare)],
    );
    assert_eq!(stdout(&out).lines().count(), 1, "{}", stdout(&out));
    let (h, b) = (git_processes(&hinted), git_processes(&bare));
    // The control: the same instrument through the same harness counts the
    // git processes of a verb that starts some, so a zero above is a count
    // and not a trace that was never written.
    let control = r.0.with_extension("status.trace");
    r.ank_with(&["status"], &[("GIT_TRACE", &control)]);
    let c = git_processes(&control);
    eprintln!("git processes: new with a hint {h}, new without one {b}, status {c}");
    assert!(c > 0, "the instrument counted nothing on status: blind");
    assert_eq!(h, b, "the hint started git processes of its own");
    assert_eq!(h, 0, "new with a hint started {h} git processes");
}
