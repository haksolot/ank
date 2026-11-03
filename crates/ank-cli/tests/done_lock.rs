//! `done` runs one task's verifiers at a time per repository, and a second
//! `done` waits (ADR-eab130f731f3, TASK-2532e621d383).
//!
//! **Through the binary, and never through a stopwatch.** Every case drives
//! two `ank done` in two linked worktrees of one scratch repository, and what
//! ran when is read from markers the verifiers write: `gate` appends `start
//! <worktree>` to `<root>/log`, waits for `<root>/go`, and appends `end
//! <worktree>`; `meet` passes only if the other worktree's `meet` is running at
//! the same moment. Elapsed time is never measured; a deadline only bounds how
//! long a red test hangs.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT_A: &str = "test/lock-a";
const AGENT_B: &str = "test/lock-b";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("done-lock-gitconfig");
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

fn command(program: &str, dir: &Path, agent: &str, args: &[&str]) -> Command {
    let config = isolated_git_config();
    let home = scratch::root();
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", agent)
        .env("NO_COLOR", "1")
        .env("XDG_CONFIG_HOME", home)
        .env("APPDATA", home)
        .env("TMPDIR", home)
        .env("TMP", home)
        .env("TEMP", home)
        .stdin(Stdio::null());
    cmd
}

fn spawn(program: &str, dir: &Path, agent: &str, args: &[&str]) -> Output {
    command(program, dir, agent, args)
        .output()
        .unwrap_or_else(|e| panic!("{program} must run: {e}"))
}

fn run(program: &str, dir: &Path, agent: &str, args: &[&str]) -> String {
    let out = spawn(program, dir, agent, args);
    assert!(
        out.status.success(),
        "{program} {args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn commit(dir: &Path, message: &str) {
    run("git", dir, AGENT_A, &["add", "-A"]);
    run("git", dir, AGENT_A, &["commit", "-q", "-m", message]);
}

/// `<root>/repo` is the repository and `<root>/wt` a linked worktree of it;
/// the markers live in `<root>`, which both reach as `..`.
struct Fixture {
    root: PathBuf,
    repo: PathBuf,
    wt: PathBuf,
}

/// A `done` started in the background, its standard error going to a file the
/// test can read while it runs.
struct Background {
    child: Child,
    stderr: PathBuf,
}

impl Background {
    fn stderr(&self) -> String {
        fs::read_to_string(&self.stderr).unwrap_or_default()
    }

    fn finish(mut self) -> (Option<i32>, String) {
        let status = self.child.wait().unwrap();
        (status.code(), self.stderr())
    }
}

impl Fixture {
    /// The repository with the three verifiers declared, `lock_off` writing
    /// `verifier_lock: false`, and one committed task per `(title, verify)`.
    /// The worktree is added last, so it carries every task. Answers the ids
    /// in the order given.
    fn new(what: &str, lock_off: bool, tasks: &[(&str, &[&str])]) -> (Fixture, Vec<String>) {
        let root = scratch::dir(what);
        let repo = root.join("repo");
        let wt = root.join("wt");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/x"), "x\n").unwrap();
        run("git", &repo, AGENT_A, &["init", "-q", "-b", "main"]);
        run(ANK, &repo, AGENT_A, &["init"]);
        run(ANK, &repo, AGENT_A, &["config", "default_branch", "main"]);
        let gate = r#"w=$(basename "$PWD"); echo "start $w" >> ../log; i=0; while [ ! -e ../go ] && [ $i -lt 1200 ]; do sleep 0.05; i=$((i+1)); done; echo "end $w" >> ../log; test -e ../go"#;
        let meet = r#"touch "../meet-$(basename "$PWD")"; i=0; while { [ ! -e ../meet-repo ] || [ ! -e ../meet-wt ]; } && [ $i -lt 1200 ]; do sleep 0.05; i=$((i+1)); done; [ -e ../meet-repo ] && [ -e ../meet-wt ]"#;
        for (key, value) in [
            ("verifiers.gate.run", gate),
            ("verifiers.gate.timeout", "2m"),
            ("verifiers.meet.run", meet),
            ("verifiers.meet.timeout", "2m"),
            ("verifiers.fine.run", "echo fine"),
            ("verifiers.fine.inputs", r#"["src/**"]"#),
        ] {
            run(ANK, &repo, AGENT_A, &["config", key, value]);
        }
        if lock_off {
            run(ANK, &repo, AGENT_A, &["config", "verifier_lock", "false"]);
        }
        commit(&repo, "init");
        let ids = tasks
            .iter()
            .map(|(title, verify)| task(&repo, title, verify))
            .collect();
        run(
            "git",
            &repo,
            AGENT_A,
            &["worktree", "add", "-q", "-b", "other", wt.to_str().unwrap()],
        );
        (Fixture { root, repo, wt }, ids)
    }

    fn done_in_background(&self, dir: &Path, agent: &str, name: &str) -> Background {
        let stderr = self.root.join(format!("{name}.stderr"));
        let child = command(ANK, dir, agent, &["done"])
            .stdout(Stdio::null())
            .stderr(fs::File::create(&stderr).unwrap())
            .spawn()
            .unwrap();
        Background { child, stderr }
    }

    fn log(&self) -> Vec<String> {
        fs::read_to_string(self.root.join("log"))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn open_the_gate(&self) {
        fs::write(self.root.join("go"), "").unwrap();
    }
}

/// A task over `src/**` declaring `verify`, committed in `dir`, not claimed.
fn task(dir: &Path, title: &str, verify: &[&str]) -> String {
    let mut args = vec![
        "new",
        "task",
        "--title",
        title,
        "--scope",
        "src/**",
        "--criteria",
        "the verifiers pass",
        "--json",
    ];
    for v in verify {
        args.extend(["--verify", v]);
    }
    let new = run(ANK, dir, AGENT_A, &args);
    let at = new.find("TASK-").expect("new task --json names the id");
    let id = new[at..at + "TASK-".len() + 12].to_string();
    commit(dir, title);
    id
}

/// Polls until `cond` holds. The deadline bounds a red test and is never what
/// a case asserts.
fn wait_until(what: &str, mut cond: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(90);
    while !cond() {
        assert!(Instant::now() < deadline, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn waiting_line(stderr: &str) -> Option<&str> {
    stderr.lines().find(|l| l.starts_with("waiting:"))
}

#[test]
fn two_dones_started_together_run_their_verifiers_one_after_the_other() {
    let (f, ids) = Fixture::new(
        "done-lock-serial",
        false,
        &[("first gated", &["gate"]), ("second gated", &["gate"])],
    );
    run(ANK, &f.repo, AGENT_A, &["claim", &ids[0]]);
    run(ANK, &f.wt, AGENT_B, &["claim", &ids[1]]);

    let a = f.done_in_background(&f.repo, AGENT_A, "a");
    let b = f.done_in_background(&f.wt, AGENT_B, "b");

    wait_until("one gate started and the other done waits", || {
        f.log().iter().any(|l| l.starts_with("start "))
            && (waiting_line(&a.stderr()).is_some() || waiting_line(&b.stderr()).is_some())
    });
    let log = f.log();
    assert_eq!(
        log.iter().filter(|l| l.starts_with("start ")).count(),
        1,
        "the second verifier started while the first held the lock: {log:?}"
    );
    let first = log[0].strip_prefix("start ").unwrap().to_string();
    // Who waits is whoever lost the race; the line names the other.
    let (waiter, holder_task, holder_agent) = if first == "repo" {
        ("wt", &ids[0], AGENT_A)
    } else {
        ("repo", &ids[1], AGENT_B)
    };
    let waiter_err = if waiter == "wt" {
        b.stderr()
    } else {
        a.stderr()
    };
    let line = waiting_line(&waiter_err)
        .unwrap_or_else(|| panic!("the waiter says why:\n{waiter_err}"))
        .to_string();
    assert!(
        line.contains(holder_task.as_str()),
        "names the task: {line}"
    );
    assert!(line.contains(holder_agent), "names the identity: {line}");

    f.open_the_gate();
    let (code_a, err_a) = a.finish();
    let (code_b, err_b) = b.finish();
    assert_eq!(code_a, Some(0), "{err_a}");
    assert_eq!(code_b, Some(0), "{err_b}");
    assert_eq!(
        [err_a.as_str(), err_b.as_str()]
            .iter()
            .filter(|e| waiting_line(e).is_some())
            .count(),
        1,
        "one waits, once:\n{err_a}\n---\n{err_b}"
    );
    assert_eq!(
        waiter_err
            .lines()
            .filter(|l| l.starts_with("waiting:"))
            .count(),
        1,
        "said once"
    );
    assert_eq!(
        f.log(),
        vec![
            format!("start {first}"),
            format!("end {first}"),
            format!("start {waiter}"),
            format!("end {waiter}"),
        ],
        "one after the other"
    );
}

#[test]
fn with_the_lock_off_the_two_verifiers_run_at_the_same_time() {
    let (f, ids) = Fixture::new(
        "done-lock-off",
        true,
        &[("first meets", &["meet"]), ("second meets", &["meet"])],
    );
    run(ANK, &f.repo, AGENT_A, &["claim", &ids[0]]);
    run(ANK, &f.wt, AGENT_B, &["claim", &ids[1]]);

    // `meet` passes only if the other one is running at the same moment:
    // serialised, the first would wait for a marker nobody writes until it
    // has given up.
    let a = f.done_in_background(&f.repo, AGENT_A, "a");
    let b = f.done_in_background(&f.wt, AGENT_B, "b");
    let (code_a, err_a) = a.finish();
    let (code_b, err_b) = b.finish();
    assert_eq!(code_a, Some(0), "{err_a}");
    assert_eq!(code_b, Some(0), "{err_b}");
    assert!(waiting_line(&err_a).is_none(), "{err_a}");
    assert!(waiting_line(&err_b).is_none(), "{err_b}");
}

#[test]
fn a_done_killed_while_holding_the_lock_leaves_it_free() {
    let (f, ids) = Fixture::new(
        "done-lock-killed",
        false,
        &[("killed", &["gate"]), ("next", &["gate"])],
    );
    run(ANK, &f.repo, AGENT_A, &["claim", &ids[0]]);
    run(ANK, &f.wt, AGENT_B, &["claim", &ids[1]]);

    let mut a = f.done_in_background(&f.repo, AGENT_A, "a");
    wait_until("the first gate started", || {
        f.log().iter().any(|l| l == "start repo")
    });
    a.child.kill().unwrap();
    a.child.wait().unwrap();
    // The orphaned verifier, if the kill left one, ends on the gate too.
    f.open_the_gate();

    let out = spawn(ANK, &f.wt, AGENT_B, &["done"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert!(
        waiting_line(&err).is_none(),
        "a killed holder leaves nothing to wait for:\n{err}"
    );
}

#[test]
fn a_done_whose_verifiers_are_all_reused_takes_no_lock() {
    let (f, ids) = Fixture::new(
        "done-lock-reused",
        false,
        &[
            ("runs fine", &["fine"]),
            ("holds the lock", &["gate"]),
            ("reuses fine", &["fine"]),
        ],
    );
    run(ANK, &f.repo, AGENT_A, &["claim", &ids[0]]);
    let first = spawn(ANK, &f.repo, AGENT_A, &["done"]);
    assert_eq!(
        first.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );

    run(ANK, &f.wt, AGENT_B, &["claim", &ids[1]]);
    let holder = f.done_in_background(&f.wt, AGENT_B, "holder");
    wait_until("the gate started, so the lock is held", || {
        f.log().iter().any(|l| l == "start wt")
    });

    run(ANK, &f.repo, AGENT_A, &["claim", &ids[2]]);
    let out = spawn(ANK, &f.repo, AGENT_A, &["done"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert!(err.contains("reused"), "fine was reused:\n{err}");
    assert!(
        waiting_line(&err).is_none(),
        "nothing ran, so nothing waited:\n{err}"
    );

    f.open_the_gate();
    let (code, err) = holder.finish();
    assert_eq!(code, Some(0), "{err}");
}

#[test]
fn config_reads_and_writes_verifier_lock() {
    let (f, _) = Fixture::new("done-lock-config", false, &[]);
    assert_eq!(
        run(ANK, &f.repo, AGENT_A, &["config", "verifier_lock"]),
        "true (default)",
        "absent, the lock is on"
    );
    run(ANK, &f.repo, AGENT_A, &["config", "verifier_lock", "false"]);
    let file = fs::read_to_string(f.repo.join(".ank/config.yml")).unwrap();
    assert!(file.contains("verifier_lock: false\n"), "{file}");
    assert_eq!(
        run(ANK, &f.repo, AGENT_A, &["config", "verifier_lock"]),
        "false"
    );
    let bad = spawn(ANK, &f.repo, AGENT_A, &["config", "verifier_lock", "maybe"]);
    assert_ne!(bad.status.code(), Some(0), "a boolean or nothing");
    let file = fs::read_to_string(f.repo.join(".ank/config.yml")).unwrap();
    assert!(file.contains("verifier_lock: false\n"), "unchanged: {file}");
}
