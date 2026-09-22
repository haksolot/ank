//! A remote that has no copy of a claim is not somebody else's write
//! (TASK-2d779142ca70).
//!
//! The claim is taken with no remote, then an origin is added that carries the
//! `+refs/ank/*:refs/ank/*` refspec `ank init` writes and no ank ref at all.
//! Every write after that leases the push on the object the local ref holds,
//! the origin has nothing there, and the refused lease used to be read as a
//! takeover: `log` warned that the claim was not renewed while renewing it, and
//! `done` exited 4 after writing the status, the proof and the completion ref.
//!
//! Through the binary, over a bare origin reached by `file://`, because the
//! defect lives in the push and a fixture without a remote never pushes.

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "tool/1.0";
const T: &str = "TASK-000000000001";
const TAKEOVER: &str = "was taken over while logging";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("remote-without-claim-gitconfig");
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

/// Returns once the wall clock is in a later second than it was on entry, so
/// the next renewal stamps an expiry the ref does not already carry.
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

/// A corpus of one task, claimed with no remote, then given an origin that
/// holds no ank ref.
struct Corpus {
    repo: PathBuf,
    origin: PathBuf,
}

impl Corpus {
    fn claimed_then_remote_added(what: &str) -> Corpus {
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
        let c = Corpus { repo, origin };
        c.ok(&["claim", T]);

        // `file:///C:/...` on Windows, `file:///tmp/...` elsewhere.
        let path = c.origin.to_str().unwrap().replace('\\', "/");
        let url = if path.starts_with('/') {
            format!("file://{path}")
        } else {
            format!("file:///{path}")
        };
        git(&c.repo, &["remote", "add", "origin", &url]);
        git(
            &c.repo,
            &[
                "config",
                "--add",
                "remote.origin.fetch",
                "+refs/ank/*:refs/ank/*",
            ],
        );
        assert_eq!(c.remote_ank_refs(), "", "the origin starts with no ank ref");
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

    fn field(&self, key: &str) -> String {
        let record = git(
            &self.repo,
            &["cat-file", "-p", &format!("refs/ank/claims/{T}")],
        );
        record
            .lines()
            .find_map(|l| l.strip_prefix(&format!("{key}: ")).map(str::to_string))
            .unwrap_or_else(|| panic!("the record carries no {key}: {record}"))
    }

    fn remote_ank_refs(&self) -> String {
        git(&self.repo, &["ls-remote", "origin", "refs/ank/*"])
    }
}

/// Each form gets a corpus of its own: the first renewal that succeeds puts the
/// ref on the origin, and a second call on the same corpus would no longer be
/// in the state the defect needs.
#[test]
fn log_renews_and_reports_no_takeover_when_the_remote_has_no_copy() {
    for (what, args) in [
        ("remote-without-claim-log", &["log", "hello"][..]),
        (
            "remote-without-claim-log-json",
            &["log", "--json", "hello"][..],
        ),
    ] {
        let c = Corpus::claimed_then_remote_added(what);
        let before = c.field("expires");
        cross_a_second();

        let out = c.ok(args);
        let (stdout, stderr) = (text(&out.stdout), text(&out.stderr));
        assert!(
            !stdout.contains(TAKEOVER) && !stderr.contains(TAKEOVER),
            "{args:?} reported a takeover: {stdout}{stderr}"
        );
        assert_eq!(c.field("holder"), AGENT, "{args:?}: the holder moved");
        assert_ne!(c.field("expires"), before, "{args:?} renews the lease");
    }
}

#[test]
fn done_exits_zero_when_the_remote_has_no_copy() {
    let c = Corpus::claimed_then_remote_added("remote-without-claim-done");
    cross_a_second();
    let head = git(&c.repo, &["rev-parse", "HEAD"]);

    let out = c.ank(&["done", "--proof", &format!("commit:{head}")]);
    let err = text(&out.stderr);
    assert!(
        !err.contains("moved while it was being completed"),
        "done reported a move that did not happen: {err}"
    );
    assert_eq!(out.status.code(), Some(0), "done: {err}");
}
