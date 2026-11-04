//! A dead commit proof owes nothing once a later proof on the task names a
//! reachable commit (TASK-c2ed4811f8c7).
//!
//! A proof list is append-only, so the sha a rebase replaced stays in it for
//! good. `check` used to signal every such sha, which made the remedy it
//! printed, `ank attest <id> --proof commit:<sha>`, a command that cleared
//! nothing: the attestation appended a live proof and the dead one went on
//! being reported beside it. Five tasks of this corpus carried the signal with
//! a live proof after the dead one (TASK-a646a88c1c67).
//!
//! Every case through the binary, on one scratch repository:
//!
//! - a dead proof followed by a live one draws no signal;
//! - a task whose only commit proofs are dead still draws it, with the remedy;
//! - and that remedy, run, clears it.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "human:marie";

/// A dead proof, and a live one after it.
const SUPERSEDED: &str = "TASK-00000000c2e1";
/// Only dead proofs.
const DEAD_ONLY: &str = "TASK-00000000c2e2";

const KIND: &str = "proof-commit-unreachable";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("proof-superseded-gitconfig");
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

fn said(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

struct Repo(PathBuf);

impl Repo {
    fn new() -> Repo {
        let dir = scratch::dir("proof-superseded");
        fs::create_dir_all(dir.join(".ank/entities")).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        let r = Repo(dir);
        r.git(&["init", "-q", "-b", "main"]);
        fs::write(
            r.0.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(r.0.join(".gitignore"), ".ank/index.db*\n").unwrap();
        r
    }

    fn git(&self, args: &[&str]) -> String {
        let out = spawn("git")
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", said(&out));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn commit(&self, message: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", message, "--allow-empty"]);
        self.git(&["rev-parse", "HEAD"])
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

    /// A commit no ref reaches: made on a branch that is then deleted, which is
    /// where a rebase leaves the sha `done` recorded.
    fn dead_commit(&self, branch: &str) -> String {
        self.git(&["checkout", "-q", "-b", branch]);
        fs::write(self.0.join(format!("src/{branch}.rs")), "// gone\n").unwrap();
        let sha = self.commit(branch);
        self.git(&["checkout", "-q", "main"]);
        self.git(&["branch", "-q", "-D", branch]);
        sha
    }

    fn done_task(&self, id: &str, commits: &[&str]) {
        let proofs: String = commits
            .iter()
            .map(|c| format!("  - type: commit\n    ref: {c}\n"))
            .collect();
        fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\nslug: example\ntitle: Example\n\
                 created: 2026-07-01T00:00:00Z\nauthor: human:marie\nstatus: done\n\
                 scope:\n  - src/**\nblocked_by: []\n\
                 done_criteria: |\n  A verifiable criterion.\ncriteria_by: creator\n\
                 proof:\n{proofs}schema: 4\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
    }
}

fn unreachable_lines<'a>(said: &'a str, id: &str) -> Vec<&'a str> {
    said.lines()
        .filter(|l| l.contains(id) && l.contains("names no commit reachable"))
        .collect()
}

#[test]
fn a_dead_commit_proof_followed_by_a_live_one_owes_nothing_and_attest_clears_one_that_has_none() {
    let r = Repo::new();
    let live = r.commit("seed");
    let dead_a = r.dead_commit("rebased-a");
    let dead_b = r.dead_commit("rebased-b");
    // The fixture is asserted and not assumed: a dead sha some ref still
    // reached would leave every assertion below passing for the wrong reason.
    let reachable = r.git(&["rev-list", "--all"]);
    assert!(reachable.contains(&live), "the live commit is reachable");
    assert!(
        !reachable.contains(&dead_a) && !reachable.contains(&dead_b),
        "the dead commits are reached by no ref:\n{reachable}"
    );

    r.done_task(SUPERSEDED, &[&dead_a, &live]);
    r.done_task(DEAD_ONLY, &[&dead_b]);
    r.commit("corpus");

    // --all, so a signal that owes nothing is printed too and cannot hide in
    // the count line.
    let out = r.ank(&["check", "--all"]);
    let text = said(&out);
    assert!(
        unreachable_lines(&text, SUPERSEDED).is_empty(),
        "a dead proof with a live one after it owes nothing:\n{text}"
    );
    let dead = unreachable_lines(&text, DEAD_ONLY);
    assert_eq!(
        dead.len(),
        1,
        "the task with only dead proofs is named:\n{text}"
    );
    assert!(dead[0].contains(&dead_b), "it names the reference:\n{text}");
    assert!(
        dead[0].contains(&format!("ank attest {DEAD_ONLY} --proof commit:<sha>")),
        "with the remedy it prints today:\n{text}"
    );

    // And the --kind view agrees: the superseded task is absent from it.
    let out = r.ank(&["check", "--kind", KIND]);
    let text = said(&out);
    assert!(
        !text.contains(SUPERSEDED),
        "no {KIND} on {SUPERSEDED}:\n{text}"
    );
    assert!(text.contains(DEAD_ONLY), "{KIND} on {DEAD_ONLY}:\n{text}");

    // The remedy, run: it clears the signal it is printed beside.
    let out = r.ank(&["attest", DEAD_ONLY, "--proof", &format!("commit:{live}")]);
    assert_eq!(out.status.code(), Some(0), "attest: {}", said(&out));
    // The dead entry stays: a proof list is append-only.
    let file = fs::read_to_string(r.0.join(format!(".ank/entities/{DEAD_ONLY}.md"))).unwrap();
    assert!(file.contains(&dead_b), "the dead entry is never removed");

    let out = r.ank(&["check", "--all"]);
    let text = said(&out);
    assert!(
        unreachable_lines(&text, DEAD_ONLY).is_empty(),
        "attest cleared the signal it names:\n{text}"
    );
    let out = r.ank(&["check", "--kind", KIND]);
    let text = said(&out);
    assert!(
        !text.contains(DEAD_ONLY),
        "nothing of {KIND} about {DEAD_ONLY} after attest:\n{text}"
    );
}
