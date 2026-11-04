//! `check` prints what is owed and counts what is only known
//! (TASK-519112db3aa5, ADR-61c273118d97).
//!
//! Every case through the binary, on scratch corpora that hold, for each way a
//! finding comes to be owed, one finding that is and -- where the rule can
//! have one -- one that is not:
//!
//! - a fault, always owed;
//! - a signal about its subject (`scope-not-yet` on an open task, owed, against
//!   `scope-closed` on a closed one, owing nothing; `read-by-no-human` on an
//!   open task against the same on closed ones, folded within its half);
//! - a signal naming the command that clears it (`previous-layout`, whose
//!   subject is the corpus and settles nothing), owed;
//! - a `burst`, owed while one entity it counts is unsettled, and owing
//!   nothing once every one of them is closed.
//!
//! The predicate of every kind is declared in one table, and the test that a
//! kind without one fails is a unit test beside that table in `human.rs`.

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
        let p = scratch::path("check-owed-gitconfig");
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
        .env_remove("NO_COLOR")
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

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn task_id(n: usize) -> String {
    format!("TASK-{n:012x}")
}

struct Repo(PathBuf);

/// The open task whose scope nothing matches yet: owed, by its subject.
const NOT_YET: usize = 0x1;
/// The closed task whose scope nothing matches: owes nothing.
const CLOSED: usize = 0x2;
/// The task in the previous layout: the signal names `git mv`, so it is owed
/// although its subject is the corpus.
const LEGACY: usize = 0x3;
/// The one open task among the eleven of the owed burst.
const LIVE_OPEN: usize = 0x200;
/// A task blocked by one the corpus does not hold: a fault.
const BLOCKED: usize = 0x4;

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
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n\
             verifiers:\n  t:\n    run: \"true\"\n",
        )
        .unwrap();
        fs::write(r.0.join(".gitignore"), ".ank/index.db*\n").unwrap();
        r
    }

    /// The corpus every case reads: one owed and one unowed finding per way a
    /// signal comes to be owed, and no fault.
    fn seeded(what: &str) -> Repo {
        let r = Repo::new(what);
        r.task(
            ".ank/entities",
            NOT_YET,
            "human:marie",
            "open",
            "src/later/**",
            "2026-07-01T00:00:00Z",
            "",
        );
        r.task(
            ".ank/entities",
            CLOSED,
            "human:marie",
            "closed",
            "src/gone/**",
            "2026-07-01T00:00:00Z",
            "",
        );
        fs::create_dir_all(r.0.join(".ank/tasks")).unwrap();
        r.task(
            ".ank/tasks",
            LEGACY,
            "human:marie",
            "open",
            "src/**",
            "2026-07-01T00:00:00Z",
            "",
        );
        // Eleven in an hour, every one closed: a burst that owes nothing.
        for n in 0..11 {
            r.task(
                ".ank/entities",
                0x100 + n,
                "claude-code/bulk",
                "closed",
                "src/**",
                &format!("2026-07-10T10:{:02}:00Z", n * 2),
                "",
            );
        }
        // Eleven in an hour, one still open: a burst that is owed.
        for n in 0..11 {
            let status = if n == 0 { "open" } else { "closed" };
            r.task(
                ".ank/entities",
                LIVE_OPEN + n,
                "claude-code/live",
                status,
                "src/**",
                &format!("2026-07-12T10:{:02}:00Z", n * 2),
                "",
            );
        }
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

    fn commit(&self) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", "seed", "--allow-empty"]);
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

    #[allow(clippy::too_many_arguments)]
    fn task(
        &self,
        dir: &str,
        n: usize,
        author: &str,
        status: &str,
        scope: &str,
        created: &str,
        blocked_by: &str,
    ) {
        let id = task_id(n);
        fs::write(
            self.0.join(dir).join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\nslug: example\ntitle: Example\n\
                 created: {created}\nauthor: {author}\nstatus: {status}\n\
                 scope:\n  - {scope}\nblocked_by: [{blocked_by}]\n\
                 done_criteria: |\n  A verifiable criterion.\ncriteria_by: creator\n\
                 schema: 4\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
    }
}

fn line_about(said: &str, subject: &str, words: &str) -> bool {
    said.lines()
        .any(|l| l.contains(&format!(": {subject}: ")) && l.contains(words))
}

/// The one line counting what owes nothing, and the number it states.
fn unowed_line(said: &str) -> Option<(usize, String)> {
    let line = said
        .lines()
        .find(|l| l.contains("owe nothing") && l.contains("ank check --all"))?;
    let n = line
        .split_whitespace()
        .next()?
        .parse()
        .unwrap_or_else(|_| panic!("the line opens with the count: {line}"));
    Some((n, line.to_string()))
}

/// Each finding object of a `check --json` document, as its text.
fn findings(json: &str) -> Vec<String> {
    json.split("{\"level\":")
        .skip(1)
        .map(|s| format!("{{\"level\":{s}"))
        .collect()
}

/// The value under `"<key>":` in a flat document, as a number.
fn num(json: &str, key: &str) -> usize {
    let at = json.find(&format!("\"{key}\":")).expect(key) + key.len() + 3;
    json[at..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap()
}

fn owed_of(found: &[String], subject: &str, words: &str) -> bool {
    let f = found
        .iter()
        .find(|f| f.contains(&format!("\"subject\":\"{subject}\"")) && f.contains(words))
        .unwrap_or_else(|| panic!("no finding on {subject} saying {words}:\n{found:#?}"));
    if f.contains("\"owed\":true") {
        true
    } else if f.contains("\"owed\":false") {
        false
    } else {
        panic!("no owed on {f}")
    }
}

#[test]
fn check_prints_what_is_owed_and_counts_the_rest_in_one_line() {
    let r = Repo::seeded("owed-default");
    r.commit();
    let out = r.ank(&["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let said = stdout(&out);

    // Owed: by its subject, by its remedy, by one entity a burst counts.
    assert!(
        line_about(&said, &task_id(NOT_YET), "matches no file yet"),
        "{said}"
    );
    assert!(line_about(&said, "corpus", "previous layout"), "{said}");
    assert!(
        line_about(&said, "corpus", "burst creation by claude-code/live"),
        "{said}"
    );
    assert!(
        line_about(&said, &task_id(LIVE_OPEN), "read by no human"),
        "{said}"
    );
    // Owing nothing: not printed.
    assert!(
        !line_about(&said, &task_id(CLOSED), "matches no file"),
        "{said}"
    );
    assert!(
        !said.contains("burst creation by claude-code/bulk"),
        "{said}"
    );
    assert!(
        !said.contains(&format!(": {}: ", task_id(0x101))),
        "a closed task read by no human owes nothing:\n{said}"
    );
    // And counted, once, naming the command that lists them.
    let (n, line) = unowed_line(&said).unwrap_or_else(|| panic!("no count line:\n{said}"));
    assert!(n >= 3, "scope-closed, a burst, read-by-no-human: {line}");
    assert_eq!(
        said.lines().filter(|l| l.contains("owe nothing")).count(),
        1,
        "{said}"
    );

    // What the count says is what --json says.
    let json = stdout(&r.ank(&["check", "--json"]));
    assert_eq!(num(&json, "unowed"), n, "{json}");
}

#[test]
fn check_all_prints_every_finding_and_kind_prints_one_kind_whole() {
    let r = Repo::seeded("owed-all");
    r.commit();
    let out = r.ank(&["check", "--all"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let all = stdout(&out);
    for (subject, words) in [
        (task_id(NOT_YET), "matches no file yet"),
        (task_id(CLOSED), "matches no file, and the task is closed"),
        ("corpus".to_string(), "previous layout"),
        ("corpus".to_string(), "burst creation by claude-code/live"),
        ("corpus".to_string(), "burst creation by claude-code/bulk"),
    ] {
        assert!(
            line_about(&all, &subject, words),
            "{subject} {words}:\n{all}"
        );
    }
    assert!(
        unowed_line(&all).is_none(),
        "--all hides nothing to count:\n{all}"
    );

    // --kind prints that kind whole, owed or not.
    let out = r.ank(&["check", "--kind", "scope-closed"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        line_about(&stdout(&out), &task_id(CLOSED), "the task is closed"),
        "{}",
        stdout(&out)
    );
    let out = r.ank(&["check", "--kind", "burst"]);
    let bursts = stdout(&out);
    assert!(bursts.contains("claude-code/bulk"), "{bursts}");
    assert!(bursts.contains("claude-code/live"), "{bursts}");
    let out = r.ank(&["check", "--kind", "read-by-no-human"]);
    assert!(
        line_about(&stdout(&out), &task_id(0x101), "read by no human"),
        "per entity, owed or not:\n{}",
        stdout(&out)
    );
}

#[test]
fn json_carries_owed_on_every_finding() {
    let r = Repo::seeded("owed-json");
    r.commit();
    let out = r.ank(&["check", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let json = stdout(&out);
    let found = findings(&json);
    assert!(!found.is_empty(), "{json}");
    for f in &found {
        assert!(
            f.contains("\"owed\":true") || f.contains("\"owed\":false"),
            "owed on every finding: {f}"
        );
    }
    assert!(owed_of(&found, &task_id(NOT_YET), "matches no file yet"));
    assert!(!owed_of(&found, &task_id(CLOSED), "the task is closed"));
    assert!(owed_of(&found, "corpus", "previous layout"));
    assert!(owed_of(&found, "corpus", "claude-code/live"));
    assert!(!owed_of(&found, "corpus", "claude-code/bulk"));
    assert_eq!(
        num(&json, "unowed"),
        found
            .iter()
            .filter(|f| f.contains("\"level\":\"signal\"") && f.contains("\"owed\":false"))
            .count(),
        "{json}"
    );
}

#[test]
fn a_fault_is_always_owed_and_the_exit_code_never_moves() {
    let r = Repo::seeded("owed-fault");
    r.task(
        ".ank/entities",
        BLOCKED,
        "human:marie",
        "open",
        "src/**",
        "2026-07-01T00:00:00Z",
        &task_id(0xfff000),
    );
    r.commit();
    for args in [
        vec!["check"],
        vec!["check", "--all"],
        vec!["check", "--kind", "scope-closed"],
        vec!["check", "--json"],
        vec!["check", "--all", "--json"],
    ] {
        let out = r.ank(&args);
        assert_eq!(code(&out), 8, "{args:?}: {}{}", stdout(&out), stderr(&out));
    }
    let said = stdout(&r.ank(&["check"]));
    assert!(
        said.lines()
            .any(|l| l.starts_with(&format!("error: {}: ", task_id(BLOCKED)))),
        "{said}"
    );
    let json = stdout(&r.ank(&["check", "--json"]));
    let found = findings(&json);
    assert!(owed_of(&found, &task_id(BLOCKED), "does not exist"));

    // And a healthy corpus exits 0 whatever is shown.
    let r = Repo::seeded("owed-exit-clean");
    r.commit();
    for args in [
        vec!["check"],
        vec!["check", "--all"],
        vec!["check", "--json"],
    ] {
        assert_eq!(code(&r.ank(&args)), 0, "{args:?}");
    }
}

#[test]
fn status_and_review_count_owed_signals_and_name_the_unowed_beside_them() {
    let r = Repo::seeded("owed-readers");
    r.commit();
    let check = stdout(&r.ank(&["check", "--json"]));
    let signals = num(&check, "signals");
    let unowed = num(&check, "unowed");
    assert!(unowed > 0 && unowed < signals, "{check}");
    let owed = signals - unowed;

    for verb in ["status", "review"] {
        let out = r.ank(&[verb, "--json"]);
        assert_eq!(code(&out), 0, "{verb}: {}", stderr(&out));
        let json = stdout(&out);
        assert_eq!(num(&json, "signals"), signals, "{verb}: {json}");
        assert_eq!(num(&json, "unowed"), unowed, "{verb}: {json}");

        let out = r.ank(&[verb]);
        let said = stdout(&out);
        assert!(
            said.contains(&format!("{owed} signal(s), {unowed} owing nothing")),
            "{verb}:\n{said}"
        );
    }
    // Twice, so the second status reads the verdict the first cached.
    let said = stdout(&r.ank(&["status"]));
    assert!(
        said.contains(&format!("{owed} signal(s), {unowed} owing nothing")),
        "the cached verdict:\n{said}"
    );
}
