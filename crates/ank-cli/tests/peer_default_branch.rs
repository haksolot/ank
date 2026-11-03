//! A peer is read from git objects at its default branch, never from its
//! working tree (ADR-09b985a5627e, TASK-a8362b510188).
//!
//! In the multi-agent cycle the peer's checkout is another agent's workspace:
//! the branch it is on, what it has not committed, are that agent's business
//! and change nothing this repository is served. Measured before the change at
//! e04c0dad: a claim passed with the peer on an unmerged branch, the same ADR
//! read active or proposed by the peer's checkout, and an uncommitted draft was
//! listed as proposed.
//!
//! **Through the binary, over a pair** `api` / `web`, each declaring the other.
//! Every verb run in `web` is bracketed by a byte-for-byte snapshot of the whole
//! of `api`, `.git` included: no verb writes anything under the peer.

use crate::scratch;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-default-branch";

/// An ADR of `api` binding `web:src/**`.
const RULE: &str = "ADR-a1a1a1a1a1a1";
/// An ADR of `api` never committed.
const DRAFT: &str = "ADR-d2d2d2d2d2d2";
/// A task of `api` that a task of `web` waits on.
const FIRST: &str = "TASK-f3f3f3f3f3f3";
/// A local ADR of `web`, the local answer that survives an unreadable peer.
const LOCAL: &str = "ADR-e4e4e4e4e4e4";

struct Pair {
    base: PathBuf,
    api: PathBuf,
    web: PathBuf,
}

struct Answer {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Pair {
    fn run(&self, dir: &Path, args: &[&str]) -> Answer {
        let root = scratch::root();
        let home = self.base.join("home");
        let out = Command::new(ANK)
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
            .expect("the binary runs");
        Answer {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        }
    }

    /// A verb run in `web`, asserting nothing under `api` moved.
    fn web(&self, args: &[&str]) -> Answer {
        let before = snapshot(&self.api);
        let answer = self.run(&self.web, args);
        let after = snapshot(&self.api);
        assert!(
            before == after,
            "ank {args:?} in web modified api: {:?}",
            changed(&before, &after)
        );
        answer
    }

    fn web_ok(&self, args: &[&str]) -> String {
        let a = self.web(args);
        assert_eq!(a.code, Some(0), "ank {args:?}: {}", a.stderr);
        a.stdout
    }

    fn api_git(&self, args: &[&str]) {
        git(&self.api, args);
    }
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.email=t@example.invalid", "-c", "user.name=t"])
        .args(["-c", "commit.gpgsign=false"])
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

fn commit(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-qm", message]);
}

fn adr(dir: &Path, id: &str, status: &str, glob: &str) {
    fs::write(
        dir.join(format!(".ank/entities/{id}.md")),
        format!(
            "---\nid: {id}\ntype: adr\nslug: s-{id}\ntitle: Scoped {glob}\n\
             created: 2026-08-01T00:00:00Z\nstatus: {status}\nscope:\n  - '{glob}'\n\
             constraint: |\n  A rule about {glob}.\nschema: 1\nversion: 1\n---\n\nWhy.\n"
        ),
    )
    .unwrap();
}

fn task(dir: &Path, id: &str, status: &str) {
    fs::write(
        dir.join(format!(".ank/entities/{id}.md")),
        format!(
            "---\nid: {id}\ntype: task\nslug: s\ntitle: Peer work that comes first\n\
             created: 2026-08-01T00:00:00Z\nstatus: {status}\nscope:\n  - src/**\n\
             blocked_by: []\ndone_criteria: |\n  Something holds.\nschema: 4\nversion: 1\n\
             ---\n"
        ),
    )
    .unwrap();
}

fn pair(name: &str) -> Pair {
    let base = scratch::dir(name);
    let p = Pair {
        api: base.join("api"),
        web: base.join("web"),
        base,
    };
    for (dir, peer, path) in [(&p.api, "web", "../web"), (&p.web, "api", "../api")] {
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/lib.rs"), "\n").unwrap();
        git(dir, &["init", "-q", "-b", "main"]);
        // A commit or a merge otherwise starts `git maintenance run --auto
        // --detach`, which takes `.git/objects/maintenance.lock` in the
        // background while the next verb runs, and the snapshot of `api` reads
        // git's own write as one of ank's.
        git(dir, &["config", "maintenance.auto", "false"]);
        git(dir, &["config", "gc.auto", "0"]);
        for args in [
            vec!["init"],
            vec!["config", "default_branch", "main"],
            vec!["config", &format!("peers.{peer}"), path],
        ] {
            let a = p.run(dir, &args);
            assert_eq!(a.code, Some(0), "ank {args:?}: {}", a.stderr);
        }
    }
    adr(&p.web, LOCAL, "accepted", "src/**");
    commit(&p.web, "init");
    commit(&p.api, "init");
    p
}

/// Every file under `dir`, `.git` included, with its bytes.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
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

fn changed(a: &BTreeMap<PathBuf, Vec<u8>>, b: &BTreeMap<PathBuf, Vec<u8>>) -> Vec<PathBuf> {
    a.keys()
        .chain(b.keys())
        .filter(|k| a.get(*k) != b.get(*k))
        .cloned()
        .collect()
}

/// The sections of `context`: what it lists as binding, and as proposed.
fn sections(context: &str) -> (String, String) {
    let at = context.find("PROPOSED").unwrap_or(context.len());
    (context[..at].to_string(), context[at..].to_string())
}

fn rule() -> String {
    format!("{RULE}@api")
}

#[test]
fn an_adr_accepted_only_on_the_peers_checked_out_branch_does_not_bind() {
    let p = pair("peer-default-branch-adr");
    adr(&p.api, RULE, "proposed", "web:src/**");
    commit(&p.api, "propose");

    p.api_git(&["checkout", "-q", "-b", "work"]);
    adr(&p.api, RULE, "accepted", "web:src/**");
    commit(&p.api, "accept on work");

    let context = p.web_ok(&["context", "src/"]);
    let (active, proposed) = sections(&context);
    assert!(
        !active.contains(&rule()),
        "accepted on work only, it binds nobody:\n{context}"
    );
    assert!(
        proposed.contains(&rule()),
        "main holds it proposed, and that is what is read:\n{context}"
    );

    // Merged into the default branch, it binds, with api still on work.
    p.api_git(&["checkout", "-q", "main"]);
    p.api_git(&["merge", "-q", "--ff-only", "work"]);
    p.api_git(&["checkout", "-q", "-b", "elsewhere", "HEAD~1"]);
    let context = p.web_ok(&["context", "src/"]);
    let (active, _) = sections(&context);
    assert!(
        active.contains(&rule()),
        "accepted on main, it binds whatever api has checked out:\n{context}"
    );

    let _ = fs::remove_dir_all(&p.base);
}

#[test]
fn an_adr_left_uncommitted_in_the_peer_is_listed_by_no_verb() {
    let p = pair("peer-default-branch-draft");
    adr(&p.api, DRAFT, "proposed", "web:src/**");
    let draft = format!("{DRAFT}@api");

    for args in [
        vec!["context", "src/"],
        vec!["context"],
        vec!["scope", "src/lib.rs"],
        vec!["find", "@api"],
        vec!["find", "--type", "adr", "@api"],
    ] {
        let a = p.web(&args);
        assert!(
            !a.stdout.contains(DRAFT) && !a.stderr.contains(DRAFT),
            "ank {args:?} lists the draft:\n{}\n{}",
            a.stdout,
            a.stderr
        );
    }
    let shown = p.web(&["show", &draft]);
    assert_ne!(shown.code, Some(0), "{}", shown.stdout);

    // Committed on the default branch, the same verbs reach it.
    commit(&p.api, "draft");
    assert!(p.web_ok(&["show", &draft]).contains(DRAFT));
    let (_, proposed) = sections(&p.web_ok(&["context", "src/"]));
    assert!(proposed.contains(&draft), "{proposed}");

    let _ = fs::remove_dir_all(&p.base);
}

#[test]
fn a_peer_task_done_on_an_unmerged_branch_still_blocks_the_claim() {
    let p = pair("peer-default-branch-claim");
    task(&p.api, FIRST, "open");
    commit(&p.api, "first");

    let blocker = format!("{FIRST}@api");
    let created = p.web_ok(&[
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
        &blocker,
        "--json",
    ]);
    let at = created.find("\"TASK-").expect("the new task is named") + 1;
    let id = created[at..at + 17].to_string();
    commit(&p.web, "waits");

    p.api_git(&["checkout", "-q", "-b", "fix"]);
    task(&p.api, FIRST, "done");
    commit(&p.api, "done on fix");

    let refused = p.web(&["claim", &id]);
    assert_eq!(
        refused.code,
        Some(7),
        "done on an unmerged branch is not done: {}{}",
        refused.stdout,
        refused.stderr
    );
    assert!(refused.stderr.contains(&blocker), "{}", refused.stderr);

    p.api_git(&["checkout", "-q", "main"]);
    p.api_git(&["merge", "-q", "--no-ff", "-m", "merge fix", "fix"]);
    p.api_git(&["checkout", "-q", "-b", "next", "HEAD~1"]);
    let taken = p.web(&["claim", &id]);
    assert_eq!(
        taken.code,
        Some(0),
        "merged, it holds no more: {}",
        taken.stderr
    );

    let _ = fs::remove_dir_all(&p.base);
}

#[test]
fn the_remote_tracking_branch_answers_when_the_local_one_is_absent_and_neither_warns() {
    let p = pair("peer-default-branch-origin");
    adr(&p.api, RULE, "proposed", "web:src/**");
    commit(&p.api, "propose");
    p.api_git(&["update-ref", "refs/remotes/origin/main", "refs/heads/main"]);
    p.api_git(&["checkout", "-q", "-b", "side"]);
    adr(&p.api, RULE, "accepted", "web:src/**");
    commit(&p.api, "accept on side");
    p.api_git(&["branch", "-q", "-D", "main"]);

    let context = p.web_ok(&["context", "src/"]);
    let (active, proposed) = sections(&context);
    assert!(
        proposed.contains(&rule()) && !active.contains(&rule()),
        "origin/main holds it proposed:\n{context}"
    );

    p.api_git(&["update-ref", "-d", "refs/remotes/origin/main"]);
    let a = p.web(&["context", "src/"]);
    assert_eq!(a.code, Some(0), "{}", a.stderr);
    let both = format!("{}{}", a.stderr, a.stdout);
    let warnings: Vec<&str> = both
        .lines()
        .filter(|l| l.starts_with("warning:") && l.contains("peer 'api'"))
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "one warning names the peer: {}\n{}",
        a.stderr,
        a.stdout
    );
    assert!(
        !a.stdout.contains(RULE),
        "nothing of api is served:\n{}",
        a.stdout
    );
    assert!(
        a.stdout.contains(&LOCAL[..8]),
        "the local answer:\n{}",
        a.stdout
    );

    let _ = fs::remove_dir_all(&p.base);
}

/// A task of `api` that exists only on its unmerged branch `work`.
const PLANNED: &str = "TASK-a5a5a5a5a5a5";
/// An id `api` holds on no branch at all.
const NOWHERE: &str = "TASK-b6b6b6b6b6b6";

/// The lines of `check` in `web` that name `blocker`, with its exit code and
/// the whole of what it printed.
fn edge_findings(p: &Pair, blocker: &str) -> (Option<i32>, Vec<String>, String) {
    let a = p.web(&["check"]);
    let all = format!("{}{}", a.stdout, a.stderr);
    let found = all
        .lines()
        .filter(|l| l.contains(blocker))
        .map(str::to_string)
        .collect();
    (a.code, found, all)
}

#[test]
fn a_peer_task_only_on_an_unmerged_branch_is_accepted_held_and_a_signal() {
    let p = pair("peer-default-branch-unmerged");
    // The pair's local ADR is written by hand in a form `check` faults as
    // non-canonical; this test reads the exit code of `check`, so the only
    // finding it can be about is the edge.
    fs::remove_file(p.web.join(format!(".ank/entities/{LOCAL}.md"))).unwrap();
    commit(&p.web, "no local rule");
    p.api_git(&["checkout", "-q", "-b", "work"]);
    task(&p.api, PLANNED, "open");
    commit(&p.api, "planned on work");

    // new: accepted, with a warning naming the branch.
    let blocker = format!("{PLANNED}@api");
    let created = p.web(&[
        "new",
        "task",
        "--title",
        "Local work planned before the peer lands",
        "--scope",
        "src/**",
        "--criteria",
        "It waits.",
        "--no-verify",
        "--blocked-by",
        &blocker,
        "--json",
    ]);
    assert_eq!(
        created.code,
        Some(0),
        "a task on a branch of the peer is accepted: {}{}",
        created.stdout,
        created.stderr
    );
    let warned: Vec<&str> = created
        .stderr
        .lines()
        .filter(|l| l.starts_with("warning:") && l.contains(&blocker))
        .collect();
    assert!(
        warned.len() == 1 && warned[0].contains("'work'"),
        "one warning names the edge and the branch: {}",
        created.stderr
    );
    let at = created
        .stdout
        .find("\"TASK-")
        .expect("the new task is named")
        + 1;
    let id = created.stdout[at..at + 17].to_string();
    commit(&p.web, "waits");

    // new: a task the peer holds on no branch keeps its exit 2.
    let nowhere = format!("{NOWHERE}@api");
    let refused = p.web(&[
        "new",
        "task",
        "--title",
        "Mistyped",
        "--scope",
        "src/**",
        "--criteria",
        "Never.",
        "--no-verify",
        "--blocked-by",
        &nowhere,
    ]);
    assert_eq!(
        refused.code,
        Some(2),
        "no branch holds it: {}{}",
        refused.stdout,
        refused.stderr
    );

    // claim: the edge holds, naming the peer task.
    let held = p.web(&["claim", &id]);
    assert_eq!(held.code, Some(7), "{}{}", held.stdout, held.stderr);
    assert!(held.stderr.contains(&blocker), "{}", held.stderr);

    // check: a signal, exit 0, whichever branch api has checked out.
    for checkout in ["work", "main"] {
        p.api_git(&["checkout", "-q", checkout]);
        let (code, found, all) = edge_findings(&p, &blocker);
        assert_eq!(code, Some(0), "api on {checkout}:\n{all}");
        assert!(
            found.len() == 1 && found[0].starts_with("signal:") && found[0].contains("'work'"),
            "api on {checkout}, one signal naming the branch: {found:?}"
        );
    }

    // Merged into main and done there: nothing about the edge, and claim
    // succeeds.
    p.api_git(&["checkout", "-q", "work"]);
    task(&p.api, PLANNED, "done");
    commit(&p.api, "done on work");
    p.api_git(&["checkout", "-q", "main"]);
    p.api_git(&["merge", "-q", "--no-ff", "-m", "merge work", "work"]);
    let (code, found, all) = edge_findings(&p, &blocker);
    assert_eq!(code, Some(0), "{all}");
    assert!(found.is_empty(), "merged and done: {found:?}");
    let taken = p.web(&["claim", &id]);
    assert_eq!(taken.code, Some(0), "{}", taken.stderr);

    let _ = fs::remove_dir_all(&p.base);
}
