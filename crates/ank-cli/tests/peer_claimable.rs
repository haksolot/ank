//! `context`, `claim` and the ready-task hint a refused claim prints give one
//! answer about a task a peer edge blocks (ADR-c23bef1cc93e,
//! TASK-d28565b8ed63).
//!
//! The defect this pins: `claim` held on a peer edge while `context` listed the
//! same task as claimable, because the listing never opened the peer; and the
//! hint skipped every task carrying a peer edge, so one whose peer blocker is
//! done was never offered. Measured at 6c99e7f7 in a scratch pair: `ready:
//! true` in `context --json` with the peer task open, with the peer moved away
//! and with it undeclared, while `claim` refused with code 7 in every one.
//!
//! **An edge that cannot be read holds**, here as in `claim`: a peer moved away
//! or undeclared is not claimable and is not offered.
//!
//! **Read only.** Every file under the peer's `.ank/` is compared byte for byte
//! before and after.
//!
//! **Through the binary**, every case.

mod scratch;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-claimable";
const OTHER: &str = "claude-code@peer-claimable-other";

/// The task `b` holds, which the local task waits on.
const TASK: &str = "TASK-abcdef012345";

struct Fixture {
    base: PathBuf,
    a: PathBuf,
    b: PathBuf,
}

impl Fixture {
    fn run_as(&self, agent: &str, args: &[&str]) -> (Option<i32>, String, String) {
        let root = scratch::root();
        let home = self.base.join("home");
        let out: Output = Command::new(ANK)
            .args(args)
            .current_dir(&self.a)
            .env("ANK_AGENT", agent)
            .env("TMPDIR", root)
            .env("TMP", root)
            .env("TEMP", root)
            .env("NO_COLOR", "1")
            .env("XDG_CONFIG_HOME", &home)
            .env("APPDATA", &home)
            .output()
            .expect("the binary runs");
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
        )
    }

    fn ok(&self, args: &[&str]) -> String {
        let (code, stdout, stderr) = self.run_as(AGENT, args);
        assert_eq!(code, Some(0), "ank {args:?} must succeed: {stderr}");
        stdout
    }

    /// The peer task's file, rewritten at `status`.
    fn peer_task(&self, status: &str) {
        fs::write(
            self.b.join(format!(".ank/entities/{TASK}.md")),
            format!(
                "---\nid: {TASK}\ntype: task\nslug: s\ntitle: Peer work that comes first\n\
                 created: 2026-08-01T00:00:00Z\nstatus: {status}\nscope:\n  - src/**\n\
                 blocked_by: []\ndone_criteria: |\n  Something holds.\nschema: 4\nversion: 1\n\
                 ---\n"
            ),
        )
        .unwrap();
    }

    fn task(&self, title: &str, blocked_by: Option<&str>) -> String {
        let mut args = vec![
            "new",
            "task",
            "--title",
            title,
            "--scope",
            "src/**",
            "--criteria",
            "It holds.",
            "--no-verify",
            "--json",
        ];
        if let Some(b) = blocked_by {
            args.extend(["--blocked-by", b]);
        }
        id_of(&self.ok(&args))
    }
}

fn id_of(json: &str) -> String {
    let at = json.find("\"TASK-").expect("the new task is named") + 1;
    json[at..at + 17].to_string()
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn commit(dir: &Path) {
    git(dir, &["add", "-A"]);
    git(
        dir,
        &[
            "-c",
            "user.email=t@example.invalid",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "init",
        ],
    );
}

fn corpus(f: &Fixture, dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    let run = |args: &[&str]| {
        let out = Command::new(ANK)
            .args(args)
            .current_dir(dir)
            .env("ANK_AGENT", AGENT)
            .env("TMPDIR", scratch::root())
            .env("XDG_CONFIG_HOME", f.base.join("home"))
            .env("APPDATA", f.base.join("home"))
            .output()
            .expect("the binary runs");
        assert!(out.status.success(), "{out:?}");
    };
    run(&["init"]);
    run(&["config", "default_branch", "main"]);
}

/// Every file under the peer's `.ank/`, with its bytes.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.join(".ank")];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(
                    p.strip_prefix(dir).unwrap().to_path_buf(),
                    fs::read(&p).unwrap(),
                );
            }
        }
    }
    out
}

/// The command a refusal names, as the refusal printed it.
fn hint(stderr: &str) -> String {
    stderr
        .lines()
        .find_map(|l| l.trim_start().strip_prefix("-> "))
        .unwrap_or_else(|| panic!("the refusal names a command: {stderr}"))
        .trim()
        .to_string()
}

/// The `ready` field `context --json` gives the task `id`.
fn ready_in_json(json: &str, id: &str) -> bool {
    let at = json
        .find(&format!("\"id\":\"{id}\""))
        .unwrap_or_else(|| panic!("{id} is listed: {json}"));
    let rest = &json[at..];
    let ready = rest.find("\"ready\":").expect("a ready field") + "\"ready\":".len();
    rest[ready..].starts_with("true")
}

/// Asks the three readers about the waiting task and checks they agree with
/// `expected`. The hint is drawn by refusing a claim on `held`, which another
/// identity holds, so the waiting task is the only candidate in scope.
fn agree(f: &Fixture, state: &str, waiting: &str, held: &str, expected: bool) {
    let short = &waiting[..9];

    let text = f.ok(&["context"]);
    assert!(
        text.contains("Local work that waits"),
        "{state}: listed: {text}"
    );
    assert_eq!(
        text.contains(&format!("ank claim {short}")),
        expected,
        "{state}: context text offers the task as claimable: {text}"
    );
    let json = f.ok(&["context", "--json"]);
    assert_eq!(
        ready_in_json(&json, waiting),
        expected,
        "{state}: context --json ready: {json}"
    );

    let (code, _, err) = f.run_as(AGENT, &["claim", held]);
    assert_eq!(code, Some(4), "{state}: held by the other identity: {err}");
    let offered = hint(&err) == format!("ank claim {waiting}   (another ready task in this scope)");
    assert_eq!(offered, expected, "{state}: the hint: {err}");
    if !expected {
        assert_eq!(hint(&err), "ank context", "{state}: {err}");
    }

    // claim, last: it gives the same answer, and on success the task is held,
    // so it is handed back for the next state.
    let (code, out, err) = f.run_as(AGENT, &["claim", waiting]);
    assert_eq!(
        code == Some(0),
        expected,
        "{state}: claim agrees with context: {out}{err}"
    );
    if expected {
        f.ok(&["release", waiting, "--reason", "measured"]);
    } else {
        assert_eq!(code, Some(7), "{state}: refused as blocked: {err}");
    }
}

#[test]
fn context_claim_and_the_hint_agree_on_a_task_a_peer_edge_blocks() {
    let base = scratch::dir("peer-claimable");
    let f = Fixture {
        a: base.join("a"),
        b: base.join("b"),
        base,
    };
    corpus(&f, &f.b);
    f.peer_task("open");
    commit(&f.b);
    corpus(&f, &f.a);
    f.ok(&["config", "peers.bb", "../b"]);
    let at = format!("{TASK}@bb");
    let waiting = f.task("Local work that waits", Some(&at));
    let held = f.task("Local work held elsewhere", None);
    commit(&f.a);
    let (code, _, err) = f.run_as(OTHER, &["claim", &held]);
    assert_eq!(code, Some(0), "{err}");

    let before = snapshot(&f.b);
    agree(&f, "peer task open", &waiting, &held, false);
    assert_eq!(snapshot(&f.b), before, "nothing under bb's .ank changed");

    f.peer_task("done");
    let before_done = snapshot(&f.b);

    let moved = f.base.join("b-moved");
    fs::rename(&f.b, &moved).unwrap();
    agree(&f, "peer moved away", &waiting, &held, false);
    fs::rename(&moved, &f.b).unwrap();

    f.ok(&["config", "--unset", "peers.bb"]);
    agree(&f, "peer undeclared", &waiting, &held, false);
    f.ok(&["config", "peers.bb", "../b"]);

    agree(&f, "peer task done", &waiting, &held, true);

    assert_ne!(before, before_done, "the fixture moved the peer task");
    assert_eq!(
        snapshot(&f.b),
        before_done,
        "nothing under bb's .ank changed"
    );
}
