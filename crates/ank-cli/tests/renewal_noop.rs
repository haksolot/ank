//! A renewal that changes nothing writes nothing (TASK-43c2e64d1d30).
//!
//! `context` under a claim renews it, and the record is stamped to the second,
//! so a renewal landing in the second the claim was written computes the record
//! already on the ref. It used to be written anyway: `hash-object`,
//! `update-ref` of a sha onto itself, and at level 1 a push of bytes the remote
//! already had (LOG-f21bd3d43750 on TASK-24ea4fbba3df).
//!
//! **Level 1, through the binary**, over a bare origin reached by `file://`:
//! the push is the cost the task is about, and a fixture without a remote would
//! measure a renewal that never pushes either way.
//!
//! Process counts are read from `GIT_TRACE` at an absolute path and never from a
//! wall clock (CLAUDE.md). The one wait here is for a second to turn, which is a
//! wait on state: nothing is asserted about how long anything took.

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// What git prefixes the argument list of every git it runs, under `GIT_TRACE`.
const MARK: &str = "trace: built-in: git ";

const AGENT: &str = "claude-code@holder";
const T: &str = "TASK-000000000001";

/// The TTL every claim below is taken with, in seconds, and the `--ttl`
/// spelling of it. A renewal computes `now + ttl`, so the test needs the number
/// to arrange the expiry a renewal in a chosen second will write.
const TTL_SECONDS: u64 = 7_200;
const TTL: &str = "2h";

/// How many seconds a verb is allowed to take between the instant the test
/// names and the instant the binary stamps. Only the format guard uses it; the
/// arrangement below does not depend on a duration.
const TOLERATED_SECONDS: u64 = 3;

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("renewal-noop-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n[core]\n\tautocrlf = false\n\
             [protocol \"file\"]\n\tallow = always\n",
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

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Returns once the wall clock is in a later second than it was on entry.
fn cross_a_second() {
    let start = now_secs();
    while now_secs() == start {
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Returns at the start of `second`, having spent the wait asleep in short
/// steps so the return lands near its beginning rather than anywhere in it.
fn wait_for_second(second: u64) {
    while now_secs() < second {
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// `YYYY-MM-DDThh:mm:ssZ`, the stamp `claim` writes into a record.
///
/// Restated here because the binary exposes no library to borrow it from, and
/// a restated format is a format free to drift -- so
/// [`Level1::assert_stamp_matches_the_binary`] confronts it with one the binary
/// wrote before anything rests on it.
fn rfc3339(epoch_secs: u64) -> String {
    let days = (epoch_secs / 86_400) as i64;
    let secs = epoch_secs % 86_400;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// A repository with a corpus of one task, and a bare origin it pushes to.
struct Level1 {
    base: PathBuf,
    repo: PathBuf,
}

impl Level1 {
    fn new(what: &str) -> Level1 {
        let base = scratch::dir(what);
        let repo = base.join("repo");
        let origin = base.join("origin.git");
        fs::create_dir_all(repo.join(".ank/entities")).unwrap();
        fs::create_dir_all(&origin).unwrap();
        git(&origin, &["init", "-q", "--bare", "-b", "main"]);
        git(&repo, &["init", "-q", "-b", "main"]);
        fs::write(
            repo.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(repo.join(".gitignore"), ".ank/index.db\n").unwrap();
        fs::write(
            repo.join(".ank/entities").join(format!("{T}.md")),
            format!(
                "---\nid: {T}\ntype: task\nslug: example\ntitle: The task in hand\n\
                 created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                 blocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
                 criteria_by: creator\nschema: 1\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-qm", "seed"]);
        // `file:///C:/...` on Windows, `file:///tmp/...` elsewhere.
        let path = origin.to_str().unwrap().replace('\\', "/");
        let url = if path.starts_with('/') {
            format!("file://{path}")
        } else {
            format!("file:///{path}")
        };
        git(&repo, &["remote", "add", "origin", &url]);
        git(&repo, &["push", "-q", "origin", "main"]);
        Level1 { base, repo }
    }

    fn ank(&self, args: &[&str], trace: Option<&Path>) -> Output {
        let mut c = spawn(ANK);
        c.args(args)
            .arg("--repo")
            .arg(&self.repo)
            .env("ANK_AGENT", AGENT)
            .current_dir(scratch::root());
        if let Some(t) = trace {
            c.env("GIT_TRACE", t);
        }
        c.output().expect("the binary must have been built")
    }

    fn ok(&self, args: &[&str]) {
        let out = self.ank(args, None);
        assert_eq!(out.status.code(), Some(0), "{args:?}: {}", stderr(&out));
    }

    fn claim_ref(&self) -> String {
        format!("refs/ank/claims/{T}")
    }

    fn record(&self) -> String {
        git(&self.repo, &["cat-file", "-p", &self.claim_ref()])
    }

    /// `ank context` under the claim, traced; the git command lines it started.
    fn traced_context(&self, name: &str) -> Vec<String> {
        let trace = self.base.join(format!("{name}.trace"));
        let _ = fs::remove_file(&trace);
        let out = self.ank(&["context"], Some(&trace));
        assert_eq!(out.status.code(), Some(0), "context: {}", stderr(&out));
        let text = fs::read_to_string(&trace).unwrap_or_default();
        let lines: Vec<String> = text
            .lines()
            .filter_map(|l| l.split_once(MARK).map(|(_, rest)| rest.to_string()))
            .collect();
        assert!(!lines.is_empty(), "the trace records no git process");
        lines
    }

    /// The `expires:` stamp the record currently carries.
    fn expiry(&self) -> String {
        self.record()
            .lines()
            .find_map(|l| l.strip_prefix("expires: ").map(str::to_string))
            .expect("a claim record carries an expiry")
    }

    /// Confronts [`rfc3339`] with a stamp the binary wrote, so nothing below
    /// rests on a format this file only believes it shares.
    ///
    /// The renewal that produced the record happened somewhere inside the
    /// second the caller names or the one after it -- the verb is what sits
    /// between -- so both are admitted, and a format that differs in any other
    /// way matches neither.
    fn assert_stamp_matches_the_binary(&self, at_or_after: u64) {
        let written = self.expiry();
        let candidates: Vec<String> = (0..=TOLERATED_SECONDS)
            .map(|d| rfc3339(at_or_after + d + TTL_SECONDS))
            .collect();
        assert!(
            candidates.contains(&written),
            "the binary writes an expiry this file cannot reproduce: \
             wrote {written}, this file formats {candidates:?}"
        );
    }

    /// Moves the claim's expiry on the ref and on the origin alike, so the next
    /// renewal changes the record and its push leases on what the origin holds.
    fn forge_expiry_ahead(&self) {
        self.forge_expiry("2099-01-01T00:00:00Z");
    }

    /// The same route, to a stamp of the caller's choosing: the expiry a
    /// renewal in a chosen second computes, so the identical-record case is
    /// arranged instead of waited for.
    fn forge_expiry(&self, stamp: &str) {
        let rewritten: String = self
            .record()
            .lines()
            .map(|l| {
                if l.starts_with("expires: ") {
                    format!("expires: {stamp}\n")
                } else {
                    format!("{l}\n")
                }
            })
            .collect();
        let mut child = spawn("git")
            .args(["hash-object", "-w", "--stdin"])
            .current_dir(&self.repo)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
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
        let blob = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let name = self.claim_ref();
        git(&self.repo, &["update-ref", &name, &blob]);
        git(
            &self.repo,
            &["push", "-q", "-f", "origin", &format!("{name}:{name}")],
        );
    }
}

impl Drop for Level1 {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

/// The processes of a traced `context` that came before its renewal. The
/// renewal is the last thing the dispatch does, after the verb, and its first
/// process is the blob it writes: everything before that line is the verb.
fn before_the_renewal(lines: &[String]) -> usize {
    lines
        .iter()
        .position(|l| l.starts_with("hash-object"))
        .unwrap_or(lines.len())
}

fn names(lines: &[String]) -> String {
    lines
        .iter()
        .map(|l| l.split_whitespace().next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The criterion, both halves, at level 1.
///
/// A renewal whose record is byte-identical to the one on the ref starts no
/// write and no push, so `context` costs what it costs without a renewal; a
/// renewal that changes the record still writes and still pushes.
#[test]
fn a_renewal_that_changes_nothing_writes_nothing_and_pushes_nothing() {
    let r = Level1::new("renewal-noop");

    // A renewal writes `now + ttl` and is a no-op when the record already says
    // so. Racing the claim for that was what made this test fail on a slow
    // runner for a reason it does not measure (TASK-faf554e366c5): what decided
    // each attempt was the phase of the second at which `claim` happened to
    // write, which nothing aligned.
    //
    // So the state is arranged. The expiry a renewal in a named second computes
    // is written onto the ref and the origin first, and the second is then
    // entered at its start, which leaves a whole second for one `context` to
    // reach its renewal. The loop remains because a verb can still be slower
    // than that, and it is now the unlikely case rather than the usual one.
    cross_a_second();
    r.ok(&["claim", T, "--ttl", TTL]);
    let claimed_at = now_secs();
    let _ = r.traced_context("warm");
    // Nothing below is trusted until this file and the binary are shown to
    // write the same twenty characters.
    r.assert_stamp_matches_the_binary(claimed_at);

    let mut identical = None;
    for _ in 0..10 {
        let target = now_secs() + 2;
        r.forge_expiry(&rfc3339(target + TTL_SECONDS));
        wait_for_second(target);
        let before = r.record();
        let lines = r.traced_context("identical");
        if r.record() == before {
            identical = Some(lines);
            break;
        }
    }
    let identical = identical
        .expect("ten arranged seconds never held a whole context: the verb takes over a second");

    r.forge_expiry_ahead();
    let changing = r.traced_context("changing");

    // Printed for the record the task asks for (`--nocapture`).
    eprintln!(
        "git processes of context at level 1: unchanged renewal {} ({}), \
         changing renewal {} ({}), verb before its renewal {}",
        identical.len(),
        names(&identical),
        changing.len(),
        names(&changing),
        before_the_renewal(&changing)
    );

    // The renewal that changes the record still does all of it.
    for step in ["hash-object", "update-ref", "push"] {
        assert!(
            changing.iter().any(|l| l.starts_with(step)),
            "a renewal that changes the record no longer runs {step}: {}",
            names(&changing)
        );
    }
    // And the one that changes nothing does none of it.
    for step in ["hash-object", "update-ref", "push"] {
        assert!(
            !identical.iter().any(|l| l.starts_with(step)),
            "a renewal that changes nothing still runs {step}: {} processes, {}",
            identical.len(),
            names(&identical)
        );
    }
    assert_eq!(
        identical.len(),
        before_the_renewal(&changing),
        "context under an unchanged renewal costs {} git processes where the \
         verb without its renewal costs {} (a changing renewal: {} in all)",
        identical.len(),
        before_the_renewal(&changing),
        changing.len()
    );
}
