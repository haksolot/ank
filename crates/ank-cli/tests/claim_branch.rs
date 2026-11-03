//! A claim records the branch it was taken on, and `check` judges it from that
//! branch (ADR-5a3a3f1b89e1, TASK-346bdb3b4378).
//!
//! `refs/ank/*` is shared by every worktree of a repository while an entity
//! file lives per branch. Each case below is two linked worktrees, A and B, on
//! branches cut from the default one, an agent claiming in A and another
//! running `check` in B, all through the binary. The state asserted is the
//! repository's, read by git afterwards: whether the claim ref still exists,
//! and what the record on it says.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const IN_A: &str = "claude-code@worktree-a";
/// A second agent in A: one identity holds one live claim per corpus.
const ALSO_IN_A: &str = "claude-code@worktree-a-2";
const IN_B: &str = "codex@worktree-b";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("claim-branch-gitconfig");
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
    c.env_remove("GIT_TRACE")
        .env_remove("ANK_GUARD_EDIT")
        .env_remove("ANK_AGENT")
        .env("GIT_CONFIG_GLOBAL", config)
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

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A default branch carrying one task, and two worktrees cut from it.
struct Fixture {
    main: PathBuf,
    a: PathBuf,
    b: PathBuf,
    /// The task every branch carries, committed on the default branch.
    shared: String,
}

impl Fixture {
    fn new(what: &str) -> Fixture {
        let base = scratch::dir(what);
        let main = base.join("main");
        fs::create_dir_all(main.join("src")).unwrap();
        git(&main, &["init", "-q", "-b", "main"]);
        let out = spawn(ANK)
            .arg("init")
            .arg(&main)
            .current_dir(scratch::root())
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
        fs::write(main.join("src/main.rs"), "fn main() {}\n").unwrap();
        let out = spawn(ANK)
            .args(["config", "default_branch", "main", "--repo"])
            .arg(&main)
            .current_dir(scratch::root())
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
        let mut f = Fixture {
            a: base.join("a"),
            b: base.join("b"),
            main,
            shared: String::new(),
        };
        f.shared = f.new_task(&f.main.clone(), "The shared task", "The shared criterion.");
        git(&f.main, &["add", "-A"]);
        git(&f.main, &["commit", "-qm", "seed"]);
        git(
            &f.main,
            &["worktree", "add", "-q", "-b", "a", f.a.to_str().unwrap()],
        );
        git(
            &f.main,
            &["worktree", "add", "-q", "-b", "b", f.b.to_str().unwrap()],
        );
        f
    }

    fn ank(&self, agent: &str, at: &Path, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .arg("--repo")
            .arg(at)
            .env("ANK_AGENT", agent)
            .current_dir(at)
            .output()
            .unwrap()
    }

    fn ok(&self, agent: &str, at: &Path, args: &[&str]) -> String {
        let out = self.ank(agent, at, args);
        assert!(out.status.success(), "ank {args:?}: {}", text(&out));
        text(&out)
    }

    fn new_task(&self, at: &Path, title: &str, criteria: &str) -> String {
        let said = self.ok(
            IN_A,
            at,
            &[
                "new",
                "task",
                "--title",
                title,
                "--scope",
                "src/**",
                "--criteria",
                criteria,
                "--no-verify",
            ],
        );
        let at = said.find("TASK-").expect("new names the task it wrote");
        said[at..at + 17].to_string()
    }

    fn check(&self, at: &Path) -> (i32, String) {
        let out = self.ank(IN_B, at, &["check"]);
        (out.status.code().unwrap_or(-1), text(&out))
    }

    fn claim_ref(&self, id: &str) -> String {
        format!("refs/ank/claims/{id}")
    }

    /// The record on the claim ref, or `None` once it is gone.
    fn record(&self, id: &str) -> Option<String> {
        let out = spawn("git")
            .args(["cat-file", "-p", &self.claim_ref(id)])
            .current_dir(&self.main)
            .output()
            .unwrap();
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).to_string())
    }

    /// Rewrites the record on the claim ref as a binary predating the field
    /// would have written it: the same record, with no `branch:` line.
    fn strip_branch(&self, id: &str) {
        let record = self.record(id).expect("a claim to rewrite");
        let old: String = record
            .lines()
            .filter(|l| !l.starts_with("branch:"))
            .map(|l| format!("{l}\n"))
            .collect();
        assert_ne!(old, record, "the record carried a branch to strip");
        let path = self.main.with_extension("record");
        fs::write(&path, old).unwrap();
        let object = git(&self.main, &["hash-object", "-w", path.to_str().unwrap()]);
        git(&self.main, &["update-ref", &self.claim_ref(id), &object]);
    }

    fn entity(&self, at: &Path, id: &str) -> PathBuf {
        at.join(".ank/entities").join(format!("{id}.md"))
    }

    /// The criterion rewritten in the file and committed, past the CLI.
    fn edit_criterion_by_hand(&self, at: &Path, id: &str, from: &str, to: &str) {
        let path = self.entity(at, id);
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains(from), "{body}");
        fs::write(&path, body.replace(from, to)).unwrap();
        git(at, &["commit", "-qam", "edit the criterion by hand"]);
    }
}

fn orphaned(said: &str, id: &str) -> bool {
    said.contains(&format!("pruned refs/ank/claims/{id}"))
        || said.lines().any(|l| l.contains(id) && l.contains("orphan"))
}

fn diverged(said: &str, id: &str) -> bool {
    said.lines()
        .any(|l| l.contains(id) && l.contains("diverges from the claim"))
}

#[test]
fn the_claim_record_names_the_branch_it_was_taken_on() {
    let f = Fixture::new("claim-branch-record");
    f.ok(IN_A, &f.a, &["claim", &f.shared]);
    let record = f.record(&f.shared).unwrap();
    assert!(
        record.lines().any(|l| l == "branch: a"),
        "the record names A's branch: {record}"
    );
}

#[test]
fn a_task_only_on_the_claiming_branch_is_no_orphan_until_that_branch_is_gone() {
    let f = Fixture::new("claim-branch-orphan");
    let only_a = f.new_task(&f.a, "Born on A", "A criterion born on A.");
    git(&f.a, &["add", "-A"]);
    git(&f.a, &["commit", "-qm", "a task on A"]);
    f.ok(ALSO_IN_A, &f.a, &["claim", &only_a]);

    let (_, said) = f.check(&f.b);
    assert!(!orphaned(&said, &only_a), "B reports no orphan: {said}");
    assert!(f.record(&only_a).is_some(), "the claim survives check in B");

    // A's branch leaves, as it does once its work is abandoned.
    git(
        &f.main,
        &["worktree", "remove", "--force", f.a.to_str().unwrap()],
    );
    git(&f.main, &["branch", "-D", "a"]);
    let (_, said) = f.check(&f.b);
    assert!(
        f.record(&only_a).is_none(),
        "with A's branch gone, B prunes the ref: {said}"
    );
}

#[test]
fn a_criterion_amended_on_the_claiming_branch_draws_no_fault_elsewhere() {
    let f = Fixture::new("claim-branch-amended");
    f.ok(
        IN_A,
        &f.a,
        &["amend", &f.shared, "--criteria", "The amended criterion."],
    );
    git(&f.a, &["add", "-A"]);
    git(&f.a, &["commit", "-qm", "amend on A"]);
    f.ok(IN_A, &f.a, &["claim", &f.shared]);

    let (code, said) = f.check(&f.b);
    assert!(!diverged(&said, &f.shared), "B draws no fault: {said}");
    assert_ne!(code, 8, "{said}");
    let (code, said) = f.check(&f.a);
    assert!(!diverged(&said, &f.shared), "nor does A: {said}");
    assert_ne!(code, 8, "{said}");
}

#[test]
fn a_criterion_edited_by_hand_on_the_claiming_branch_is_a_fault_from_both() {
    let f = Fixture::new("claim-branch-hand-edit");
    // The task every checkout carries, amended on A first so that B's copy
    // differs from the claim before the hand edit makes A's differ too.
    f.ok(
        IN_A,
        &f.a,
        &["amend", &f.shared, "--criteria", "The amended criterion."],
    );
    let only_a = f.new_task(&f.a, "Born on A", "A criterion born on A.");
    git(&f.a, &["add", "-A"]);
    git(&f.a, &["commit", "-qm", "amend and add on A"]);
    f.ok(IN_A, &f.a, &["claim", &f.shared]);
    f.ok(ALSO_IN_A, &f.a, &["claim", &only_a]);

    f.edit_criterion_by_hand(&f.a, &f.shared, "The amended criterion.", "Softened.");
    f.edit_criterion_by_hand(&f.a, &only_a, "A criterion born on A.", "Softened too.");

    for at in [&f.a, &f.b] {
        let (code, said) = f.check(at);
        assert_eq!(code, 8, "{said}");
        assert!(diverged(&said, &f.shared), "{at:?}: {said}");
        assert!(diverged(&said, &only_a), "{at:?}: {said}");
    }
}

#[test]
fn a_criterion_edited_by_hand_and_left_uncommitted_is_a_fault_in_its_checkout() {
    let f = Fixture::new("claim-branch-uncommitted");
    f.ok(IN_A, &f.a, &["claim", &f.shared]);
    let path = f.entity(&f.a, &f.shared);
    let body = fs::read_to_string(&path).unwrap();
    fs::write(&path, body.replace("The shared criterion.", "Softened.")).unwrap();

    let (code, said) = f.check(&f.a);
    assert_eq!(code, 8, "{said}");
    assert!(diverged(&said, &f.shared), "{said}");
}

#[test]
fn a_record_without_a_branch_is_judged_as_before() {
    let f = Fixture::new("claim-branch-legacy");
    let only_a = f.new_task(&f.a, "Born on A", "A criterion born on A.");
    f.ok(
        IN_A,
        &f.a,
        &["amend", &f.shared, "--criteria", "The amended criterion."],
    );
    git(&f.a, &["add", "-A"]);
    git(&f.a, &["commit", "-qm", "amend and add on A"]);
    f.ok(IN_A, &f.a, &["claim", &f.shared]);
    f.ok(ALSO_IN_A, &f.a, &["claim", &only_a]);
    f.strip_branch(&f.shared);
    f.strip_branch(&only_a);

    // One run: the divergence a checkout lacking the newer file used to
    // fault on, and the orphan it used to collect.
    let (code, said) = f.check(&f.b);
    assert_eq!(code, 8, "{said}");
    assert!(diverged(&said, &f.shared), "the old fault stands: {said}");
    assert!(
        f.record(&only_a).is_none(),
        "and the old orphan is pruned: {said}"
    );
}
