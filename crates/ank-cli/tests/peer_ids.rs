//! `show`, `find` and `log` read a peer's entity as `<id>@<peer>`, and a refused
//! identifier names the form that works (issue #501, ADR-c23bef1cc93e,
//! TASK-f4c4b3bafa2d).
//!
//! `context` inside a peer already prints `ADR-…@aa`, so the form existed on
//! output and not on input: `ank show TASK-…@bb` answered "entity not found",
//! `ank show bb:TASK-…` pointed at `ank find bb:TASK-…`, which fails too, and
//! `ank log TASK-…@bb` wrote the identifier as a message on the held task.
//!
//! **Reading only** (ADR-a1de673043b4): every file under the peer is compared
//! byte for byte before and after, as TASK-13e802e46050 did, and nothing new may
//! appear there — an `index.db` above all.
//!
//! **Through the binary**, and each hint a refusal prints is run in its turn:
//! a hint that fails the way its refusal did is the defect this task exists for.

mod scratch;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-ids";

/// The task `b` holds, and the log entry about it.
const TASK: &str = "TASK-abcdef012345";
const LOG: &str = "LOG-0123456789ab";

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

    /// Run in `a`, the reader.
    fn read(&self, args: &[&str]) -> (bool, String, String) {
        let out = self.run(&self.a, args);
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
        )
    }

    fn ok(&self, args: &[&str]) -> String {
        let (ok, stdout, stderr) = self.read(args);
        assert!(ok, "ank {args:?} must succeed: {stderr}");
        stdout
    }

    fn refused(&self, args: &[&str]) -> String {
        let (ok, stdout, stderr) = self.read(args);
        assert!(!ok, "ank {args:?} must refuse, answered: {stdout}");
        stderr
    }
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
    fs::write(
        f.b.join(format!(".ank/entities/{TASK}.md")),
        format!(
            "---\nid: {TASK}\ntype: task\nslug: s\ntitle: Peer work to read\n\
             created: 2026-08-01T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
             blocked_by: []\ndone_criteria: |\n  Something holds.\nschema: 4\nversion: 1\n\
             ---\n\nThe body of the peer task.\n"
        ),
    )
    .unwrap();
    fs::write(
        f.b.join(format!(".ank/entities/{LOG}.md")),
        format!(
            "---\nid: {LOG}\ntype: log\ntitle: tried the obvious route first\n\
             created: 2026-08-01T00:00:01Z\nauthor: someone@b\nscope:\n  - src/**\n\
             about: {TASK}\nseq: 1\nschema: 4\nversion: 1\n---\n"
        ),
    )
    .unwrap();
    commit(&f.b);
    corpus(&f, &f.a);
    let out = f.run(&f.a, &["config", "peers.bb", "../b"]);
    assert!(out.status.success(), "{:?}", out);
    let out = f.run(&f.a, &["config", "peers.gone", "../nowhere"]);
    assert!(out.status.success(), "{:?}", out);
    commit(&f.a);
    f
}

/// Every file under `dir` but git's own, with its bytes.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
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

/// A hint as argv, `ank` dropped and `<path>` given a value.
fn argv(hint: &str, path: &str) -> Vec<String> {
    let mut words = hint.split_whitespace();
    assert_eq!(
        words.next(),
        Some("ank"),
        "a hint is an ank command: {hint}"
    );
    words
        .map(|w| {
            if w == "<path>" {
                path.to_string()
            } else {
                w.to_string()
            }
        })
        .collect()
}

#[test]
fn show_find_and_log_read_a_peer_entity_and_write_nothing_there() {
    let f = fixture("peer-ids-read");
    let at = format!("{TASK}@bb");
    let before = snapshot(&f.b);

    let shown = f.ok(&["show", &at]);
    assert!(shown.contains(&format!("id: {TASK}")), "{shown}");
    assert!(
        shown.contains("The body of the peer task."),
        "whole: {shown}"
    );
    assert!(shown.contains("tried the obvious route first"), "{shown}");
    // A prefix reaches it too, as it does locally.
    let short = f.ok(&["show", "TASK-abcd@bb"]);
    assert!(short.contains(&format!("id: {TASK}")), "{short}");

    let found = f.ok(&["find", &at]);
    assert!(found.contains("Peer work to read"), "{found}");
    // Listed in the form that reaches it back from here.
    assert!(found.contains("@bb"), "{found}");

    let logged = f.ok(&["log", &at]);
    assert!(logged.contains("tried the obvious route first"), "{logged}");

    // The JSON forms read the same entity.
    let json = f.ok(&["show", &at, "--json"]);
    assert!(json.contains(&format!("\"id\":\"{TASK}\"")), "{json}");

    assert_eq!(snapshot(&f.b), before, "nothing under the peer changed");
    assert!(!f.b.join(".ank/index.db").exists());
    // And `log <id>@<peer>` was read, never written as a message here.
    let (_, local, _) = f.read(&["find", "tried the obvious"]);
    assert!(!local.contains("LOG-"), "{local}");
}

#[test]
fn the_scope_form_is_refused_naming_the_identifier_form_and_that_form_answers() {
    let f = fixture("peer-ids-scope-form");
    let before = snapshot(&f.b);
    let err = f.refused(&["show", &format!("bb:{TASK}")]);
    let named = hint(&err);
    assert_eq!(named, format!("ank show {TASK}@bb"), "{err}");
    let args = argv(&named, "");
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let shown = f.ok(&refs);
    assert!(shown.contains(&format!("id: {TASK}")), "{shown}");
    assert_eq!(snapshot(&f.b), before);
}

#[test]
fn an_undeclared_peer_is_refused_naming_the_declaration_and_the_declaration_settles_it() {
    let f = fixture("peer-ids-undeclared");
    let at = format!("{TASK}@zz");
    for verb in ["show", "find", "log"] {
        let err = f.refused(&[verb, &at]);
        let named = hint(&err);
        assert_eq!(named, "ank config peers.zz <path>", "{verb}: {err}");
    }
    let args = argv("ank config peers.zz <path>", "../b");
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    f.ok(&refs);
    let shown = f.ok(&["show", &at]);
    assert!(shown.contains(&format!("id: {TASK}")), "{shown}");
}

#[test]
fn an_unreadable_peer_is_refused_naming_it_and_the_command_that_settles_it() {
    let f = fixture("peer-ids-unreadable");
    let at = format!("{TASK}@gone");
    let mut last = String::new();
    for verb in ["show", "find", "log"] {
        let err = f.refused(&[verb, &at]);
        assert!(err.contains("peer 'gone'"), "{verb} names the peer: {err}");
        last = hint(&err);
    }
    // Whatever it names, run with a path that exists, it must not fail the
    // same way: the identifier is read afterwards.
    let args = argv(&last, "../b");
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    f.ok(&refs);
    let shown = f.ok(&["show", &at]);
    assert!(shown.contains(&format!("id: {TASK}")), "{shown}");
}

#[test]
fn an_identifier_the_peer_does_not_hold_names_a_command_that_answers() {
    let f = fixture("peer-ids-missing");
    let err = f.refused(&["show", "TASK-ffffffffffff@bb"]);
    let named = hint(&err);
    // Not `ank find TASK-ffff…@bb`, which would answer an empty page: the
    // listing of what the peer does hold.
    let args = argv(&named, "");
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let listed = f.ok(&refs);
    assert!(listed.contains("@bb"), "{named} lists the peer: {listed}");
}
