//! A ref record carrying a field this binary does not know is read without it
//! (TASK-04a688353ae4).
//!
//! `refs/ank/*` travels through the remote to every version of ank a team
//! runs, so a field a later binary adds to a record lands in front of every
//! older reader. Each case below writes a record through the binary, rewrites
//! it on the ref the way a later binary would have written it -- the same
//! record plus one top-level field -- and asks the binary to read it again.
//! The damage that was corrupt before stays corrupt: no state, an unknown
//! state, a required field missing.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const HOLDER: &str = "claude-code@holder";
const OTHER: &str = "codex@other";
/// The field no binary declares, as a later one would add it.
const LATER: &str = "added_by_a_later_ank: some value\n";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("ref-record-compat-gitconfig");
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
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A repository with an origin, one task committed and pushed on `main`.
struct Fixture {
    repo: PathBuf,
    task: String,
}

impl Fixture {
    fn new(what: &str) -> Fixture {
        let base = scratch::dir(what);
        let origin = base.join("origin.git");
        let repo = base.join("repo");
        fs::create_dir_all(repo.join("src")).unwrap();
        git(&base, &["init", "-q", "--bare", origin.to_str().unwrap()]);
        git(&repo, &["init", "-q", "-b", "main"]);
        git(
            &repo,
            &["remote", "add", "origin", origin.to_str().unwrap()],
        );
        let mut f = Fixture {
            repo,
            task: String::new(),
        };
        f.ok(HOLDER, &["init", "."]);
        f.ok(HOLDER, &["config", "default_branch", "main"]);
        fs::write(f.repo.join("src/main.rs"), "fn main() {}\n").unwrap();
        let said = f.ok(
            HOLDER,
            &[
                "new",
                "task",
                "--title",
                "A task whose records are read by an older binary",
                "--scope",
                "src/**",
                "--criteria",
                "The criterion.",
                "--no-verify",
            ],
        );
        let at = said.find("TASK-").expect("new names the task it wrote");
        f.task = said[at..at + 17].to_string();
        git(&f.repo, &["add", "-A"]);
        git(&f.repo, &["commit", "-qm", "seed"]);
        git(&f.repo, &["push", "-q", "origin", "main"]);
        f
    }

    fn ank(&self, agent: &str, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .env("ANK_AGENT", agent)
            .current_dir(&self.repo)
            .output()
            .unwrap()
    }

    fn ok(&self, agent: &str, args: &[&str]) -> String {
        let out = self.ank(agent, args);
        assert!(out.status.success(), "ank {args:?}: {}", text(&out));
        text(&out)
    }

    fn check(&self) -> (Option<i32>, String) {
        let out = self.ank(OTHER, &["check"]);
        (out.status.code(), text(&out))
    }

    fn claim_ref(&self) -> String {
        format!("refs/ank/claims/{}", self.task)
    }

    fn proof_ref(&self) -> String {
        format!("refs/ank/proof/{}", self.task)
    }

    fn record(&self, name: &str) -> String {
        git(&self.repo, &["cat-file", "-p", name])
    }

    /// Puts `record` on `name`, here and on origin, as another binary sharing
    /// the remote would have left it.
    fn put(&self, name: &str, record: &str) {
        let path = self.repo.with_extension("record");
        fs::write(&path, record).unwrap();
        let object = git(&self.repo, &["hash-object", "-w", path.to_str().unwrap()]);
        git(&self.repo, &["update-ref", name, object.trim()]);
        git(
            &self.repo,
            &["push", "-q", "--force", "origin", &format!("{name}:{name}")],
        );
    }

    /// The record on `name` rewritten with one field this binary does not
    /// declare, and the record as it was.
    fn add_unknown_field(&self, name: &str) -> String {
        let record = self.record(name);
        assert!(!record.contains("added_by_a_later_ank"), "{record}");
        self.put(name, &format!("{record}{LATER}"));
        record
    }

    fn head(&self) -> String {
        git(&self.repo, &["rev-parse", "HEAD"]).trim().to_string()
    }
}

/// `check` answers about a record carrying an unknown field exactly as it
/// answered about the record without it: the same exit, the same words.
fn check_is_unmoved(f: &Fixture, before: &(Option<i32>, String), what: &str) {
    let after = f.check();
    assert!(
        !after.1.contains("unreadable record"),
        "check reads the {what} record carrying an unknown field: {}",
        after.1
    );
    assert_eq!(
        &after, before,
        "check says of the {what} record with the unknown field what it said without it"
    );
}

#[test]
fn a_claim_record_with_an_unknown_field_is_the_same_claim() {
    let f = Fixture::new("ref-compat-claim");
    f.ok(HOLDER, &["claim", &f.task]);
    let before = f.check();
    f.add_unknown_field(&f.claim_ref());

    check_is_unmoved(&f, &before, "claim");

    let out = f.ank(OTHER, &["status"]);
    assert_eq!(out.status.code(), Some(0), "status: {}", text(&out));
    assert!(
        text(&out).contains(HOLDER),
        "status names the holder: {}",
        text(&out)
    );
    let out = f.ank(HOLDER, &["status"]);
    assert_eq!(out.status.code(), Some(0), "status: {}", text(&out));

    let out = f.ank(OTHER, &["claim", &f.task]);
    assert_eq!(
        out.status.code(),
        Some(4),
        "another identity is refused the claimed task: {}",
        text(&out)
    );
    assert!(
        text(&out).contains(HOLDER),
        "the refusal names the holder: {}",
        text(&out)
    );

    // `status` and the refused claim may have renewed or rewritten nothing;
    // the holder closes on the record as the later binary left it.
    let head = f.head();
    f.ok(HOLDER, &["done", "--proof", &format!("commit:{head}")]);
    let shown = f.ok(OTHER, &["show", &f.task]);
    assert!(shown.contains("status: done"), "the task closed: {shown}");
}

#[test]
fn a_completion_and_a_proof_record_with_an_unknown_field_are_the_same_records() {
    let f = Fixture::new("ref-compat-done");
    f.ok(HOLDER, &["claim", &f.task]);
    let head = f.head();
    f.ok(HOLDER, &["done", "--proof", &format!("commit:{head}")]);
    f.ok(
        HOLDER,
        &["attest", &f.task, "--proof", "test:ci-1", "--detached"],
    );
    assert!(
        f.record(&f.claim_ref()).starts_with("state: completed"),
        "done left a completion record"
    );
    assert!(
        f.record(&f.proof_ref()).starts_with("state: proof"),
        "attest left a proof record"
    );

    let before = f.check();
    f.add_unknown_field(&f.claim_ref());
    check_is_unmoved(&f, &before, "completion");
    let proof = f.add_unknown_field(&f.proof_ref());
    check_is_unmoved(&f, &before, "proof");

    // A field a later binary adds inside an attestation entry, not at the top.
    let entry = proof
        .lines()
        .find(|l| l.starts_with("- identity:"))
        .unwrap_or_else(|| panic!("an attestation entry opens on its identity: {proof}"));
    let nested = proof.replacen(
        &format!("{entry}\n"),
        &format!("{entry}\n  added_by_a_later_ank: nested\n"),
        1,
    );
    assert_ne!(nested, proof, "the field went inside the entry");
    f.put(&f.proof_ref(), &nested);
    check_is_unmoved(&f, &before, "proof with a nested field");
}

#[test]
fn a_record_with_no_state_an_unknown_state_or_a_missing_field_stays_corrupt() {
    let f = Fixture::new("ref-compat-corrupt");
    f.ok(HOLDER, &["claim", &f.task]);
    let name = f.claim_ref();
    let record = f.record(&name);
    assert!(record.starts_with("state: claim\n"), "{record}");

    let no_state = record.replacen("state: claim\n", "", 1);
    let unknown_state = record.replacen("state: claim\n", "state: reserved\n", 1);
    let no_holder: String = record
        .lines()
        .filter(|l| !l.starts_with("holder:"))
        .map(|l| format!("{l}\n"))
        .collect();
    assert_ne!(no_holder, record, "the record carried a holder to drop");

    for (variant, detail) in [
        (no_state, "no state field"),
        (unknown_state, "unknown state 'reserved'"),
        (no_holder, "missing field `holder`"),
    ] {
        f.put(&name, &variant);
        let (code, said) = f.check();
        assert_eq!(code, Some(8), "a damaged record is a fault: {said}");
        assert!(
            said.contains(&format!("unreadable record on {name}")) && said.contains(detail),
            "check names the damage, {detail}: {said}"
        );
    }
}
