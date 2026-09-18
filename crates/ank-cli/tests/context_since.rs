//! `context --since`: what moved since the holder last worked on the task it
//! holds, with the lease as the only cursor (ADR-894d4bfbf9bd).
//!
//! **Through the binary, in one repository with two worktrees and two
//! identities**, because that is the shape the question is asked from: one
//! agent holding a task, another working beside it, and the claims plane shared
//! between them by `refs/ank/`.
//!
//! The cursor is second-grained: it is `expires - ttl`, and both are written to
//! the second. So "before the cursor" is arranged by crossing a second boundary
//! and never by waiting on a duration: an entity edited in the same second as
//! the renewal is listed, which is the direction the decision chose, and a test
//! that wants it absent has to put a whole second between the two.
//!
//! Process counts are read from `GIT_TRACE` at an absolute path and never from a
//! wall clock (CLAUDE.md).

mod fixture;
mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// What git prefixes the argument list of every git it runs, under `GIT_TRACE`.
const MARK: &str = "trace: built-in: git ";

const HOLDER: &str = "claude-code@holder";
const OTHER: &str = "claude-code@other";

/// The task the holder claims.
const T: &str = "TASK-000000000001";
/// Edited after the holder's last work: listed.
const X: &str = "TASK-000000000002";
/// Claimed by the other identity after the holder's last work: listed.
const Y: &str = "TASK-000000000003";
/// Edited before the holder's last work: not listed.
const Z: &str = "TASK-000000000004";
/// Finished by the other identity after the holder's last work: listed.
const W: &str = "TASK-000000000005";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("context-since-gitconfig");
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
        .env("GIT_CONFIG_SYSTEM", config);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = spawn("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// One repository, two checkouts of it: the holder's and the other agent's.
struct Pair {
    main: PathBuf,
    second: PathBuf,
}

impl Pair {
    fn new(what: &str) -> Pair {
        let base = scratch::dir(what);
        let main = base.join("main");
        let second = base.join("second");
        fs::create_dir_all(main.join(".ank/entities")).unwrap();
        git(&main, &["init", "-q", "-b", "main"]);
        fs::write(
            main.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(main.join(".gitignore"), ".ank/index.db\n").unwrap();
        for (id, title) in [
            (T, "The task in hand"),
            (X, "Edited after the last work"),
            (Y, "Claimed beside it"),
            (Z, "Edited before the last work"),
            (W, "Finished beside it"),
        ] {
            fs::write(
                main.join(".ank/entities").join(format!("{id}.md")),
                format!(
                    "---\nid: {id}\ntype: task\nslug: example\ntitle: {title}\n\
                     created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                     blocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
                     criteria_by: creator\nschema: 1\nversion: 1\n---\n\nFree body.\n"
                ),
            )
            .unwrap();
        }
        git(&main, &["add", "-A"]);
        git(&main, &["commit", "-qm", "seed"]);
        git(
            &main,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "other",
                second.to_str().unwrap(),
            ],
        );
        Pair { main, second }
    }

    fn ank_at(&self, at: &Path, agent: &str, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .arg("--repo")
            .arg(at)
            .env("ANK_AGENT", agent)
            .current_dir(scratch::root())
            .output()
            .expect("the binary must have been built")
    }

    /// The holder, in its own checkout.
    fn holder(&self, args: &[&str]) -> Output {
        self.ank_at(&self.main, HOLDER, args)
    }

    /// The other agent, in the other checkout.
    fn other(&self, args: &[&str]) -> Output {
        self.ank_at(&self.second, OTHER, args)
    }

    fn ok(out: Output, what: &str) -> Output {
        assert_eq!(out.status.code(), Some(0), "{what}: {}", stderr(&out));
        out
    }

    /// The claim record on `id`, read with git and not through the binary.
    fn record(&self, id: &str) -> String {
        git(
            &self.main,
            &["cat-file", "-p", &format!("refs/ank/claims/{id}")],
        )
    }

    fn field(&self, id: &str, key: &str) -> String {
        let record = self.record(id);
        record
            .lines()
            .find_map(|l| l.strip_prefix(&format!("{key}: ")))
            .unwrap_or_else(|| panic!("no {key} in {record}"))
            .to_string()
    }

    /// Rewrites the claim's expiry and leaves every other byte alone: the
    /// forgery `tests/cli.rs` performs, one clock read away from waiting.
    fn set_expiry(&self, id: &str, when: &str) {
        let rewritten: String = self
            .record(id)
            .lines()
            .map(|l| {
                if l.starts_with("expires: ") {
                    format!("expires: {when}\n")
                } else {
                    format!("{l}\n")
                }
            })
            .collect();
        let blob = {
            let mut child = spawn("git")
                .args(["hash-object", "-w", "--stdin"])
                .current_dir(&self.main)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            use std::io::Write;
            child
                .stdin
                .take()
                .unwrap()
                .write_all(rewritten.as_bytes())
                .unwrap();
            let out = child.wait_with_output().unwrap();
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        git(
            &self.main,
            &["update-ref", &format!("refs/ank/claims/{id}"), &blob],
        );
    }
}

impl Drop for Pair {
    fn drop(&mut self) {
        if let Some(base) = self.main.parent() {
            let _ = fs::remove_dir_all(base);
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Returns once the wall clock is in a later second than it was on entry.
///
/// Not a timing wall: nothing is asserted about how long anything took. It is
/// the one ordering a second-grained cursor can see, and it waits at most one
/// second.
fn cross_a_second() {
    let start = now_secs();
    while now_secs() == start {
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn utc(secs: u64) -> String {
    // Days from the epoch to a civil date (Howard Hinnant's algorithm), so the
    // forged expiry is written in the record's own format without a crate.
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

fn ids_under(doc: &serde_yaml::Value, key: &str) -> Vec<String> {
    doc[key]
        .as_sequence()
        .unwrap_or_else(|| panic!("no array under {key}: {doc:?}"))
        .iter()
        .map(|row| {
            row["id"]
                .as_str()
                .expect("a row carries its id")
                .to_string()
        })
        .collect()
}

/// Criterion (1): an entity edited after the holder's last work and a claim
/// taken in the other checkout after it are named; an entity edited before it
/// is not.
#[test]
fn since_names_what_moved_after_the_holders_last_work_and_nothing_before() {
    let p = Pair::new("since-moved");
    Pair::ok(p.holder(&["claim", T, "--ttl", "2h"]), "claim T");
    Pair::ok(
        p.holder(&["amend", Z, "--scope", "src/z/**"]),
        "edit Z before the log",
    );
    cross_a_second();
    Pair::ok(p.holder(&["log", T, "the last work"]), "log T");
    Pair::ok(
        p.holder(&["amend", X, "--scope", "src/x/**"]),
        "edit X after the log",
    );
    Pair::ok(p.other(&["claim", Y]), "claim Y in the other checkout");

    let out = Pair::ok(
        p.holder(&["context", "--since", "--json"]),
        "context --since",
    );
    let doc: serde_yaml::Value = serde_yaml::from_str(&stdout(&out))
        .unwrap_or_else(|e| panic!("not JSON: {e}\n{}", stdout(&out)));
    let entities = ids_under(&doc, "entities");
    let claims = ids_under(&doc, "claims");
    assert!(
        entities.contains(&X.to_string()),
        "{X} was edited after the last work and is not named: {entities:?}"
    );
    assert!(
        claims.contains(&Y.to_string()),
        "{Y} was claimed after the last work and is not named: {claims:?}"
    );
    assert!(
        !stdout(&out).contains(Z),
        "{Z} was edited before the last work and is named: {}",
        stdout(&out)
    );
}

/// Criterion (2): `--since` is the holder's work on the task it holds, so it
/// renews the lease; and a plain `context` under the same claim still renews
/// it (ADR-0bb7ea8991bc).
///
/// The expiry is forged a few minutes ahead and the renewal is what carries it
/// back to two hours: a renewal in the same second as the claim would be
/// invisible, and nothing here waits for a clock.
#[test]
fn since_renews_the_lease_and_plain_context_still_does() {
    let p = Pair::new("since-renews");
    Pair::ok(p.holder(&["claim", T, "--ttl", "2h"]), "claim T");

    let near = utc(now_secs() + 600);
    p.set_expiry(T, &near);
    Pair::ok(p.holder(&["context", "--since"]), "context --since");
    let after = p.field(T, "expires");
    assert!(
        after > near,
        "context --since did not renew the lease: {near} before, {after} after"
    );

    p.set_expiry(T, &near);
    Pair::ok(p.holder(&["context"]), "context");
    let after = p.field(T, "expires");
    assert!(
        after > near,
        "a plain context no longer renews the lease: {near} before, {after} after"
    );
}

/// Criterion (3): no claim, no lease, no cursor, and the refusal names the
/// command that takes one.
#[test]
fn since_without_a_claim_is_refused_with_the_command_that_takes_one() {
    let p = Pair::new("since-unclaimed");
    let out = p.holder(&["context", "--since"]);
    assert_ne!(
        out.status.code(),
        Some(0),
        "context --since answered with no claim held: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("ank claim"),
        "the refusal does not name ank claim: {}",
        stderr(&out)
    );
}

/// Criterion (4): the `--json` document of `context --since` is pinned by a
/// golden.
///
/// The cursor is placed by a plain `context` a whole second after everything
/// the fixture wrote to set itself up, so what is named is exactly what moved
/// after it: one edit (the entity and the entry that records the edit), one
/// claim and one completion from the other checkout.
#[test]
fn the_since_document_is_pinned_by_a_golden() {
    let p = Pair::new("since-golden");
    Pair::ok(p.holder(&["claim", T, "--ttl", "2h"]), "claim T");
    cross_a_second();
    Pair::ok(p.holder(&["context"]), "context, which places the cursor");
    // The file itself, and not `amend`: `amend` also mints the entry recording
    // the edit, whose short form is four hex digits no redaction can tell from
    // a word, so the golden would pin a different name on every run. A file
    // changing is what the entity half of the answer measures either way.
    let x = p.main.join(".ank/entities").join(format!("{X}.md"));
    let text = fs::read_to_string(&x).unwrap();
    fs::write(&x, text.replace("Free body.", "Free body, edited.")).unwrap();
    Pair::ok(p.other(&["claim", Y]), "claim Y");
    // A third identity finishes W, because `claim` refuses one identity a
    // second live claim and Y's has to stay live to be named.
    Pair::ok(
        p.ank_at(&p.second, "claude-code@third", &["claim", W]),
        "claim W",
    );
    let head = git(&p.second, &["rev-parse", "HEAD"]);
    Pair::ok(
        p.ank_at(
            &p.second,
            "claude-code@third",
            &["done", W, "--proof", &format!("commit:{head}")],
        ),
        "finish W",
    );

    let out = Pair::ok(
        p.holder(&["context", "--since", "--json"]),
        "context --since",
    );
    fixture::pin("context-since", &stdout(&out));
}

/// Criterion (5): `--since` reads the planes `context` already opens, so it
/// starts exactly as many git processes (ADR-cc65f1388a71).
///
/// Both runs renew a record the forgery moved, so the renewal each pays for is
/// the same write and the only difference between the two is the flag.
#[test]
fn since_starts_as_many_git_processes_as_context() {
    let p = Pair::new("since-count");
    Pair::ok(p.holder(&["claim", T, "--ttl", "2h"]), "claim T");
    Pair::ok(p.other(&["claim", Y]), "claim Y");

    let count = |args: &[&str], name: &str| -> usize {
        p.set_expiry(T, &utc(now_secs() + 600));
        let trace = scratch::root().join(format!("since-count-{name}.trace"));
        let _ = fs::remove_file(&trace);
        let out = spawn(ANK)
            .args(args)
            .arg("--repo")
            .arg(&p.main)
            .env("ANK_AGENT", HOLDER)
            .env("GIT_TRACE", &trace)
            .current_dir(scratch::root())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0), "{args:?}: {}", stderr(&out));
        let text = fs::read_to_string(&trace).unwrap_or_default();
        let n = text.lines().filter(|l| l.contains(MARK)).count();
        assert!(n > 0, "the trace records no git process: {text:.400}");
        n
    };
    let plain = count(&["context"], "plain");
    let since = count(&["context", "--since"], "since");
    assert_eq!(
        plain, since,
        "context --since starts {since} git processes where context starts {plain}"
    );
}
