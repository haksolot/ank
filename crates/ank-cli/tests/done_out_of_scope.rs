//! `done` names the files the branch changed outside the task's scope, and
//! never refuses on them (TASK-64f941335624).
//!
//! **Through the binary.** Workers widen a scope in nearly every wave, and
//! until now it was seen only when the worker logged it. Every case drives
//! `ank done` on a branch cut from `main`, for a task scoped `src/a/**`, and
//! reads what it printed: one line on standard error naming the files, and the
//! list under `outside_scope` in `--json`.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "test/done-out-of-scope";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("done-out-of-scope-gitconfig");
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

fn spawn(program: &str, dir: &Path, args: &[&str], trace: Option<&Path>) -> Output {
    let config = isolated_git_config();
    let home = scratch::root();
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", AGENT)
        .env("NO_COLOR", "1")
        .env("XDG_CONFIG_HOME", home)
        .env("APPDATA", home)
        .env("TMPDIR", home)
        .env("TMP", home)
        .env("TEMP", home)
        .env_remove("GIT_TRACE")
        .stdin(Stdio::null());
    if let Some(t) = trace {
        cmd.env("GIT_TRACE", t);
    }
    cmd.output()
        .unwrap_or_else(|e| panic!("{program} must run: {e}"))
}

fn run(program: &str, dir: &Path, args: &[&str]) -> String {
    let out = spawn(program, dir, args, None);
    assert!(
        out.status.success(),
        "{program} {args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn write(repo: &Path, path: &str, text: &str) {
    let p = repo.join(path);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn commit(dir: &Path, message: &str) {
    run("git", dir, &["add", "-A"]);
    run("git", dir, &["commit", "-q", "-m", message]);
}

/// A repository on `main` holding `src/a/x.rs`, `src/b/y.rs` and `docs/z.md`,
/// one verifier that always passes, and a task scoped `src/a/**` committed
/// there; then a branch `work` cut from `main`, with the task claimed on it.
fn fixture(what: &str) -> PathBuf {
    let repo = scratch::dir(what).join("repo");
    write(&repo, "src/a/x.rs", "x\n");
    write(&repo, "src/b/y.rs", "y\n");
    write(&repo, "docs/z.md", "z\n");
    run("git", &repo, &["init", "-q", "-b", "main"]);
    run(ANK, &repo, &["init"]);
    run(ANK, &repo, &["config", "default_branch", "main"]);
    run(ANK, &repo, &["config", "verifiers.fine.run", "echo fine"]);
    run(
        ANK,
        &repo,
        &["config", "verifiers.fine.inputs", r#"["src/**"]"#],
    );
    commit(&repo, "init");
    let new = run(
        ANK,
        &repo,
        &[
            "new",
            "task",
            "--title",
            "work on a",
            "--scope",
            "src/a/**",
            "--criteria",
            "the verifier passes",
            "--verify",
            "fine",
            "--json",
        ],
    );
    let at = new.find("TASK-").expect("new task --json names the id");
    let id = new[at..at + "TASK-".len() + 12].to_string();
    commit(&repo, "task");
    run("git", &repo, &["checkout", "-q", "-b", "work"]);
    run(ANK, &repo, &["claim", &id]);
    repo
}

/// The lines of standard error that name files changed outside the scope.
fn outside_lines(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter(|l| l.contains("outside the scope"))
        .collect()
}

fn done(repo: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let mut all = vec!["done"];
    all.extend(args);
    let out = spawn(ANK, repo, &all, None);
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn done_names_committed_and_uncommitted_files_outside_the_scope_in_one_line() {
    let repo = fixture("done-out-of-scope-named");
    write(&repo, "src/a/x.rs", "x changed\n");
    write(&repo, "src/b/y.rs", "y changed\n");
    commit(&repo, "work");
    // Uncommitted: a tracked file modified, and a file git does not know yet.
    write(&repo, "docs/z.md", "z changed\n");
    write(&repo, "docs/w.md", "w new\n");

    let (code, stdout, stderr) = done(&repo, &["--json"]);
    assert_eq!(
        code,
        Some(0),
        "done closes as it would without them:\n{stderr}"
    );

    let lines = outside_lines(&stderr);
    assert_eq!(lines.len(), 1, "exactly one line names them:\n{stderr}");
    let line = lines[0];
    for named in ["src/b/y.rs", "docs/z.md", "docs/w.md"] {
        assert!(line.contains(named), "{named} is named: {line}");
    }
    assert!(
        !line.contains("src/a/x.rs"),
        "a file in scope is not named: {line}"
    );
    assert!(!line.contains(".ank/"), "the corpus is not named: {line}");

    assert!(
        stdout.contains(r#""outside_scope":["docs/w.md","docs/z.md","src/b/y.rs"]"#),
        "done --json carries the list, sorted: {stdout}"
    );
}

#[test]
fn files_changed_only_on_the_default_branch_are_not_named_after_a_merge() {
    let repo = fixture("done-out-of-scope-merged");
    write(&repo, "src/a/x.rs", "x changed\n");
    write(&repo, "src/b/y.rs", "y changed\n");
    commit(&repo, "work");
    run("git", &repo, &["checkout", "-q", "main"]);
    write(&repo, "lib/m.rs", "landed elsewhere\n");
    write(&repo, "docs/z.md", "z on main\n");
    commit(&repo, "elsewhere");
    run("git", &repo, &["checkout", "-q", "work"]);
    run("git", &repo, &["merge", "-q", "--no-edit", "main"]);

    let (code, _, stderr) = done(&repo, &[]);
    assert_eq!(code, Some(0), "{stderr}");
    let lines = outside_lines(&stderr);
    assert_eq!(lines.len(), 1, "{stderr}");
    assert!(lines[0].contains("src/b/y.rs"), "{}", lines[0]);
    for only_on_main in ["lib/m.rs", "docs/z.md"] {
        assert!(
            !lines[0].contains(only_on_main),
            "{only_on_main} changed only on main and is not named: {}",
            lines[0]
        );
    }
}

#[test]
fn with_every_change_inside_the_scope_no_line_is_printed() {
    let repo = fixture("done-out-of-scope-none");
    write(&repo, "src/a/x.rs", "x changed\n");
    commit(&repo, "work");
    write(&repo, "src/a/new.rs", "uncommitted, in scope\n");

    let (code, stdout, stderr) = done(&repo, &["--json"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(outside_lines(&stderr).is_empty(), "no line: {stderr}");
    assert!(stdout.contains(r#""outside_scope":[]"#), "{stdout}");
}

/// The git processes a `done` starts with `files` committed outside the scope.
fn git_processes(what: &str, files: usize) -> usize {
    let repo = fixture(what);
    write(&repo, "src/a/x.rs", "x changed\n");
    for i in 0..files {
        write(&repo, &format!("docs/f{i}.md"), "outside\n");
    }
    commit(&repo, "work");
    let trace = repo.parent().unwrap().join("trace");
    let out = spawn(ANK, &repo, &["done"], Some(&trace));
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    fs::read_to_string(&trace)
        .unwrap()
        .lines()
        .filter(|l| l.contains("trace: built-in: git"))
        .count()
}

#[test]
fn the_line_starts_as_many_git_processes_for_one_file_as_for_thirty() {
    let one = git_processes("done-out-of-scope-trace-1", 1);
    let thirty = git_processes("done-out-of-scope-trace-30", 30);
    assert_eq!(one, thirty, "the count does not grow with the files named");
}
