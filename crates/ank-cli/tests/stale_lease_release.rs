//! A release after a renewal that never reached the remote (TASK-34fe87319194).
//!
//! A renewal that does not travel keeps its local write (ADR-af533e7a3e03,
//! ADR-aaee5307b34c), so the clone's claim ref and the remote's diverge: the
//! remote still holds the caller's own earlier record of the same claim. The
//! deletion `release` then pushes leases on the local object, git refuses it as
//! `rejected (stale info)`, and `ls-remote` answers with that older record. Read
//! as a lost race, it made `release` print `released` and nothing else while
//! origin kept the claim until its TTL.
//!
//! Through the binary, over a bare origin reached by `file://`, because the
//! defect lives in the push and a fixture without a remote never pushes.

use crate::scratch;

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "tool/1.0";
const OTHER: &str = "other/2.0";
const T: &str = "TASK-000000000001";
const STILL_HELD: &str = "another clone still reads the task as held";

/// A `pre-receive` that refuses every `refs/ank/*` update.
const REFUSING_HOOK: &str = "#!/bin/sh\n\
while read old new ref; do\n\
  case \"$ref\" in refs/ank/*) exit 1;; esac\n\
done\n\
exit 0\n";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("stale-lease-release-gitconfig");
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

/// Returns once the wall clock is in a later second than it was on entry, so
/// the renewal stamps an expiry the pushed record does not already carry.
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

/// Installs `script` as the bare remote's `pre-receive` hook, and runs it once
/// before git does: a script written moments before it is executed can meet
/// `ETXTBSY` on Linux while another test thread's fork still holds the write
/// descriptor, and git would then report a hook it could not start as a hook
/// that declined.
fn install_pre_receive(origin: &Path, script: &str) -> PathBuf {
    let hook = origin.join("hooks").join("pre-receive");
    fs::create_dir_all(hook.parent().unwrap()).unwrap();
    fs::write(&hook, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
        for attempt in 0.. {
            match spawn(&hook)
                .current_dir(origin)
                .stdin(Stdio::null())
                .output()
            {
                Ok(_) => break,
                Err(e) if e.raw_os_error() == Some(26) && attempt < 50 => {
                    std::thread::sleep(Duration::from_millis(20))
                }
                Err(e) => panic!("the hook could not be started: {e}"),
            }
        }
    }
    hook
}

/// A corpus of one task with a bare origin, claimed through the binary so the
/// claim is on the origin before anything else happens.
struct Corpus {
    repo: PathBuf,
    origin: PathBuf,
}

impl Corpus {
    fn claimed(what: &str) -> Corpus {
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

        // `file:///C:/...` on Windows, `file:///tmp/...` elsewhere.
        let path = origin.to_str().unwrap().replace('\\', "/");
        let url = if path.starts_with('/') {
            format!("file://{path}")
        } else {
            format!("file:///{path}")
        };
        git(&repo, &["remote", "add", "origin", &url]);
        git(
            &repo,
            &[
                "config",
                "--add",
                "remote.origin.fetch",
                "+refs/ank/*:refs/ank/*",
            ],
        );
        let c = Corpus { repo, origin };
        c.ok(&["claim", T]);
        assert_eq!(
            c.remote_claim(),
            Some(c.local_claim().unwrap()),
            "the claim reached the origin"
        );
        c
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

    fn ok(&self, args: &[&str]) -> Output {
        let out = self.ank(args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?}: {}",
            text(&out.stderr)
        );
        out
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

    fn remote_claim(&self) -> Option<String> {
        let line = git(&self.repo, &["ls-remote", "origin", &Self::claim_ref()]);
        line.split_whitespace().next().map(str::to_string)
    }

    /// Renews the claim with `log` while the origin is out of reach, by moving
    /// its directory away and back: the renewal keeps its local write and the
    /// two refs diverge.
    fn renew_while_unreachable(&self) {
        let away = self.origin.with_extension("away");
        fs::rename(&self.origin, &away).unwrap();
        cross_a_second();
        let out = self.ok(&["log", "renewed while the origin was away"]);
        fs::rename(&away, &self.origin).unwrap();
        assert!(
            text(&out.stderr).contains("not pushed"),
            "the renewal was expected to stay local: {}",
            text(&out.stderr)
        );
    }

    /// Renews the claim with `log` while the origin refuses every ank ref, then
    /// lifts the refusal.
    fn renew_while_refused(&self) {
        let hook = install_pre_receive(&self.origin, REFUSING_HOOK);
        cross_a_second();
        let out = self.ok(&["log", "renewed while the origin refused"]);
        fs::remove_file(hook).unwrap();
        assert!(
            text(&out.stderr).contains("pre-receive hook declined"),
            "the renewal was expected to be refused: {}",
            text(&out.stderr)
        );
    }

    /// Writes a claim record held by `holder` straight into the origin, as
    /// another clone's claim would arrive there.
    fn plant_on_origin(&self, holder: &str) -> String {
        let record = format!(
            "state: claim\ntask: {T}\nholder: {holder}\nclaimed: 2026-07-28T00:00:00Z\n\
             expires: 2999-01-01T00:00:00Z\nttl: 7200\ncriteria: '0'\nconstraints: '0'\n"
        );
        let mut child = spawn("git")
            .args(["hash-object", "-w", "--stdin"])
            .current_dir(&self.origin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(record.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        let object = text(&out.stdout).trim().to_string();
        git(&self.origin, &["update-ref", &Self::claim_ref(), &object]);
        object
    }
}

/// The criterion's first clause, both ways a renewal fails to travel: the
/// release exits 0 and either the claim is gone from origin or stderr says
/// another clone still reads the task as held. Never silence over a kept claim.
#[test]
fn a_release_after_a_renewal_that_stayed_local_never_leaves_origin_holding_in_silence() {
    for (what, diverge) in [
        (
            "stale-lease-release-unreachable",
            Corpus::renew_while_unreachable as fn(&Corpus),
        ),
        ("stale-lease-release-refused", Corpus::renew_while_refused),
    ] {
        let c = Corpus::claimed(what);
        let pushed = c.remote_claim().unwrap();
        diverge(&c);
        assert_ne!(
            c.local_claim().unwrap(),
            pushed,
            "{what}: the renewal moved the local ref only"
        );
        assert_eq!(
            c.remote_claim(),
            Some(pushed),
            "{what}: origin kept the old one"
        );

        let out = c.ank(&["release", "--reason", "handing it back"]);
        let stderr = text(&out.stderr);
        assert_eq!(out.status.code(), Some(0), "{what}: release: {stderr}");
        let kept = c.remote_claim();
        assert!(
            kept.is_none() || stderr.contains(STILL_HELD),
            "{what}: release was silent while origin kept the claim {kept:?}: {stderr}"
        );
        assert_eq!(
            kept, None,
            "{what}: origin reachable and accepting, so the deletion should have been made"
        );
        assert_eq!(c.local_claim(), None, "{what}: the local claim is gone");
    }
}

/// The criterion's second clause: a remote holding another identity's claim is
/// a claim the deletion never touches, whatever this clone's lease said.
#[test]
fn a_release_never_deletes_another_identitys_claim_on_origin() {
    let c = Corpus::claimed("stale-lease-release-other-identity");
    c.renew_while_unreachable();
    let theirs = c.plant_on_origin(OTHER);

    let out = c.ank(&["release", "--reason", "handing it back"]);
    assert_eq!(out.status.code(), Some(0), "release: {}", text(&out.stderr));
    assert_eq!(
        c.remote_claim(),
        Some(theirs),
        "the release deleted a claim another identity holds on origin"
    );
}

/// The same holder name with another claim instance is not the caller's own
/// earlier record either: it is a claim taken again elsewhere, and it stays.
#[test]
fn a_release_never_deletes_a_later_claim_of_the_same_name_on_origin() {
    let c = Corpus::claimed("stale-lease-release-same-name");
    c.renew_while_unreachable();
    let theirs = c.plant_on_origin(AGENT);

    let out = c.ank(&["release", "--reason", "handing it back"]);
    assert_eq!(out.status.code(), Some(0), "release: {}", text(&out.stderr));
    assert_eq!(
        c.remote_claim(),
        Some(theirs),
        "the release deleted a claim it did not write"
    );
}

/// When the deletion cannot be made -- the origin still refusing at release --
/// the verb degrades and says so: exit 0, the remote's reason, and the warning
/// that another clone still reads the task as held.
#[test]
fn a_release_the_origin_still_refuses_warns_that_the_task_reads_as_held() {
    let c = Corpus::claimed("stale-lease-release-still-refused");
    let pushed = c.remote_claim().unwrap();
    let _hook = install_pre_receive(&c.origin, REFUSING_HOOK);
    cross_a_second();
    c.ok(&["log", "renewed while the origin refused"]);

    let out = c.ank(&["release", "--reason", "handing it back"]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "release: {stderr}");
    assert_eq!(
        c.remote_claim(),
        Some(pushed),
        "the hook refused the deletion"
    );
    assert!(
        stderr.contains(STILL_HELD) && stderr.contains("pre-receive hook declined"),
        "release was silent while origin kept the claim: {stderr}"
    );
    assert_eq!(c.local_claim(), None, "the local claim is gone");
}
