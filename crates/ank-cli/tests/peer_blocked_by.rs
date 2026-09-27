//! A `blocked_by` names a task of a declared peer as `<id>@<peer>`, and an edge
//! whose peer cannot be read holds (issue #501, ADR-c23bef1cc93e,
//! TASK-08615a199a6a).
//!
//! **Holding is the point.** A reader that cannot see a peer answers locally
//! and says so (ADR-a1de673043b4); a blocker that did the same would answer
//! "nothing blocks you", which releases exactly the work the edge exists to
//! hold. So `claim` refuses while the peer's task is not done, and goes on
//! refusing when the peer is gone, naming the peer and the command.
//!
//! **Read only.** Every file under the peer's `.ank/` is compared byte for byte
//! before and after every verb, and nothing new may appear there.
//!
//! **Through the binary**, every case.

mod scratch;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-blocked-by";

/// The task `b` holds, and one it does not.
const TASK: &str = "TASK-abcdef012345";
const ABSENT: &str = "TASK-ffffffffffff";

struct Fixture {
    base: PathBuf,
    a: PathBuf,
    b: PathBuf,
}

impl Fixture {
    fn run(&self, dir: &Path, args: &[&str]) -> Output {
        let root = scratch::root();
        let home = self.base.join("home");
        Command::new(ANK)
            .args(args)
            .current_dir(dir)
            .env("ANK_AGENT", AGENT)
            .env("TMPDIR", root)
            .env("TMP", root)
            .env("TEMP", root)
            .env("NO_COLOR", "1")
            .env("XDG_CONFIG_HOME", &home)
            .env("APPDATA", &home)
            .output()
            .expect("the binary runs")
    }

    /// Run in `a`, the corpus that declares the peer.
    fn read(&self, args: &[&str]) -> (Option<i32>, String, String) {
        let out = self.run(&self.a, args);
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
        )
    }

    fn ok(&self, args: &[&str]) -> String {
        let (code, stdout, stderr) = self.read(args);
        assert_eq!(code, Some(0), "ank {args:?} must succeed: {stderr}");
        stdout
    }

    fn refused(&self, args: &[&str]) -> String {
        let (code, stdout, stderr) = self.read(args);
        assert_ne!(
            code,
            Some(0),
            "ank {args:?} must refuse, answered: {stdout}"
        );
        stderr
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

    /// A task of `a`, created through the binary, blocked by `blocker`.
    fn blocked_task(&self, blocker: &str) -> String {
        let out = self.ok(&[
            "new",
            "task",
            "--title",
            "Local work that waits",
            "--scope",
            "src/**",
            "--criteria",
            "It waits.",
            "--no-verify",
            "--blocked-by",
            blocker,
            "--json",
        ]);
        id_of(&out)
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

fn corpus(f: &Fixture, dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    let out = f.run(dir, &["init"]);
    assert!(out.status.success(), "{:?}", out);
    let out = f.run(dir, &["config", "default_branch", "main"]);
    assert!(out.status.success(), "{:?}", out);
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

fn fixture(name: &str) -> Fixture {
    let base = scratch::dir(name);
    let f = Fixture {
        a: base.join("a"),
        b: base.join("b"),
        base,
    };
    corpus(&f, &f.b);
    f.peer_task("open");
    commit(&f.b);
    corpus(&f, &f.a);
    let out = f.run(&f.a, &["config", "peers.bb", "../b"]);
    assert!(out.status.success(), "{:?}", out);
    commit(&f.a);
    f
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

#[test]
fn new_and_amend_accept_a_task_the_peer_holds_and_refuse_one_it_does_not() {
    let f = fixture("peer-blocked-by-write");
    let before = snapshot(&f.b);
    let at = format!("{TASK}@bb");

    let id = f.blocked_task(&at);
    let shown = f.ok(&["show", &id]);
    assert!(shown.contains(&format!("blocked_by: [{at}]")), "{shown}");

    // A prefix is resolved in the peer and stored whole.
    let second = f.blocked_task("TASK-abcd@bb");
    let shown = f.ok(&["show", &second]);
    assert!(shown.contains(&format!("blocked_by: [{at}]")), "{shown}");

    let absent = format!("{ABSENT}@bb");
    let err = f.refused(&[
        "new",
        "task",
        "--title",
        "Waits on nothing real",
        "--scope",
        "src/**",
        "--criteria",
        "It waits.",
        "--no-verify",
        "--blocked-by",
        &absent,
    ]);
    assert!(err.contains(&absent), "names what bb lacks: {err}");

    // amend: a local task gains the peer edge, and a missing one is refused.
    let plain = id_of(&f.ok(&[
        "new",
        "task",
        "--title",
        "Amended into waiting",
        "--scope",
        "src/**",
        "--criteria",
        "It waits.",
        "--no-verify",
        "--json",
    ]));
    let err = f.refused(&["amend", &plain, "--blocked-by", &absent]);
    assert!(err.contains(&absent), "names what bb lacks: {err}");
    let amended = f.ok(&["amend", &plain, "--blocked-by", &at]);
    assert!(amended.contains(&format!("+blocked_by {at}")), "{amended}");
    let shown = f.ok(&["show", &plain]);
    assert!(shown.contains(&format!("blocked_by: [{at}]")), "{shown}");
    let dropped = f.ok(&["amend", &plain, "--drop-blocked-by", &at]);
    assert!(dropped.contains(&format!("-blocked_by {at}")), "{dropped}");
    let shown = f.ok(&["show", &plain]);
    assert!(shown.contains("blocked_by: []"), "{shown}");

    // An undeclared peer is refused at creation, naming the declaration.
    let err = f.refused(&["amend", &plain, "--blocked-by", &format!("{TASK}@zz")]);
    assert_eq!(hint(&err), "ank config peers.zz <path>", "{err}");

    let checked = f.read(&["check"]);
    assert!(
        !checked.1.contains(&at),
        "a resolvable edge is no finding: {checked:?}"
    );
    assert_eq!(snapshot(&f.b), before, "nothing under bb's .ank changed");
}

#[test]
fn graph_draws_the_edge_to_the_peer() {
    let f = fixture("peer-blocked-by-graph");
    let before = snapshot(&f.b);
    let at = format!("{TASK}@bb");
    let id = f.blocked_task(&at);

    let drawn = f.ok(&["graph"]);
    let line = drawn
        .lines()
        .find(|l| l.contains("Local work that waits"))
        .unwrap_or_else(|| panic!("the task is drawn: {drawn}"));
    assert!(
        line.contains("@bb"),
        "the edge is drawn on its node: {drawn}"
    );

    let json = f.ok(&["graph", "--json"]);
    assert!(
        json.contains(&format!("{{\"task\":\"{id}\",\"blocked_by\":\"{at}\"}}")),
        "{json}"
    );
    assert_eq!(snapshot(&f.b), before, "nothing under bb's .ank changed");
}

#[test]
fn claim_holds_until_the_peer_task_is_done_and_holds_when_the_peer_is_gone() {
    let f = fixture("peer-blocked-by-claim");
    let at = format!("{TASK}@bb");
    let id = f.blocked_task(&at);
    commit(&f.a);

    let before = snapshot(&f.b);
    let err = f.refused(&["claim", &id]);
    assert!(err.contains(&at), "names the peer blocker: {err}");
    assert_eq!(hint(&err), format!("ank show {at}"), "{err}");
    assert_eq!(snapshot(&f.b), before, "nothing under bb's .ank changed");

    // The peer's task, done as the peer's corpus holds it on disk.
    f.peer_task("done");
    let before = snapshot(&f.b);

    // bb moved away: the edge cannot be read, and it holds.
    let moved = f.base.join("b-moved");
    fs::rename(&f.b, &moved).unwrap();
    let err = f.refused(&["claim", &id]);
    assert!(err.contains("peer 'bb'"), "names the peer: {err}");
    assert!(err.contains(&at), "names the edge: {err}");
    let settle = hint(&err);
    assert!(settle.starts_with("ank "), "names a command: {err}");
    let (_, checked, _) = f.read(&["check"]);
    let finding = checked
        .lines()
        .find(|l| l.contains(&at))
        .unwrap_or_else(|| panic!("check reports the edge: {checked}"));
    assert!(finding.contains("cannot be read"), "{checked}");

    // Back, and done there: the claim is taken.
    fs::rename(&moved, &f.b).unwrap();
    let claimed = f.ok(&["claim", &id]);
    assert!(claimed.contains(&id), "{claimed}");
    assert_eq!(snapshot(&f.b), before, "nothing under bb's .ank changed");
}

#[test]
fn check_reports_an_undeclared_peer_and_a_task_the_peer_does_not_hold() {
    let f = fixture("peer-blocked-by-check");
    let before = snapshot(&f.b);
    let at = format!("{TASK}@bb");
    let id = f.blocked_task(&at);

    // A task bb does not hold, written by hand: `new` would refuse it.
    let path = f.a.join(format!(".ank/entities/{id}.md"));
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.replace(TASK, ABSENT)).unwrap();
    let (code, checked, _) = f.read(&["check"]);
    assert_eq!(code, Some(8), "a fault: {checked}");
    let finding = checked
        .lines()
        .find(|l| l.contains(&format!("{ABSENT}@bb")))
        .unwrap_or_else(|| panic!("check reports the edge: {checked}"));
    assert!(finding.contains("does not hold"), "{checked}");
    fs::write(&path, text).unwrap();

    f.ok(&["config", "--unset", "peers.bb"]);
    let (code, checked, _) = f.read(&["check"]);
    assert_eq!(code, Some(8), "a fault: {checked}");
    let finding = checked
        .lines()
        .find(|l| l.contains(&at))
        .unwrap_or_else(|| panic!("check reports the edge: {checked}"));
    assert!(finding.contains("undeclared peer"), "{checked}");
    // And claim holds on it too.
    let err = f.refused(&["claim", &id]);
    assert_eq!(hint(&err), "ank config peers.bb <path>", "{err}");
    assert_eq!(snapshot(&f.b), before, "nothing under bb's .ank changed");
}
