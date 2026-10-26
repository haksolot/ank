//! A push refused before it reaches the remote's update (TASK-530f27a4aeb3,
//! ADR-92bb0266b376).
//!
//! The remote answers `ls-remote` and refuses the push before any ref is
//! updated: a missing credential, or a URL readable and not writable. The push
//! exits 128 with nothing on stdout, so there is no porcelain status line for
//! the ref, and the ref is still where the lease said it was. Measured before
//! the fix, every verb read that as a lost race: `claim` exited 4 naming the
//! caller as the holder, `log` warned "taken over" and fetched the remote record
//! over its own write, `done` exited 4 "moved" after writing the task `done`,
//! and `release` said `released` while both refs survived.
//!
//! Arranged with `remote.origin.pushurl` naming a repository that does not
//! exist while `remote.origin.url` names the real one: the shape of a public
//! repository and no credential, with no network and no credential helper, the
//! same on Linux, macOS and Windows.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "tool/1.0";
const T: &str = "TASK-000000000001";
const DIAGNOSE: &str = "git push --dry-run";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("push-refused-before-update-gitconfig");
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

fn spawn(program: impl AsRef<std::ffi::OsStr>) -> Command {
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

/// `file:///C:/...` on Windows, `file:///tmp/...` elsewhere.
fn file_url(path: &Path) -> String {
    let path = path.to_str().unwrap().replace('\\', "/");
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

/// Returns once the wall clock is in a later second than it was on entry, so
/// a renewal stamps an expiry the pushed record does not already carry.
fn cross_a_second() {
    let now = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };
    let start = now();
    while now() == start {
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// A corpus of one task with a bare origin that every read reaches.
struct Corpus {
    repo: PathBuf,
    origin: PathBuf,
}

impl Corpus {
    fn new(what: &str) -> Corpus {
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
        fs::write(repo.join(".gitignore"), ".ank/index.db*\n").unwrap();
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
        git(&repo, &["remote", "add", "origin", &file_url(&origin)]);
        git(
            &repo,
            &[
                "config",
                "--add",
                "remote.origin.fetch",
                "+refs/ank/*:refs/ank/*",
            ],
        );
        git(&repo, &["push", "-q", "origin", "main"]);
        Corpus { repo, origin }
    }

    /// The same corpus with the claim taken and on the origin.
    fn claimed(what: &str) -> Corpus {
        let c = Corpus::new(what);
        let out = c.ank(&["claim", T]);
        assert_eq!(out.status.code(), Some(0), "claim: {}", text(&out.stderr));
        assert_eq!(
            c.remote_claim(),
            Some(c.local_claim().unwrap()),
            "the claim reached the origin"
        );
        c
    }

    /// From here on the origin is read as before and refuses every push before
    /// it reaches an update: the push URL names a repository that is not there.
    fn refuse_pushes(&self) {
        let nowhere = self.origin.with_extension("nowhere");
        git(
            &self.repo,
            &["config", "remote.origin.pushurl", &file_url(&nowhere)],
        );
        let out = spawn("git")
            .args(["push", "--porcelain", "origin", "HEAD:refs/heads/probe"])
            .current_dir(&self.repo)
            .output()
            .unwrap();
        assert!(!out.status.success(), "the push was expected to fail");
        assert_eq!(
            text(&out.stdout).trim(),
            "",
            "the push was expected to print no status line, which is the case under test"
        );
    }

    fn ank(&self, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .arg("--repo")
            .arg(&self.repo)
            .env("ANK_AGENT", AGENT)
            .current_dir(scratch::root())
            .output()
            .expect("the binary must have been built")
    }

    fn claim_ref() -> String {
        format!("refs/ank/claims/{T}")
    }

    fn local_claim(&self) -> Option<String> {
        let out = spawn("git")
            .args(["rev-parse", "--verify", "--quiet", &Self::claim_ref()])
            .current_dir(&self.repo)
            .output()
            .unwrap();
        out.status
            .success()
            .then(|| text(&out.stdout).trim().to_string())
    }

    /// Read from the bare origin itself, so no fetch configuration stands
    /// between the question and the answer.
    fn remote_claim(&self) -> Option<String> {
        let out = spawn("git")
            .args(["rev-parse", "--verify", "--quiet", &Self::claim_ref()])
            .current_dir(&self.origin)
            .output()
            .unwrap();
        out.status
            .success()
            .then(|| text(&out.stdout).trim().to_string())
    }

    fn status_field(&self) -> String {
        let file = fs::read_to_string(self.repo.join(format!(".ank/entities/{T}.md"))).unwrap();
        file.lines()
            .find_map(|l| l.strip_prefix("status: "))
            .unwrap()
            .to_string()
    }

    /// Runs the diagnosing command the verb named, as named, in the clone: it
    /// must fail the way the push did, which is what makes it the diagnosis.
    fn diagnosis_reproduces(&self, said: &str) {
        let start = said
            .find(DIAGNOSE)
            .unwrap_or_else(|| panic!("no command that diagnoses the failure is named: {said}"));
        let command = said[start..]
            .split(|c| c == '`' || c == ')' || c == ';' || c == '\n')
            .next()
            .unwrap();
        let mut words = command.split_whitespace();
        assert_eq!(words.next(), Some("git"));
        let out = spawn("git")
            .args(words)
            .current_dir(&self.repo)
            .output()
            .unwrap();
        assert!(
            !out.status.success(),
            "`{command}` succeeded, so it does not reproduce the refused push"
        );
    }
}

/// Nothing a race would print: nobody holds the task, nobody moved it, nobody
/// took it over.
fn no_race_reported(verb: &str, said: &str) {
    for race in ["held by", "moved while", "taken over", "lost"] {
        assert!(
            !said.contains(race),
            "{verb} reported a race nobody ran ({race:?}): {said}"
        );
    }
}

#[test]
fn a_claim_refused_before_the_update_exits_9_naming_the_diagnosis_and_leaves_no_claim() {
    let c = Corpus::new("push-refused-claim");
    c.refuse_pushes();

    let out = c.ank(&["claim", T]);
    let said = format!("{}{}", text(&out.stdout), text(&out.stderr));
    assert_eq!(
        out.status.code(),
        Some(9),
        "a remote that refuses this clone's writes is an environment to repair:\n{said}"
    );
    no_race_reported("claim", &said);
    c.diagnosis_reproduces(&said);
    assert_eq!(
        c.local_claim(),
        None,
        "the clone kept a claim the remote refused"
    );
    assert_eq!(c.remote_claim(), None, "the origin was never written");
}

#[test]
fn a_renewal_refused_before_the_update_keeps_the_local_write_and_names_the_diagnosis() {
    let c = Corpus::claimed("push-refused-log");
    let pushed = c.remote_claim().unwrap();
    c.refuse_pushes();
    cross_a_second();

    let out = c.ank(&["log", "the push url refuses"]);
    let said = format!("{}{}", text(&out.stdout), text(&out.stderr));
    assert_eq!(out.status.code(), Some(0), "log wrote its entry:\n{said}");
    no_race_reported("log", &said);
    c.diagnosis_reproduces(&said);
    let local = c.local_claim().unwrap();
    assert_ne!(
        local, pushed,
        "the remote record was fetched over the renewal this clone wrote"
    );
    assert_eq!(
        c.remote_claim(),
        Some(pushed),
        "the origin was never written"
    );
}

#[test]
fn done_refused_before_the_update_completes_locally_and_names_the_diagnosis() {
    let c = Corpus::claimed("push-refused-done");
    let pushed = c.remote_claim().unwrap();
    c.refuse_pushes();
    let head = git(&c.repo, &["rev-parse", "HEAD"]);

    let out = c.ank(&["done", "--proof", &format!("commit:{head}")]);
    let said = format!("{}{}", text(&out.stdout), text(&out.stderr));
    assert_eq!(
        out.status.code(),
        Some(0),
        "done completes locally, as for an unreachable remote:\n{said}"
    );
    no_race_reported("done", &said);
    c.diagnosis_reproduces(&said);
    assert_eq!(c.status_field(), "done");
    let local = c.local_claim().unwrap();
    assert_ne!(
        local, pushed,
        "the remote claim was fetched over the completion record"
    );
    let record = git(&c.repo, &["cat-file", "-p", &local]);
    assert!(
        record.contains("state: completed"),
        "the local ref is not the completion record:\n{record}"
    );
    assert_eq!(
        c.remote_claim(),
        Some(pushed),
        "the origin was never written"
    );
}

#[test]
fn release_and_close_refused_before_the_update_drop_the_local_claim_and_name_the_diagnosis() {
    for (what, args) in [
        (
            "push-refused-release",
            vec!["release", "--reason", "the push url refuses"],
        ),
        (
            "push-refused-close",
            vec!["close", T, "--reason", "the push url refuses"],
        ),
    ] {
        let c = Corpus::claimed(what);
        let pushed = c.remote_claim().unwrap();
        c.refuse_pushes();

        let out = c.ank(&args);
        let said = format!("{}{}", text(&out.stdout), text(&out.stderr));
        assert_eq!(
            out.status.code(),
            Some(0),
            "{what}: the hand-back is on disk:\n{said}"
        );
        no_race_reported(what, &said);
        assert!(
            said.contains("still reads the task as held"),
            "{what}: silent while the origin kept the claim: {said}"
        );
        c.diagnosis_reproduces(&said);
        assert_eq!(
            c.local_claim(),
            None,
            "{what}: the remote claim was fetched back over the deletion"
        );
        assert_eq!(
            c.remote_claim(),
            Some(pushed),
            "{what}: the origin was never written"
        );
    }
}

/// The second write after the push started failing (TASK-37b528499ed8). The
/// first renewal stays local, so the next one leases on an object the origin
/// never received while the origin still holds this clone's own earlier record
/// of the same claim. Measured before the fix: the second `log` warned "taken
/// over while logging" and fetched the origin's record over the renewal, and
/// `done` exited 4 "moved while it was being completed".
#[test]
fn a_second_renewal_and_done_refused_before_the_update_lease_on_the_callers_own_earlier_record() {
    let c = Corpus::claimed("push-refused-second-write");
    let pushed = c.remote_claim().unwrap();
    c.refuse_pushes();

    for n in 1..=2 {
        cross_a_second();
        let before = c.local_claim().unwrap();
        let out = c.ank(&["log", &format!("renewal {n} the push url refuses")]);
        let said = format!("{}{}", text(&out.stdout), text(&out.stderr));
        assert_eq!(
            out.status.code(),
            Some(0),
            "log {n} wrote its entry:\n{said}"
        );
        no_race_reported(&format!("log {n}"), &said);
        c.diagnosis_reproduces(&said);
        let local = c.local_claim().unwrap();
        assert_ne!(
            local, pushed,
            "log {n}: the origin's record was fetched over the renewal"
        );
        assert_ne!(local, before, "log {n}: the claim was not renewed");
        assert_eq!(
            c.remote_claim(),
            Some(pushed.clone()),
            "the origin was never written"
        );
    }

    let head = git(&c.repo, &["rev-parse", "HEAD"]);
    let out = c.ank(&["done", "--proof", &format!("commit:{head}")]);
    let said = format!("{}{}", text(&out.stdout), text(&out.stderr));
    assert_eq!(
        out.status.code(),
        Some(0),
        "done completes locally after two local renewals:\n{said}"
    );
    no_race_reported("done", &said);
    c.diagnosis_reproduces(&said);
    assert_eq!(c.status_field(), "done");
    let local = c.local_claim().unwrap();
    let record = git(&c.repo, &["cat-file", "-p", &local]);
    assert!(
        record.contains("state: completed"),
        "the origin's record was fetched over the completion record:\n{record}"
    );
    assert_eq!(
        c.remote_claim(),
        Some(pushed),
        "the origin was never written"
    );
}

/// The other half of the criterion: only the same claim instance is the
/// caller's own. A record on the origin differing in task, holder or claimed is
/// somebody else's write, and it stays a lost race, fetched where the caller
/// looks next.
#[test]
fn a_record_differing_in_task_holder_or_claimed_stays_a_lost_race() {
    for field in ["task", "holder", "claimed"] {
        let c = Corpus::claimed(&format!("push-refused-other-{field}"));
        let pushed = c.remote_claim().unwrap();
        c.refuse_pushes();
        cross_a_second();
        let out = c.ank(&["log", "the first renewal stays local"]);
        assert_eq!(out.status.code(), Some(0), "log: {}", text(&out.stderr));

        // Another write lands on the origin: the pushed record with one of the
        // three fields changed.
        let theirs = git(&c.origin, &["cat-file", "-p", &pushed]);
        let changed: String = theirs
            .lines()
            .map(|l| match l.split_once(": ") {
                Some((k, _)) if k == field => match field {
                    "task" => format!("{k}: TASK-000000000002"),
                    "holder" => format!("{k}: someone-else/1.0"),
                    _ => format!("{k}: 2026-07-28T00:00:00Z"),
                },
                _ => l.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        assert_ne!(
            changed.trim(),
            theirs.trim(),
            "the record has no {field} line:\n{theirs}"
        );
        let path = c.origin.with_extension(format!("{field}.record"));
        fs::write(&path, &changed).unwrap();
        let object = git(&c.origin, &["hash-object", "-w", path.to_str().unwrap()]);
        git(&c.origin, &["update-ref", &Corpus::claim_ref(), &object]);
        // In this clone's object store too, so the record is read and compared
        // field by field rather than dismissed as an object the clone lacks.
        assert_eq!(
            git(&c.repo, &["hash-object", "-w", path.to_str().unwrap()]),
            object
        );

        cross_a_second();
        let out = c.ank(&["log", "the second renewal meets another write"]);
        let said = format!("{}{}", text(&out.stdout), text(&out.stderr));
        assert!(
            said.contains("taken over"),
            "{field}: a record differing in {field} was read as the caller's own:\n{said}"
        );
        assert_eq!(
            c.local_claim(),
            Some(object.clone()),
            "{field}: the other write was not fetched where the caller looks next"
        );
        assert_eq!(
            c.remote_claim(),
            Some(object),
            "{field}: the origin was overwritten"
        );
    }
}
