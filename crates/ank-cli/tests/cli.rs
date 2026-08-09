//! The binary, run as a process (§4).
//!
//! Everything else in this crate is tested inside the modules, where it is
//! cheaper and sharper. What cannot be tested there is what only exists once
//! the process exists: dispatch actually reaching a verb, and the exit code
//! carrying the semantics of §4 all the way out to the shell.
//!
//! The distinction is not academic here. Two real defects had already slipped
//! through green unit tests — a lock whose release failed under concurrency,
//! and a `--repo` resolution that dispatch never reached because it rejected
//! the verb first. `claim` is the third of the family: the module was complete
//! and tested, and no invocation of the binary could reach a line of it,
//! because the arm did not exist while six module headers said it did.
//!
//! An integration test is also the only place `CARGO_BIN_EXE_ank` is defined,
//! and `ank-cli` has no library target — so there is no unit test that could
//! spawn the binary even if we wanted one.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// A real repository with a real `.ank/`: `startup` resolves the repository,
/// checks git and loads the config before any verb runs, so a fixture missing
/// any of the three would test the failure path instead.
struct Repo(PathBuf);

impl Repo {
    fn new() -> Repo {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "ank-cli-it-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(p.join(".ank/tasks")).unwrap();
        std::fs::create_dir_all(p.join(".ank/adr")).unwrap();
        let r = Repo(p);
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "test@ank.local"]);
        r.git(&["config", "user.name", "Test"]);
        r.git(&["config", "core.autocrlf", "false"]);
        std::fs::write(
            r.0.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        r
    }

    fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("git must be installed: it is a hard dependency");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Declares verifiers and makes a commit, which `done` needs on both
    /// counts: something to run, and a HEAD for the completion record to name.
    fn with_verifiers(self, verifiers: &str) -> Repo {
        std::fs::write(
            self.0.join(".ank/config.yml"),
            format!("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n{verifiers}"),
        )
        .unwrap();
        std::fs::write(self.0.join("seed.txt"), "x").unwrap();
        self.git(&["add", "-A"]);
        self.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);
        self
    }

    fn head(&self) -> String {
        self.git(&["rev-parse", "HEAD"])
    }

    /// Reads the ref with git and not through `claim::read`: what has to be
    /// true is the state of the repository, not the module's agreement with
    /// itself.
    fn claim_ref(&self, id: &str) -> Option<String> {
        let out = Command::new("git")
            .current_dir(&self.0)
            .args(["cat-file", "-p", &format!("refs/ank/claims/{id}")])
            .output()
            .unwrap();
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn seed_task(&self, id: &str, criteria: Option<&str>) {
        self.seed_task_with(id, criteria, &[]);
    }

    fn seed_task_with(&self, id: &str, criteria: Option<&str>, verify: &[&str]) {
        let criteria = match criteria {
            Some(c) => format!("done_criteria: |\n  {c}\ncriteria_by: creator\n"),
            None => String::new(),
        };
        let verify = if verify.is_empty() {
            String::new()
        } else {
            format!("verify: [{}]\n", verify.join(", "))
        };
        std::fs::write(
            self.0.join(".ank/tasks").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\nslug: example\ntitle: Example task\n\
                 created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                 blocked_by: []\n{criteria}{verify}schema: 1\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
    }

    /// The same seed with a scope of its own, for a fixture that needs an
    /// entity outside the perimeter under test.
    fn seed_task_scoped(&self, id: &str, scope: &str) {
        self.seed_task(id, Some("A verifiable criterion."));
        let text = self
            .task_text(id)
            .replace("  - src/**", &format!("  - {scope}"));
        std::fs::write(self.0.join(".ank/tasks").join(format!("{id}.md")), text).unwrap();
    }

    /// Adds blockers to a seeded task. Written into the file rather than through
    /// `amend`, so that a graph fixture does not depend on the verb that edits
    /// the field it is drawing.
    fn blocked(&self, id: &str, blockers: &[&str]) {
        let list = blockers.join(", ");
        let text = self
            .task_text(id)
            .replace("blocked_by: []", &format!("blocked_by: [{list}]"));
        std::fs::write(self.0.join(".ank/tasks").join(format!("{id}.md")), text).unwrap();
    }

    fn task_text(&self, id: &str) -> String {
        std::fs::read_to_string(self.0.join(".ank/tasks").join(format!("{id}.md"))).unwrap()
    }

    fn adr_text(&self, id: &str) -> String {
        std::fs::read_to_string(self.0.join(".ank/adr").join(format!("{id}.md"))).unwrap()
    }

    fn seed_adr(&self, id: &str, constraint: &str, scope: &str) {
        std::fs::write(
            self.0.join(".ank/adr").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: adr\nslug: example\ntitle: A decision\n\
                 created: 2026-07-20T00:00:00Z\nstatus: proposed\nscope:\n  - {scope}\n\
                 constraint: |\n  {constraint}\nschema: 1\nversion: 1\n---\n\nWhy.\n"
            ),
        )
        .unwrap();
    }

    /// `accept` signs for real, and a signature configured from the developer's
    /// own global git config would make this test pass here and nowhere else.
    /// SSH because `ssh-keygen` ships beside git on all three platforms and
    /// needs no agent, no keyring and no passphrase prompt.
    fn enable_signing(&self) {
        let key = self.0.join("signing-key");
        let out = Command::new("ssh-keygen")
            .args(["-t", "ed25519", "-N", "", "-C", "ank test", "-q", "-f"])
            .arg(&key)
            .output()
            .expect("ssh-keygen ships with git on all three platforms");
        assert!(
            out.status.success(),
            "ssh-keygen: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        self.git(&["config", "gpg.format", "ssh"]);
        self.git(&[
            "config",
            "user.signingkey",
            key.with_extension("pub").to_str().unwrap(),
        ]);
    }

    /// `--repo` rather than a working directory, so that the flag stays on a
    /// path a real invocation takes.
    fn ank(&self, agent: &str, args: &[&str]) -> Output {
        self.ank_at(agent, args, &self.0)
    }

    /// The same invocation, pointed at another checkout of the same
    /// repository. A worktree shares `refs/ank/` with the checkout that made
    /// it, which is why a question about a ref can never be answered from a
    /// working tree.
    fn ank_at(&self, agent: &str, args: &[&str], repo: &Path) -> Output {
        Command::new(ANK)
            .args(args)
            .arg("--repo")
            .arg(repo)
            .env("ANK_AGENT", agent)
            .current_dir(std::env::temp_dir())
            .output()
            .expect("the binary must have been built")
    }

    /// The same invocation with extra environment variables set.
    ///
    /// What the color rule of §4 reads, besides the terminal itself, is the
    /// environment — so the environment has to be under the test's control
    /// rather than inherited from whoever is running the suite. A developer
    /// with `NO_COLOR` exported would otherwise be testing a different rule
    /// from CI, and both would pass.
    fn ank_env(&self, agent: &str, args: &[&str], env: &[(&str, Option<&str>)]) -> Output {
        let mut c = Command::new(ANK);
        c.args(args)
            .arg("--repo")
            .arg(&self.0)
            .env("ANK_AGENT", agent)
            .current_dir(std::env::temp_dir());
        for (key, value) in env {
            match value {
                Some(v) => c.env(key, v),
                None => c.env_remove(key),
            };
        }
        c.output().expect("the binary must have been built")
    }

    /// The same invocation with `$EDITOR` under the test's control. `None`
    /// removes it, which is the environment failure §4 specifies — and removing
    /// it rather than trusting its absence matters, because the developer
    /// running this suite very probably has one exported.
    fn ank_edit(&self, agent: &str, args: &[&str], editor: Option<&str>) -> Output {
        let mut c = Command::new(ANK);
        c.args(args)
            .arg("--repo")
            .arg(&self.0)
            .env("ANK_AGENT", agent)
            .current_dir(std::env::temp_dir());
        match editor {
            Some(e) => c.env("EDITOR", e),
            None => c.env_remove("EDITOR"),
        };
        c.output().expect("the binary must have been built")
    }

    fn config_text(&self) -> String {
        std::fs::read_to_string(self.0.join(".ank/config.yml")).unwrap()
    }

    fn set_config(&self, text: &str) {
        std::fs::write(self.0.join(".ank/config.yml"), text).unwrap();
    }

    /// A non-interactive editor that saves `text`.
    ///
    /// `cp` is the exact shape of one: `edit` appends the file to open as the
    /// last word of the command line, so `EDITOR="cp <replacement>"` runs
    /// `cp <replacement> <scratch>` and the editor has saved. It ships beside
    /// git on all three platforms, which is the reason `enable_signing` reaches
    /// for `ssh-keygen`.
    fn editor_saving(&self, text: &str) -> String {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let p = self
            .0
            .join(format!("saved-{}.md", SEQ.fetch_add(1, Ordering::Relaxed)));
        std::fs::write(&p, text).unwrap();
        format!("cp '{}'", p.display())
    }
}

/// An editor that fills a `new` template in, rather than replacing it.
///
/// It has to transform: the id is minted by `new` before the editor runs and is
/// refused if it comes back changed, so the wholesale `cp` that serves `edit`
/// cannot serve here.
///
/// A shell function, and not `sed -i`, because `-i` is where the three
/// platforms part company: it takes a mandatory backup suffix on the BSD sed
/// macOS ships, so `sed -i -e ...` consumes `-e` as the suffix there and the
/// suite would pass on two runners and fail on the third. Redirecting into a
/// second file and moving it is POSIX everywhere. The path arrives as `$1`
/// because `sh -c 'f; f <path>'` binds it — which is also what makes this read
/// like an editor rather than a fixture.
fn editor_filling(title: &str, scope: &str, body: &str) -> String {
    format!(
        r#"f() {{ sed -e "s|^title:.*|title: {title}|" -e "s|^scope: .*|scope: [\"{scope}\"]|" "$1" > "$1.new"; printf "%s\n" "" "{body}" >> "$1.new"; mv "$1.new" "$1"; }}; f"#
    )
}

/// The same, plus one arbitrary substitution — for the tests that need to move
/// a field the template does not invite anyone to touch.
fn editor_filling_and(title: &str, scope: &str, extra: &str) -> String {
    format!(
        r#"f() {{ sed -e "s|^title:.*|title: {title}|" -e "s|^scope: .*|scope: [\"{scope}\"]|" -e "{extra}" "$1" > "$1.new"; mv "$1.new" "$1"; }}; f"#
    )
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn code(out: &Output) -> i32 {
    out.status
        .code()
        .expect("the process must exit, not signal")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

const ID: &str = "TASK-000000000001";

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// The freeze stops being declared and becomes verifiable. Through the binary
/// on purpose: "check reports a divergence" is a statement about the process,
/// and the anchor it compares against lives in git rather than in the corpus.
#[test]
fn an_edited_constraint_is_reported_altered_and_stops_being_injected() {
    const ADR: &str = "ADR-0000000000ab";
    let r = Repo::new();
    r.enable_signing();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    r.seed_task(ID, Some("A verifiable criterion."));
    // A scope matching nothing is a fault on an ADR, and it would exit 8 before
    // the comparison under test ever ran.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);

    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // Ratified and untouched: the comparison must be silent, or the finding
    // below would prove nothing.
    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        !stdout(&out).contains("altered"),
        "an intact freeze reports nothing: {}",
        stdout(&out)
    );

    // The constraint moves in the file. The anchor recorded in the signed
    // commit cannot follow it without another signature.
    let edited = r.adr_text(ADR).replace("Do not do X.", "Do not do Y.");
    std::fs::write(r.0.join(".ank/adr").join(format!("{ADR}.md")), edited).unwrap();

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 8, "a divergence is a fault: {}", stderr(&out));
    assert!(
        stdout(&out).contains("altered since ratification"),
        "{}",
        stdout(&out)
    );

    // And it stops binding. Reporting it while still injecting it would leave
    // whoever edited the file rewriting the rule every agent works under.
    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("altered since ratification"),
        "the warning section 3 requires: {}",
        stdout(&out)
    );
    assert!(
        !stdout(&out).contains("Do not do Y."),
        "the altered constraint is withheld: {}",
        stdout(&out)
    );
}

/// Declares the key `enable_signing` generated, so the signature is judged at
/// all. Without the file there is no allowlist and §8 puts the corpus in
/// advisory mode, where every verdict is `None` — which is the shape of silence
/// this test exists to refuse.
fn declare_signing_key(r: &Repo) {
    let pub_key = std::fs::read_to_string(r.0.join("signing-key.pub")).unwrap();
    std::fs::write(
        r.0.join(".ank/allowed_signers"),
        format!("test@ank.local {}", pub_key.trim()),
    )
    .unwrap();
}

/// The incident of TASK-1ea38a17d854, reproduced at the only altitude that
/// would have caught it. GitHub rebased a ratification commit while merging a
/// pull request: same tree, same message, same anchor, signature gone. Anyone
/// who can push a branch can produce that commit, and the anchor it carries is
/// then worth nothing.
///
/// Through the binary, and that is the whole point. `signature_state` already
/// answered `Absent` in a unit test, and `check` already turned `Absent` into a
/// fault in another — and the binary measured on the real corpus still exited
/// 0, because the binary in hand was older than the check itself. Nothing
/// asserted that a process invocation runs the wiring end to end, so nothing
/// noticed that one wasn't.
///
/// The negative is only worth what the positive above it is worth: a real
/// `accept` runs first and must be silent, or an exit 8 here would prove
/// nothing but a broken fixture.
#[test]
fn a_ratification_commit_stripped_of_its_signature_is_a_fault_through_the_binary() {
    const ADR: &str = "ADR-0000000000cd";
    let r = Repo::new();
    r.enable_signing();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    // A scope matching nothing is a fault of its own, and it would exit 8
    // before the signature was ever consulted.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    declare_signing_key(&r);
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);

    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let ratification = r.head();

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(
        code(&out),
        0,
        "a real ratification must be silent, or the fault below proves nothing: {}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(!stdout(&out).contains("not signed"), "{}", stdout(&out));

    // The rewrite. `--amend` keeps the message and the tree and drops the
    // signature, which is exactly what the merge did to f770c98.
    r.git(&[
        "-c",
        "commit.gpgsign=false",
        "commit",
        "--amend",
        "--no-edit",
        "-q",
    ]);
    assert_ne!(
        r.head(),
        ratification,
        "the commit must have been rewritten"
    );

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(
        code(&out),
        8,
        "an unsigned ratification is a fault: {}{}",
        stdout(&out),
        stderr(&out)
    );
    let said = stdout(&out);
    assert!(
        said.contains(ADR) && said.contains("not signed"),
        "the finding names the ADR and says what is wrong: {said}"
    );

    // And it is that finding and no other. The anchor still agrees with the
    // file and the commit is still reachable, so a report crying divergence or
    // unverifiability here would be describing a different repository.
    assert!(
        !said.contains("altered since ratification") && !said.contains("is reachable"),
        "only the signature is gone: {said}"
    );
}

/// `ank status` answers where am I, in one command (TASK-15336a0012d5).
///
/// The four scenarios the task names, in order: orientation with no claim,
/// execution with one, the ratification queue and an unmerged completion seen
/// from the default branch, and the degraded case with no default branch at all.
///
/// Through the binary, and the degraded case is the reason: `status` is what an
/// agent runs when it does not know where it is, so every one of its inputs can
/// be missing — no claim, no remote, no default branch, no commit — and each has
/// to be a line rather than an error. That is a property of the process, not of
/// the functions it composes.
#[test]
fn status_answers_where_am_i_in_every_state() {
    const ADR: &str = "ADR-00000000f001";
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    r.seed_task(ID, Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);

    // 1. Orientation: no claim, and the whole repository as the perimeter.
    let out = r.ank("claude-code@ank", &["status"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);
    assert!(said.starts_with("branch main (default main)"), "{said}");
    assert!(said.contains("no claim"), "{said}");
    assert!(said.contains("the whole repository"), "{said}");
    // A proposed ADR is the ratification queue.
    assert!(said.contains("queue 1 proposal(s)"), "{said}");
    assert!(said.contains("corpus 0 fault(s)"), "{said}");
    // Ends with the command to run next (§4), and it is the one this state
    // calls for rather than a fixed string.
    assert!(
        said.trim_end().ends_with("> ank context"),
        "orientation points at context: {said}"
    );

    // 2. Execution: the claim, its expiry, and the perimeter it implies.
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    let out = r.ank("claude-code@ank", &["status"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);
    assert!(said.contains(&format!("claim {ID}")), "{said}");
    assert!(
        said.contains("expires 20"),
        "the expiry is what makes a claim actionable: {said}"
    );
    assert!(
        said.contains(&format!("the scope of {ID}")),
        "under a claim the perimeter is the task's own scope: {said}"
    );
    assert!(
        said.trim_end().ends_with("> ank done"),
        "holding a claim points at done: {said}"
    );

    // The claim of another agent is not this agent's claim.
    let out = r.ank("codex@host-9", &["status"]);
    assert!(stdout(&out).contains("no claim"), "{}", stdout(&out));

    // 3. Accepted, so the queue empties and the constraint starts counting.
    r.enable_signing();
    assert_eq!(
        code(&r.ank("claude-code@ank", &["release", "--reason", "x"])),
        0
    );
    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(said.contains("queue 0 proposal(s)"), "{said}");
    assert!(said.contains("1 constraint(s)"), "{said}");

    // 4. Degraded: no default branch and no remote to infer one from. It is a
    // warning and a full report, never a refusal.
    let bare = Repo::new();
    std::fs::write(
        bare.0.join(".ank/config.yml"),
        "schema: 1\nclaim_ttl_max: 2h\n",
    )
    .unwrap();
    bare.seed_task(ID, Some("A verifiable criterion."));
    std::fs::create_dir_all(bare.0.join("src")).unwrap();
    std::fs::write(bare.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    bare.git(&["add", "-A"]);
    bare.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);
    let out = bare.ank("claude-code@ank", &["status"]);
    assert_eq!(
        code(&out),
        0,
        "a missing default branch degrades, it does not refuse: {}",
        stderr(&out)
    );
    let said = stdout(&out);
    assert!(said.contains("warning: no default branch"), "{said}");
    assert!(
        said.contains("default_branch"),
        "the warning carries the fix: {said}"
    );
    assert!(
        said.contains("corpus "),
        "and the rest of the report still arrives: {said}"
    );

    // `--json` is available on every command without exception (§4).
    let out = r.ank("claude-code@ank", &["status", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let j = stdout(&out);
    assert!(j.contains("\"branch\":\"main\""), "{j}");
    assert!(j.contains("\"claim\":null"), "{j}");
    assert!(j.contains("\"queue\":0"), "{j}");
}

/// `ank graph` draws the `blocked_by` DAG (TASK-253e897d3330).
///
/// §5's ordering already walks these edges to count what a task unblocks; this
/// makes the same structure visible to a reader. Through the binary, because the
/// criterion is about what the process prints.
///
/// The chain is built so that every clause has something to be wrong about: a
/// root, two levels under it, a diamond, and a task whose only blocker sits
/// outside the perimeter.
#[test]
fn graph_draws_the_dag_and_names_the_perimeter() {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::create_dir_all(r.0.join("docs")).unwrap();
    std::fs::write(r.0.join("docs/guide.md"), "# guide\n").unwrap();

    // root <- mid <- leaf, and `side` also blocked by root: a diamond only in
    // the sense that `leaf` is reachable twice once `side` blocks it too.
    let root = "TASK-000000000a01";
    let mid = "TASK-000000000a02";
    let leaf = "TASK-000000000a03";
    let outer = "TASK-000000000a04";
    let held = "TASK-000000000a05";
    for id in [root, mid, leaf, held] {
        r.seed_task(id, Some("A verifiable criterion."));
    }
    r.blocked(mid, &[root]);
    r.blocked(leaf, &[mid]);
    // Scoped to docs/, so it is outside the src/ perimeter below.
    r.seed_task_scoped(outer, "docs/**");
    r.blocked(held, &[outer]);

    let out = r.ank("claude-code@ank", &["graph", "src"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);

    // Names the perimeter it drew (§4).
    assert_eq!(
        said.lines().next().unwrap_or_default(),
        "src",
        "the perimeter is named: {said}"
    );

    // The shape: each level one connector deeper than what blocks it.
    //
    // Measured over the drawing alphabet of §4 rather than over whitespace. The
    // claim is unchanged — a01 flush left, a02 under it, a03 under that — but
    // `trim_start` answered it by counting spaces, and a row now begins with a
    // connector. It would have read zero for every indented line and gone on
    // passing for the wrong reason.
    let depth = |needle: &str| -> usize {
        let line = said
            .lines()
            .find(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("{needle} missing from:\n{said}"));
        let drawing = line
            .chars()
            .take_while(|c| matches!(c, ' ' | '│' | '├' | '└' | '─'))
            .count();
        // Reported in the old unit so the numbers below still say what they
        // said: four columns per level, two before.
        drawing / 2
    };
    assert_eq!(depth("000000000a01"), 0, "the root is flush left: {said}");
    assert_eq!(depth("000000000a02"), 2, "{said}");
    assert_eq!(depth("000000000a03"), 4, "{said}");

    // The connectors themselves, not only their width: a level drawn with the
    // right number of blanks and no glyph would satisfy every count above.
    let row = |needle: &str| -> &str {
        said.lines()
            .find(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("{needle} missing from:\n{said}"))
    };
    assert!(
        row("000000000a02").starts_with("└── "),
        "the only child of the root is drawn as the last one: {said}"
    );
    assert!(
        row("000000000a03").starts_with("    └── "),
        "and its child continues under a cleared gutter: {said}"
    );

    // A blocker outside the perimeter is never silently dropped: drawing this
    // one flush left with no mark would say nothing is stopping it.
    let held_line = said
        .lines()
        .find(|l| l.contains("000000000a05"))
        .unwrap_or_default();
    assert!(
        held_line.contains("outside"),
        "a task held by something outside the perimeter says so: {said}"
    );
    assert!(
        !said.contains("000000000a04"),
        "and the outside task itself is not drawn: {said}"
    );

    // Says so explicitly when the perimeter holds no task (§4). `LICENSE` and
    // not `docs/nowhere`: the latter is *inside* `docs/**`, which is the whole
    // point of `overlaps_dir`, and the fixture would have been testing nothing.
    let out = r.ank("claude-code@ank", &["graph", "LICENSE"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("no task in this perimeter"),
        "{}",
        stdout(&out)
    );

    // `--json` for the raw edges (§4) — including the edge leaving the
    // perimeter, which the drawing can only mark and not show.
    let out = r.ank("claude-code@ank", &["graph", "src", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let j = stdout(&out);
    assert!(j.contains("\"path\":\"src\""), "{j}");
    assert!(
        j.contains(&format!("{{\"task\":\"{mid}\",\"blocked_by\":\"{root}\"}}")),
        "{j}"
    );
    assert!(
        j.contains(&format!(
            "{{\"task\":\"{held}\",\"blocked_by\":\"{outer}\"}}"
        )),
        "the raw edges include the one leaving the perimeter: {j}"
    );
}

/// A cycle is a corpus fault `check` reports, and `graph` still has to draw the
/// repository that has one rather than hang on it.
///
/// **Two cycles, because they take different paths through the code and only
/// one exercises the guard.** A cycle with no way in has no root, so nothing
/// recurses into it and it is collected flat at the end. A cycle reachable from
/// a root is walked, and only there does the visited-path check stand between
/// the drawing and a stack overflow. Removing that check left the first case
/// passing, which is how this test was found to be weaker than it looked — by a
/// mutation run, not by reading it.
#[test]
fn graph_terminates_on_a_cycle_and_says_where_it_is() {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    // Closed on itself and unreachable: no root, so nothing recurses into it.
    let a = "TASK-000000000b01";
    let b = "TASK-000000000b02";
    // A root leading into a cycle: this is the one that recurses, and the only
    // one the visited-path guard stands in front of.
    let root = "TASK-000000000b03";
    let mid = "TASK-000000000b04";
    let back = "TASK-000000000b05";
    for id in [a, b, root, mid, back] {
        r.seed_task(id, Some("A verifiable criterion."));
    }
    r.blocked(a, &[b]);
    r.blocked(b, &[a]);
    r.blocked(mid, &[root, back]);
    r.blocked(back, &[mid]);

    let out = r.ank("claude-code@ank", &["graph", "src"]);
    assert_eq!(
        code(&out),
        0,
        "a cycle is check's fault to report, not graph's: {}",
        stderr(&out)
    );
    let said = stdout(&out);

    // Reachable: walked, and stopped at the repetition rather than at the stack.
    assert!(
        said.contains("(cycle)"),
        "the walk names where it turned back: {said}"
    );
    assert!(
        said.contains("000000000b04") && said.contains("000000000b05"),
        "{said}"
    );

    // Unreachable: under no root, so it would vanish under a header with
    // nothing beneath it. Drawn, and named as the reason.
    assert!(said.contains("under no root"), "{said}");
    assert!(
        said.contains("000000000b01") && said.contains("000000000b02"),
        "{said}"
    );
}

/// `ank show` on a task says what it unblocks, not only what blocks it
/// (TASK-2415ddb92df8).
///
/// The same derivation `graph` draws, narrowed to one entity. Through the
/// binary, because the criterion is about what the process prints — and because
/// the entity has to still come out whole above the sections, which is a
/// property of the bytes on the pipe and of nothing else.
///
/// The fixture gives every clause something to be wrong about: a blocker that
/// is already `done`, two tasks waiting on the same one, a blocker naming an
/// entity the corpus does not hold, and a leaf with neither direction.
#[test]
fn show_lists_what_a_task_unblocks_alongside_its_blockers() {
    let r = Repo::new();
    let root = "TASK-000000000c01";
    let mid = "TASK-000000000c02";
    let leaf = "TASK-000000000c03";
    let side = "TASK-000000000c04";
    let ghost = "TASK-0000000000ff";
    for id in [root, mid, leaf, side] {
        r.seed_task(id, Some("A verifiable criterion."));
    }
    // One finished in each direction. A `done` entity keeps its line and
    // carries its status, which is what makes the status on each line worth
    // printing — and it is the difference between this list and §5's count,
    // which drops what is no longer held up because it is ordering work.
    for id in [root, side] {
        let done = r.task_text(id).replace("status: open", "status: done");
        std::fs::write(r.0.join(".ank/tasks").join(format!("{id}.md")), done).unwrap();
    }
    r.blocked(mid, &[root, ghost]);
    r.blocked(leaf, &[mid]);
    r.blocked(side, &[mid]);

    let before = r.task_text(mid);
    let out = r.ank("claude-code@ank", &["show", mid]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);

    // The entity first, whole and unreformatted. `show` is still the one
    // command that does not summarise; the sections are appended under it.
    assert!(
        said.starts_with(&before),
        "the entity is no longer verbatim:\n{said}"
    );
    let derived = &said[before.len()..];

    // One line each, with status, in both directions.
    assert!(derived.contains("BLOCKED BY (2)"), "{said}");
    assert!(
        derived.contains(&format!("{root}  [done]")),
        "a blocker keeps its line and its status once done: {said}"
    );
    assert!(
        derived.contains(&format!("{ghost}  (no such entity)")),
        "a dangling blocker is named, never dropped: {said}"
    );
    assert!(derived.contains("UNBLOCKS (2)"), "{said}");
    let at = |needle: &str| derived.find(needle).unwrap_or_else(|| panic!("{said}"));
    assert!(
        derived.contains(&format!("{side}  [done]")),
        "a task that waited and is now done keeps its line: the list is the edge \
         set, not the count of what is still held up: {said}"
    );
    assert!(
        at(&format!("{leaf}  [open]")) < at(&format!("{side}  [done]")),
        "the derived direction is ordered by id, like every other listing: {said}"
    );
    // The two directions are not one list: `mid` is blocked by `root` and
    // unblocks `leaf`, and neither may appear under the other heading.
    let (blockers, unblocks) = derived.split_at(at("UNBLOCKS"));
    assert!(!blockers.contains(leaf), "{said}");
    assert!(!unblocks.contains(root), "{said}");

    // A leaf has neither direction, and both headings still print: an absent
    // heading and a heading with nothing under it are not the same answer.
    let out = r.ank("claude-code@ank", &["show", leaf]);
    let said = stdout(&out);
    assert!(said.contains("BLOCKED BY (1)"), "{said}");
    assert!(said.contains("UNBLOCKS (0)"), "{said}");

    // Derived at read time and stored nowhere: a task that did not exist at the
    // first read appears at the second, and the file `show` printed from is
    // byte for byte what it was.
    let late = "TASK-000000000c05";
    r.seed_task(late, Some("A verifiable criterion."));
    r.blocked(late, &[mid]);
    let out = r.ank("claude-code@ank", &["show", mid]);
    let said = stdout(&out);
    assert!(
        said.contains("UNBLOCKS (3)") && said.contains(&format!("{late}  [open]")),
        "the reverse edge is derived, not stored: {said}"
    );
    assert_eq!(
        r.task_text(mid),
        before,
        "show wrote the derivation into the entity it was asked to print"
    );

    // `--json` carries the same two lists, and the null of an unresolved one.
    let out = r.ank("claude-code@ank", &["show", mid, "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let j = stdout(&out);
    assert!(
        j.contains(&format!(
            "{{\"id\":\"{root}\",\"short\":\"{root}\",\"status\":\"done\""
        )),
        "{j}"
    );
    assert!(
        j.contains(&format!(
            "{{\"id\":\"{ghost}\",\"short\":\"{ghost}\",\"status\":null,\"title\":null}}"
        )),
        "{j}"
    );
    assert!(j.contains("\"unblocks\":[{"), "{j}");
}

/// One directory, one answer, however it is typed (TASK-df4c39031583).
///
/// Reported from a Windows shell against the real corpus. `docs` answered five
/// live constraints, `docs\` answered **four**, `.\docs\` answered zero. The
/// zeros were survivable because they were obvious; the four was not. It is what
/// tab-completion produces on Windows, it looks like a correct answer, and it
/// silently dropped a rule that binds — because the argument reached glob
/// matching verbatim, where a `**` glob still matched a backslash as an ordinary
/// character and a glob naming a segment did not.
///
/// Through the binary and across all four path-taking verbs, because the defect
/// was never in the matcher: it was in what each verb handed to it, and three of
/// them had their own copy of the handing.
///
/// **No claim is held in this fixture, and that is load-bearing.** `context`
/// with a claim is in execution mode and ignores the path entirely (§5), so a
/// comparison made while holding one would pass whatever the code did. I made
/// that exact mistake measuring the fix by hand.
#[test]
fn a_directory_resolves_the_same_however_the_path_is_written() {
    const ADR: &str = "ADR-00000000dddd";
    let r = Repo::new();
    r.enable_signing();
    std::fs::create_dir_all(r.0.join("docs")).unwrap();
    std::fs::write(r.0.join("docs/guide.md"), "# guide\n").unwrap();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    // One entity inside the perimeter, one outside: an answer that ignored the
    // path would match both and look like success.
    r.seed_adr(ADR, "Do not do X.", "docs/**");
    r.seed_task(ID, Some("A verifiable criterion."));
    // A dead scope under `docs/` and nowhere else. `check` reports corpus-wide
    // totals whatever the perimeter, so without a finding that exists on one
    // side and not the other it answers identically for every path — and the
    // comparison below would be measuring nothing. The assertion caught that too.
    r.seed_adr("ADR-00000000eeee", "Do not do Y.", "docs/missing/**");
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);
    // Ratified, or `review` lists no live constraint at all and the two
    // perimeters answer identically for a reason that has nothing to do with
    // the path. The assertion below catches exactly that, and did.
    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let forms = [
        "docs",
        "docs/",
        "docs\\",
        "./docs",
        ".\\docs\\",
        "docs/../docs",
        "docs//",
    ];
    for verb in ["context", "review", "check", "scope"] {
        let base = r.ank("claude-code@ank", &[verb, "docs"]);
        let expected = stdout(&base);
        let expected_code = code(&base);
        for form in forms {
            let got = r.ank("claude-code@ank", &[verb, form]);
            assert_eq!(
                code(&got),
                expected_code,
                "`ank {verb} {form}` exits differently from `ank {verb} docs`: {}",
                stderr(&got)
            );
            assert_eq!(
                stdout(&got),
                expected,
                "`ank {verb} {form}` must answer as `ank {verb} docs` does"
            );
        }
        // And the perimeter is real: the ADR lives under docs/, the task under
        // src/, so the two paths must not give the same answer.
        let elsewhere = stdout(&r.ank("claude-code@ank", &[verb, "src"]));
        assert_ne!(
            elsewhere, expected,
            "`ank {verb}` gives one answer for two different perimeters, so \
             comparing path forms proves nothing"
        );
    }

    // `scope` echoes the perimeter it drew, and it has to echo the one it used.
    let out = r.ank("claude-code@ank", &["scope", ".\\docs\\"]);
    assert_eq!(
        stdout(&out).lines().next().unwrap_or_default(),
        "docs",
        "the echoed path is the normalised one: {}",
        stdout(&out)
    );

    // A path that leaves the repository has no answer, and saying nothing would
    // be the silently-partial set all over again.
    for outside in ["/etc/passwd", "..", "../sibling", "docs/../../elsewhere"] {
        let out = r.ank("claude-code@ank", &["review", outside]);
        assert_eq!(code(&out), 1, "{outside} must be refused: {}", stdout(&out));
        let err = stderr(&out);
        assert!(err.starts_with("error[1]:"), "{err}");
        assert!(
            err.contains("ank review"),
            "a refusal always says what to run next: {err}"
        );
    }
}

/// `ank scope <path>` answers what covers a path (TASK-e717ee625c5c).
///
/// The `check-ignore` of ank: a glob that matches nothing is otherwise only
/// discovered through `check`, after the entity is written and already
/// invisible. Through the binary, because the criterion is about what the
/// process prints and because dispatch reaching a new verb is the thing that
/// has silently failed here before (TASK-45d18f45de2c).
#[test]
fn scope_says_what_covers_a_path_and_says_when_nothing_does() {
    const ADR: &str = "ADR-00000000c0de";
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    r.seed_task(ID, Some("A verifiable criterion."));

    let out = r.ank("claude-code@ank", &["scope", "src/main.rs"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);

    // Grouped by type, one line each, carrying the status.
    assert!(said.contains("ADR (1)"), "{said}");
    assert!(said.contains("TASKS (1)"), "{said}");
    assert!(said.contains("[proposed] A decision"), "{said}");
    assert!(said.contains("[open] Example task"), "{said}");
    assert!(
        said.lines()
            .next()
            .unwrap_or_default()
            .contains("src/main.rs"),
        "it names the perimeter it drew: {said}"
    );

    // The same resolution `context` binds with. A directory the globs reach
    // under is covered, which is what `overlaps_dir` means and what an agent
    // asking about a package rather than a file needs.
    let out = r.ank("claude-code@ank", &["scope", "src"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("ADR (1)"), "{}", stdout(&out));

    // Nothing matching is said, never left to an empty answer: silence reads as
    // "nothing constrains this", which is the same sentence as "ank could not
    // tell", and only one of the two is safe to act on.
    let out = r.ank("claude-code@ank", &["scope", "docs/whitepaper.md"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("nothing covers this path"),
        "{}",
        stdout(&out)
    );

    // It answers about a path, not about the filesystem: a file that does not
    // exist yet resolves like any other, which is the point of asking before
    // writing the entity.
    let out = r.ank("claude-code@ank", &["scope", "src/not_written_yet.rs"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("ADR (1)"), "{}", stdout(&out));

    // `--json` is available on every command without exception (§4).
    let out = r.ank("claude-code@ank", &["scope", "src/main.rs", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let j = stdout(&out);
    assert!(j.contains("\"path\":\"src/main.rs\""), "{j}");
    assert!(j.contains("\"total\":2"), "{j}");
    assert!(j.contains("\"adr\":[{"), "{j}");
    assert!(j.contains("\"tasks\":[{"), "{j}");

    // A missing path is a refusal carrying the command to run next, not a
    // silent listing of the whole corpus.
    let out = r.ank("claude-code@ank", &["scope"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("ank scope <path>"),
        "{}",
        stderr(&out)
    );
}

/// The stamp follows the commit, including a commit that changes no file
/// (TASK-0b26c8b5bfc5).
///
/// The first `build.rs` emitted no `rerun-if-changed`, so Cargo watched the
/// package instead and a commit touching no file in it never reran the script.
/// Measured on the build that shipped `--version`: `HEAD 3110392`, stamp
/// `aeb1841`. A diagnostic that is sometimes wrong is one people learn to
/// re-verify by hand.
///
/// **Why a fixture crate and not `ank` itself.** The claim is about when Cargo
/// reruns the build script, and that needs a real `cargo build` around a real
/// commit — but rebuilding `ank` from inside its own test suite relinks
/// `target/debug/ank.exe` while other tests are executing it, which is the
/// Windows relink trap the development guide documents, and a fresh target
/// directory would rebuild `libsqlite3-sys` on every CI job of all three
/// platforms. So the fixture is a trivial crate driving **the real
/// `build.rs`** — the file under test, not a copy of its logic — through a real
/// cargo build and a real git commit. What that leaves unexercised is ank's own
/// linking, which the claim does not depend on. The literal measurement on
/// `ank --version` was taken by hand and recorded in the task log.
#[test]
fn the_stamp_follows_a_commit_that_changes_no_file() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("build.rs");

    let dir = std::env::temp_dir().join(format!("ank-stamp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::copy(&script, dir.join("build.rs")).unwrap();
    // An empty `[workspace]` so the fixture is not adopted by any workspace it
    // happens to sit under, and the one build-dependency the real script has:
    // it hashes SKILL.md with `ank_core::freeze_hash_short` (TASK-ecda4070354f),
    // and the fixture drives the real script rather than a copy of its logic, so
    // it has to build like the real one. By path, and the builds below are
    // `--offline`, so this still needs no registry beyond the cache the
    // workspace build has already warmed.
    let core = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ank-core")
        .to_string_lossy()
        .replace('\\', "/");
    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[workspace]\n\n[package]\nname = \"stamp\"\nversion = \"0.0.0\"\n\
             edition = \"2021\"\n\n[build-dependencies]\n\
             ank-core = {{ path = \"{core}\" }}\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("src/main.rs"),
        "fn main() { println!(\"{}\", env!(\"ANK_COMMIT\")); }\n",
    )
    .unwrap();

    let git = |args: &[&str]| {
        let out = Command::new("git")
            .current_dir(&dir)
            .args(args)
            .output()
            .expect("git must be installed");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@ank.local"]);
    git(&["config", "user.name", "Test"]);
    git(&["add", "-A"]);
    git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);

    let build_and_read = || {
        let out = Command::new(&cargo)
            .current_dir(&dir)
            .args(["run", "-q", "--offline"])
            .env("CARGO_TARGET_DIR", dir.join("target"))
            .output()
            .expect("cargo must be on PATH");
        assert!(
            out.status.success(),
            "cargo run: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };

    let first = build_and_read();
    assert_eq!(
        first,
        git(&["rev-parse", "--short", "HEAD"]),
        "the stamp must name the commit it was built at"
    );

    // The case the old script missed: the commit moves, no file does.
    git(&[
        "-c",
        "commit.gpgsign=false",
        "commit",
        "-q",
        "--allow-empty",
        "-m",
        "nothing changes but the commit",
    ]);
    let moved = git(&["rev-parse", "--short", "HEAD"]);
    assert_ne!(moved, first, "the fixture must actually have moved");

    let second = build_and_read();
    assert_eq!(
        second, moved,
        "a rebuild after a commit that touches no file must name the new HEAD, \
         not the one before it"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// The binary can say what it is, and can say it where nothing else works
/// (TASK-548c518cb705).
///
/// TASK-1ea38a17d854 cost a full investigation to conclude that the binary in
/// hand predated the feature it was being measured for. The answer was a file
/// timestamp. One command should have said it.
///
/// The second half is the half that matters and the one a unit test cannot
/// reach: `--version` has to answer *before* the repository is resolved, git is
/// checked and the config is loaded. A version flag that needs a healthy
/// environment is mute in the one situation it exists for, and only a process
/// launched somewhere hostile proves it is not.
#[test]
fn the_version_answers_before_anything_can_stop_it() {
    let r = Repo::new();

    let out = r.ank("claude-code@ank", &["--version"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out).trim().to_string();
    assert!(
        said.starts_with(&format!("ank {}", env!("CARGO_PKG_VERSION"))),
        "the crate version, so a build can be placed against a release: {said}"
    );
    assert!(
        said.contains('(') && said.ends_with(')'),
        "and the commit it was built from: {said}"
    );
    // Empty parentheses would satisfy the shape and answer nothing. The commit
    // is the first value inside them; the skill revision beside it is
    // TASK-ecda4070354f's and is asserted against the file itself in
    // tests/skill.rs, which is the only place that comparison means anything.
    let inside = said
        .rsplit_once('(')
        .and_then(|(_, c)| c.strip_suffix(')'))
        .unwrap_or("");
    let commit = inside.split(',').next().unwrap_or("").trim();
    assert!(
        commit.len() >= 4,
        "a sha or the word `unknown`, never a blank: {said}"
    );
    assert!(
        inside.contains("skill "),
        "and the skill revision it was built alongside: {said}"
    );

    // Nowhere in particular: no `.ank/`, no git repository, no config. This is
    // where a stale or unidentified binary is actually met.
    let nowhere = std::env::temp_dir().join("ank-cli-it-nowhere");
    std::fs::create_dir_all(&nowhere).unwrap();
    let bare = Command::new(ANK)
        .arg("--version")
        .current_dir(&nowhere)
        .output()
        .expect("the binary must have been built");
    assert_eq!(
        code(&bare),
        0,
        "outside any repository it must still answer: {}",
        stderr(&bare)
    );
    assert_eq!(
        stdout(&bare).trim(),
        said,
        "and answer the same thing, since it describes the binary and not the place"
    );
    let _ = std::fs::remove_dir_all(&nowhere);

    // Discoverable, or it answers a question nobody knows how to ask.
    let out = r.ank("claude-code@ank", &["help"]);
    assert!(
        stdout(&out).contains("--version"),
        "help must name it: {}",
        stdout(&out)
    );
}

/// Git refusing to answer is an answer, and it used to be silence
/// (TASK-c92b7cc10f13).
///
/// `signature_state` ended in `.ok()?`, so any failure of the git invocation
/// became `None` — the one verdict `check_adr` says nothing about. An ADR whose
/// signature could not be read was indistinguishable from one in a corpus that
/// declares no key at all, which is the degradation to success ADR-6b3f refuses
/// and the `Unchecked` counter already exists to prevent one level down.
///
/// The lever is a `gpg.format` git rejects, one of the causes the task named.
/// It is surgical: `rev-list --full-history`, `cat-file`, `rev-parse` and
/// `for-each-ref` all still answer, so the corpus is read normally and the
/// ratification commit is still reached — only the signature read exits
/// non-zero. That is exactly the state under test, and no other.
///
/// A signal and not a fault, deliberately: a broken environment is not a forged
/// ratification, and exiting 8 over one would fail the `check-repo` verifier of
/// every task on a machine missing nothing but a working gpg config.
#[test]
fn a_signature_git_cannot_read_is_reported_rather_than_passed_over() {
    const ADR: &str = "ADR-0000000000ef";
    let r = Repo::new();
    r.enable_signing();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    declare_signing_key(&r);
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);

    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // Readable and trusted: the silence below has to be broken by the config
    // change and by nothing else.
    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        !stdout(&out).contains("could not be read"),
        "{}",
        stdout(&out)
    );

    r.git(&["config", "gpg.format", "bogus"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = stdout(&out);
    assert!(
        said.contains("could not be read"),
        "a signature git refuses to answer about must be reported: {said}"
    );
    assert!(
        said.contains("neither verified nor refused"),
        "and it must say what that leaves undecided: {said}"
    );
    assert!(
        said.contains("gpg.format"),
        "carrying git's own reason, or the reader cannot act on it: {said}"
    );

    // Not a fault, and not confused with the missing-key case either.
    assert_eq!(
        code(&out),
        0,
        "a broken environment is not a forgery: {said}"
    );
    assert!(
        !said.contains("not signed") && !said.contains("no public key"),
        "an unread signature is not an absent one: {said}"
    );
}

/// A claim is not an orphan just because this checkout is too old to have
/// heard of the task (TASK-52fbffbfdf65).
///
/// Observed three times while diagnosing TASK-1ea38a17d854, each time on a
/// claim being held: a `check` run from a detached worktree at an earlier
/// commit printed `pruned refs/ank/claims/<id>` and deleted it. `refs/ank/` is
/// shared by every worktree, so the older checkout reached into the coordination
/// plane of the current one. A lost claim is a task two agents can hold at once.
///
/// Through the binary, and with the ref read by git afterwards rather than
/// through `claim::read`: what has to be true is the state of the repository,
/// not the module's agreement with itself.
#[test]
fn a_check_from_an_older_checkout_leaves_a_live_claim_alone() {
    let r = Repo::new();
    // An ADR-less corpus still needs its scope to exist, or `check` exits 8 on
    // a dead scope before any of this is reached.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);
    let before_the_task = r.head();

    // The task arrives after that commit, which is the whole point: the older
    // checkout below cannot see it.
    r.seed_task(ID, Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&[
        "-c",
        "commit.gpgsign=false",
        "commit",
        "-qm",
        "add the task",
    ]);

    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(r.claim_ref(ID).is_some(), "the claim must exist to be lost");

    let older = r.0.with_extension("older-checkout");
    r.git(&[
        "worktree",
        "add",
        "--detach",
        older.to_str().unwrap(),
        &before_the_task,
    ]);

    // Another agent, running `check` from that checkout. It sees no tasks at
    // all, and must still not touch a ref it cannot judge.
    let out = r.ank_at("codex@host-9", &["check"], &older);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        !stdout(&out).contains("pruned"),
        "nothing to prune from a checkout that cannot see the corpus: {}",
        stdout(&out)
    );
    assert!(
        r.claim_ref(ID).is_some(),
        "the claim survives a check run from a checkout older than the task"
    );

    r.git(&["worktree", "remove", "--force", older.to_str().unwrap()]);

    // The other half, or the fix would be "never prune". A ref whose task the
    // default branch really has lost is still collected, and the live one is
    // still left alone in the same pass.
    let gone = "TASK-00000000dead";
    r.seed_task(gone, Some("A verifiable criterion."));
    let out = r.ank("codex@host-9", &["claim", gone]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    std::fs::remove_file(r.0.join(".ank/tasks").join(format!("{gone}.md"))).unwrap();
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "drop it"]);

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        r.claim_ref(gone).is_none(),
        "a real orphan is still collected: {}",
        stdout(&out)
    );
    assert!(r.claim_ref(ID).is_some(), "and the live claim still is not");
}

#[test]
fn claiming_through_the_binary_takes_the_ref_and_moves_the_task() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let record = r
        .claim_ref(ID)
        .expect("the ref must exist, read with git and not through the module");
    assert!(record.contains("state: claim"), "{record}");
    assert!(record.contains("claude-code@ank"), "{record}");
    assert!(record.contains("expires:"), "{record}");

    let text = r.task_text(ID);
    assert!(text.contains("status: in_progress"), "{text}");
    assert!(
        !text.contains("claude-code@ank") && !text.contains("expires"),
        "the claim must not reach the file (ADR-4e7c25b1f639):\n{text}"
    );

    // A prefix resolves, the way an agent actually types it.
    let out = r.ank("claude-code@ank", &["claim", "0000"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
}

#[test]
fn the_exit_code_of_a_refusal_reaches_the_process() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("codex@host-9", &["claim", ID])), 0);

    // Held by somebody else: code 4, and the message names the holder.
    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 4, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[4]:"), "{err}");
    assert!(err.contains("codex@host-9"), "{err}");
    assert!(
        err.contains("->"),
        "a refusal always says what to run next: {err}"
    );

    // No criterion: code 7, and the message carries the command that sets one.
    let other = "TASK-00000000ffff";
    r.seed_task(other, None);
    let out = r.ank("claude-code@ank", &["claim", other]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(stderr(&out).contains("--criteria"), "{}", stderr(&out));
    assert!(
        r.claim_ref(other).is_none(),
        "a refusal must leave no ref behind"
    );

    // An unknown id is code 2, which comes from the store and must survive the
    // trip out just the same.
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", "abcd"])), 2);
}

#[test]
fn every_verb_of_the_surface_answers() {
    // This test used to assert the opposite, verb by verb, as each module was
    // still a stub. The last of them landed with the human surface, so what it
    // guards now is that none regresses to `not_implemented` — a rejection is a
    // code 1 carrying that exact phrase, and no verb may produce it.
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task(ID, Some("A criterion."));
    for argv in [
        vec!["context"],
        vec!["find", "criterion"],
        vec!["show", ID],
        vec!["check"],
        vec!["review"],
        vec!["claim", ID],
        vec!["log", "working"],
        vec!["release", "--reason", "handing it over"],
        vec!["attest", ID, "--proof", "assertion:x"],
        vec!["close", ID, "--reason", "not needed after all"],
        vec!["help"],
    ] {
        let out = r.ank("claude-code@ank", &argv);
        let err = stderr(&out);
        assert!(
            !err.contains("not implemented yet"),
            "{argv:?} still answers not_implemented: {err}"
        );
    }
}

#[test]
fn the_foundation_is_crossed_before_the_verb_and_names_its_own_failures() {
    // A --repo pointing nowhere is named as such, never disguised as a broken
    // environment -- that ordering is why `startup` resolves the repository
    // before checking git.
    let out = Command::new(ANK)
        .args(["claim", ID, "--repo"])
        .arg(std::env::temp_dir().join("ank-does-not-exist"))
        .output()
        .unwrap();
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains(".ank"), "{}", stderr(&out));

    // And a parse error never reaches the foundation at all.
    let out = Command::new(ANK)
        .args(["claim", "--tll", "30m"])
        .output()
        .unwrap();
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains("--tll"), "{}", stderr(&out));
}

#[test]
fn json_is_available_on_the_verb_that_exists() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let out = r.ank("claude-code@ank", &["claim", ID, "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.trim().starts_with('{'), "{text}");
    assert!(text.contains(ID), "{text}");
    assert!(text.contains("expires"), "{text}");
}

#[test]
fn context_exits_zero_in_both_modes_and_switches_on_the_claim() {
    // "exits 0 with an explicit message" is a statement about the process, so
    // it is the process that answers it.
    let r = Repo::new();
    r.seed_task(ID, Some("Auth integration tests pass."));

    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("TASKS (1)"), "{text}");
    assert!(text.contains("> ank claim"), "{text}");
    assert!(
        !text.contains("DONE_CRITERIA"),
        "orientation stays broad: {text}"
    );

    // Claiming flips the same command to execution, with no extra argument.
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("DONE_CRITERIA"), "{text}");
    assert!(text.contains("Auth integration tests pass."), "{text}");
    assert!(
        !text.contains("> ank claim"),
        "no other task to offer: {text}"
    );

    // A path argument is ignored while a claim is held, and said so.
    let out = r.ank("claude-code@ank", &["context", "src/"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(code(&out), 0);
    assert!(text.contains("execution context"), "{text}");

    // Another agent, no claim of its own: orientation again, and the task is
    // shown as held rather than offered.
    let out = r.ank("codex@host-9", &["context"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("[claimed:claude-code@ank]"), "{text}");
    assert!(text.contains("no ready tasks in scope"), "{text}");
}

#[test]
fn an_empty_corpus_is_a_clean_stop_and_not_a_breakdown() {
    // An agent in a loop needs a signal it can act on; an empty output reads
    // as a failure and triggers pointless retries.
    let r = Repo::new();
    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("no ready tasks in scope"), "{text}");
    assert!(!text.trim().is_empty());
}

#[test]
fn context_json_reaches_the_process_intact() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    let out = r.ank("claude-code@ank", &["context", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.trim_start().starts_with('{'), "{text}");
    assert!(text.contains("\"mode\":\"orientation\""), "{text}");
    assert!(text.contains("\"ready\":1"), "{text}");
}

#[test]
fn a_done_through_the_binary_leaves_a_completion_ref_naming_commit_and_branch() {
    // Read back with git rather than through the module: what has to be true
    // is the state of the repository, not the module's agreement with itself.
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task_with(ID, Some("A criterion."), &["ok"]);
    let head = r.head();

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    let out = r.ank("claude-code@ank", &["done"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("running: ok ... ok"), "{text}");
    assert!(text.contains("proof recorded:"), "{text}");

    let record = r
        .claim_ref(ID)
        .expect("the ref survives the done, it is not deleted");
    assert!(record.contains("state: completed"), "{record}");
    assert!(record.contains(&head), "the HEAD commit is named: {record}");
    assert!(
        record.contains("branch: main"),
        "the branch is named: {record}"
    );
    assert!(
        !record.contains("expires"),
        "a completion carries no TTL: {record}"
    );

    let file = r.task_text(ID);
    assert!(file.contains("status: done"), "{file}");
    assert!(
        file.contains("type: test"),
        "one proof entry per verifier: {file}"
    );
}

// ---------------------------------------------------------------------------
// Short forms (TASK-f3e92656b5df, ADR-962c25797569)
// ---------------------------------------------------------------------------

/// The short form is the long form, and the output proves it byte for byte.
///
/// Comparing the two invocations against each other rather than against an
/// expected listing is what makes this a test of the parser: it cannot pass
/// because `find` was taught to print something, only because both spellings
/// reached the same flag with the same value.
#[test]
fn a_short_flag_is_the_long_flag_and_takes_its_value_both_ways() {
    let r = Repo::new();
    r.seed_task("TASK-000000000f01", Some("A criterion."));
    r.seed_task("TASK-000000000f02", Some("A criterion."));
    seed_done(
        &r,
        "TASK-000000000f03",
        "  - type: commit\n    ref: abc1234\n",
    );

    let long = r.ank(
        "claude-code@ank",
        &["find", "criterion", "--status", "open"],
    );
    assert_eq!(code(&long), 0, "{}", stderr(&long));
    let expected = stdout(&long);
    assert!(
        expected.contains("TASK-000000000f01") && !expected.contains("TASK-000000000f03"),
        "the fixture must actually filter, or the comparison proves nothing:\n{expected}"
    );

    for argv in [
        vec!["find", "criterion", "-s", "open"],
        vec!["find", "criterion", "-s=open"],
    ] {
        let out = r.ank("claude-code@ank", &argv);
        assert_eq!(code(&out), 0, "{argv:?}: {}", stderr(&out));
        assert_eq!(stdout(&out), expected, "{argv:?} answered differently");
    }

    // And a global one, which is legal on every verb rather than declared per
    // verb -- the path through the parser is the same, the table is not.
    let out = r.ank("claude-code@ank", &["find", "criterion", "-j"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).trim_start().starts_with('{'),
        "-j is --json: {}",
        stdout(&out)
    );
}

/// Bundling is refused, and the refusal is the command to type instead.
#[test]
fn bundled_short_flags_are_refused_by_naming_the_flags_to_type_separately() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));

    let out = r.ank("claude-code@ank", &["find", "criterion", "-st", "task"]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(said.contains("'-st' bundles"), "{said}");
    // The exact flags, each with its value placeholder: an error that only said
    // "no bundling" would leave the caller to work out what to write.
    assert!(
        said.contains("ank find -s <v> -t <v>"),
        "the hint is the command to run next: {said}"
    );

    // A letter that names nothing on this verb is reported as itself rather
    // than folded into the bundling message, which would advise a command that
    // would refuse too.
    let out = r.ank("claude-code@ank", &["find", "criterion", "-sz"]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(said.contains("unknown flag '-z'"), "{said}");
}

/// A letter that is a real flag on another verb is a different mistake from a
/// letter that is nothing, and the error says which.
#[test]
fn a_short_flag_the_verb_does_not_take_names_the_flag_it_stands_for() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));

    let out = r.ank("claude-code@ank", &["claim", ID, "-s", "open"]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(
        said.contains("'-s' is --status") && said.contains("'claim' does not take"),
        "{said}"
    );
    assert!(
        r.task_text(ID).contains("status: open"),
        "a refused parse claims nothing"
    );

    let out = r.ank("claude-code@ank", &["claim", ID, "-z"]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(said.contains("unknown flag '-z'"), "{said}");
}

/// `ank help <verb>` shows both forms; `ank help` shows what it always did.
///
/// The second half is the one worth a test. ADR-c656cbcc33a9 froze the flat
/// listing, and a second spelling of every flag is exactly the kind of addition
/// that arrives looking like an improvement.
#[test]
fn help_shows_both_forms_for_one_verb_and_leaves_the_flat_listing_alone() {
    let r = Repo::new();

    let one = stdout(&r.ank("claude-code@ank", &["help", "find"]));
    assert!(one.contains("-s, --status <v>"), "{one}");
    assert!(one.contains("-t, --type <v>"), "{one}");
    assert!(one.contains("-j, --json"), "the globals too: {one}");
    // `--scope` has no letter: `s` went to `--status`, and §4 says so rather
    // than leaving a reader to wonder whether it was forgotten.
    assert!(
        one.contains("--scope <v>") && !one.contains(", --scope"),
        "a flag with no short form shows none: {one}"
    );

    let all = stdout(&r.ank("claude-code@ank", &["help"]));
    assert!(
        all.contains("--status"),
        "the listing still names flags: {all}"
    );
    assert!(
        !all.contains("-s, ") && !all.contains("-j, "),
        "the flat listing is unchanged: {all}"
    );

    // A script reads the mapping from --json, or it cannot use it at all.
    let j = stdout(&r.ank("claude-code@ank", &["help", "find", "--json"]));
    assert!(j.contains("\"name\":\"--status\",\"short\":\"-s\""), "{j}");
    assert!(j.contains("\"name\":\"--scope\",\"short\":null"), "{j}");
}

/// A single dash is a flag now, so the escape had to become reachable.
///
/// `ank log "-1 rebuilt"` used to be a message and is now an argument starting
/// with a dash. It is refused for what it is rather than reported as a bundle
/// of unknown letters, and `--` still carries it through.
#[test]
fn a_positional_that_starts_with_a_dash_is_refused_by_naming_the_escape() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);

    let message = "-1 rebuilt the index";
    let out = r.ank("claude-code@ank", &["log", message]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(
        said.contains("is not a flag: it contains a space"),
        "{said}"
    );
    assert!(
        said.contains(&format!("ank log -- \"{message}\"")),
        "the hint is the exact command that works: {said}"
    );

    // And it does work. Invoked directly rather than through the harness,
    // because `--` terminates everything after it and the harness appends
    // `--repo` last -- which is the terminator behaving exactly as specified,
    // and worth seeing once from the caller's side.
    let out = Command::new(ANK)
        .args(["log", "--repo"])
        .arg(&r.0)
        .args(["--", message])
        .env("ANK_AGENT", "claude-code@ank")
        .current_dir(std::env::temp_dir())
        .output()
        .expect("the binary must have been built");
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(r.task_text(ID).contains(message), "{}", r.task_text(ID));

    // A value keeps taking its dash verbatim, whichever form the flag took:
    // only a positional ever needs the escape.
    let out = r.ank("claude-code@ank", &["find", "criterion", "-s=open"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
}

/// Verbs are never abbreviated, and the short forms did not open a door to it.
#[test]
fn a_verb_is_never_abbreviated() {
    let r = Repo::new();
    let out = r.ank("claude-code@ank", &["cl", ID]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(said.contains("unknown command 'cl'"), "{said}");
}

/// A second claim under one identity is named, never refused (TASK-d79dc424c63d).
///
/// Observed while dogfooding: a task claimed in one terminal follows you into a
/// second one, because `ANK_AGENT` unset makes both `<user>@<hostname>`. The
/// identity is not bound to the session on purpose — a PID or a TTY in it would
/// break resuming a claim after a restart — so what the fix owes the user is the
/// warning and the way out.
///
/// Through the binary, because a warning that never reaches stdout is not a
/// warning, and because the environment variable under test is read by the
/// process and not by the function.
#[test]
fn a_second_claim_under_one_identity_warns_and_names_what_is_already_held() {
    let first = "TASK-000000000d01";
    let second = "TASK-000000000d02";
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task(first, Some("A criterion."));
    r.seed_task(second, Some("A criterion."));

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", first])), 0);

    let out = r.ank("claude-code@ank", &["claim", second]);
    let said = stdout(&out);
    assert_eq!(
        code(&out),
        0,
        "a convention warns, it does not refuse: {said}"
    );
    assert!(
        said.contains(&format!("already holds {first}")),
        "the claim already held is named: {said}"
    );
    assert!(said.contains("ANK_AGENT"), "and the way out of it: {said}");
    assert!(
        said.contains(&format!("claimed {second}")),
        "the claim itself still went through: {said}"
    );

    // The other identity is the supported case and must stay silent: parallel
    // agents, one ref per task, is the design and not the anomaly.
    let third = "TASK-000000000d03";
    r.seed_task(third, Some("A criterion."));
    let out = r.ank("codex@ank", &["claim", third]);
    let said = stdout(&out);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        !said.contains("warning:"),
        "a distinct identity holding its own task is not an anomaly: {said}"
    );

    // The lapsed case is a module test: the drift tolerance is two minutes, so
    // waiting for an expiry here would cost two minutes of wall time.
}

/// The warning sends the reader to `getting-started`, so `getting-started` has
/// to be where the answer is (TASK-d79dc424c63d).
///
/// Driven by the binary rather than by a hand-copied string: the point is not
/// that the guide mentions a variable, it is that the exact line the binary
/// prints has somewhere to land. A warning naming a fix nobody wrote down is
/// the defect this task is about, one level up.
#[test]
fn the_guide_documents_the_identity_the_warning_tells_you_to_set() {
    let first = "TASK-000000000d11";
    let second = "TASK-000000000d12";
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task(first, Some("A criterion."));
    r.seed_task(second, Some("A criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", first])), 0);

    let said = stdout(&r.ank("claude-code@ank", &["claim", second]));
    let warned: Vec<&str> = said.lines().filter(|l| l.starts_with("warning:")).collect();
    assert!(!warned.is_empty(), "nothing to document: {said}");

    let guide = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/getting-started.md"),
    )
    .expect("the guide is in the repository the tests run from");

    // The variable the warning names, and the shape of an invocation that sets
    // it: naming it without showing how to use it is half an answer.
    let named: Vec<&str> = warned
        .iter()
        .flat_map(|l| l.split_whitespace())
        .filter(|w| w.contains("ANK_AGENT"))
        .collect();
    assert!(!named.is_empty(), "the warning names no way out: {said}");
    for w in named {
        assert!(
            guide.contains(w),
            "the binary tells the reader about {w} and the guide never mentions it"
        );
    }
    assert!(
        guide.contains("ANK_AGENT=") && guide.contains("ank claim"),
        "the guide names the variable without showing an invocation that sets it"
    );
    assert!(
        guide.contains("concurrent session"),
        "the guide never says the fix is per session, which is the whole point"
    );
}

/// A blocker finished on another branch says which one (TASK-b5ad06f134f6).
///
/// Through the binary, because the defect is precisely a working tree being
/// asked a question only the refs can answer: `check_blockers` built its status
/// map from the files this branch carries, and on `main` the blocker's file
/// still reads `open`. Nothing below is observable from the function — it needs
/// a real repository holding a real completion ref for a commit `main` does not
/// have.
#[test]
fn a_blocker_finished_on_another_branch_is_named_with_its_branch_and_commit() {
    let blocker = "TASK-000000000c01";
    let dependent = "TASK-000000000c02";
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task_with(blocker, Some("A criterion."), &["ok"]);
    r.seed_task(dependent, Some("A criterion."));
    r.blocked(dependent, &[blocker]);
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed tasks"]);

    // A commit of its own on the branch, so the commit the completion record
    // names is one `main` genuinely does not carry. Branching alone would leave
    // HEAD on a commit both branches share, and the assertion below would pass
    // on a repository where nothing was unmerged at all.
    r.git(&["checkout", "-q", "-b", "feature"]);
    std::fs::write(r.0.join("work.txt"), "y").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "work"]);
    let finished_at = r.head();

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", blocker])), 0);
    let out = r.ank("claude-code@ank", &["done"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "done"]);

    // Back where the work has not landed: the blocker's file says `open` again,
    // and the ref is the only thing that remembers otherwise.
    r.git(&["checkout", "-q", "main"]);
    assert!(
        r.task_text(blocker).contains("status: open"),
        "the fixture is wrong if main already carries the done"
    );

    let out = r.ank("claude-code@ank", &["claim", dependent]);
    let said = stderr(&out);
    assert_eq!(code(&out), 7, "the refusal itself does not move: {said}");
    assert!(
        said.contains(&format!("blocked by {blocker}")),
        "the blocker is still named: {said}"
    );
    assert!(
        said.contains("finished on feature"),
        "the branch is the half an agent cannot guess: {said}"
    );
    assert!(
        said.contains(&finished_at[..7]),
        "the commit is what makes the branch checkable: {said}"
    );
    assert!(
        said.contains("not merged here yet"),
        "and why it still blocks: {said}"
    );
    // The hint is never a command that refuses: the blocker carries the
    // completion ref, so claiming it is exactly what fails.
    assert!(
        !said.contains(&format!("ank claim {blocker}")),
        "offered a claim that would refuse on the spot: {said}"
    );
    assert!(
        r.task_text(dependent).contains("status: open"),
        "a refused claim moves nothing"
    );
}

#[test]
fn a_failing_done_through_the_binary_leaves_the_claim_intact() {
    let r = Repo::new().with_verifiers("verifiers:\n  nope:\n    run: exit 2\n");
    r.seed_task_with(ID, Some("A criterion."), &["nope"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);

    let out = r.ank("claude-code@ank", &["done"]);
    assert_eq!(
        code(&out),
        5,
        "a verifier that ran and refused: {}",
        stderr(&out)
    );

    let record = r.claim_ref(ID).expect("the ref is still there");
    assert!(record.contains("state: claim"), "still a claim: {record}");
    assert!(record.contains("claude-code@ank"), "{record}");
    assert!(
        r.task_text(ID).contains("status: in_progress"),
        "the task did not move"
    );
}

#[test]
fn done_refuses_proof_when_the_task_declares_verifiers() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task_with(ID, Some("A criterion."), &["ok"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);

    let out = r.ank(
        "claude-code@ank",
        &["done", "--proof", "assertion:trust me"],
    );
    assert_eq!(code(&out), 5, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("--proof is refused"),
        "{}",
        stderr(&out)
    );
    assert!(r.task_text(ID).contains("status: in_progress"));
}

#[test]
fn the_whole_agent_loop_runs_through_the_binary() {
    // context -> new -> claim -> log -> release -> claim -> done, as an agent
    // actually types it. Each verb was tested in its module; what only exists
    // once the process does is that they compose.
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");

    let out = r.ank(
        "claude-code@ank",
        &["new", "task", "--title", "Rotate secrets"],
    );
    assert_eq!(code(&out), 7, "a scope is required: {}", stderr(&out));

    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "Rotate secrets",
            "--scope",
            "src/**",
            "--criteria",
            "The rotation runs.",
        ],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let created = String::from_utf8_lossy(&out.stdout).to_string();
    let id = created
        .split_whitespace()
        .nth(1)
        .expect("created <id> <title>")
        .to_string();

    assert!(
        String::from_utf8_lossy(&r.ank("claude-code@ank", &["find", "rotate"]).stdout)
            .contains("Rotate secrets")
    );

    // log without a claim is refused, with it goes through and renews.
    assert_eq!(code(&r.ank("claude-code@ank", &["log", "early"])), 6);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", &id])), 0);
    assert_eq!(
        code(&r.ank("claude-code@ank", &["log", "made progress"])),
        0
    );

    // release refuses without a reason, and deletes the ref with one.
    let out = r.ank("claude-code@ank", &["release"]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(stderr(&out).contains("--reason"), "{}", stderr(&out));
    assert!(r.claim_ref(&id).is_some(), "a refusal deletes nothing");

    let out = r.ank(
        "claude-code@ank",
        &["release", "--reason", "needs staging access"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        r.claim_ref(&id).is_none(),
        "release deletes the ref, read back with git"
    );
    let file = r.task_text(&id);
    assert!(file.contains("status: open"), "{file}");
    assert!(
        file.contains("needs staging access"),
        "the reason is in the log: {file}"
    );

    // And the next agent picks it up where the previous one stopped.
    assert_eq!(code(&r.ank("codex@host-9", &["claim", &id])), 0);
    let ctx = String::from_utf8_lossy(&r.ank("codex@host-9", &["context"]).stdout).to_string();
    assert!(ctx.contains("made progress"), "the log travels: {ctx}");
    assert!(ctx.contains("needs staging access"), "{ctx}");
}

/// `git log` reads, and now so does `ank log` given nothing but an id (§4).
///
/// Through the binary because the criterion says so, and because what has to be
/// true is a property of the process: a read that took the claim path would
/// still pass a module test run under the holder's identity, and fail for every
/// caller who is not it.
#[test]
fn log_with_an_id_and_no_message_reads_and_asks_for_no_claim() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    // An empty log is an answer, not a blank. Read here by an agent that holds
    // nothing, before any claim exists at all.
    let out = r.ank("marie@laptop", &["log", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("no log entry yet"),
        "{}",
        stdout(&out)
    );

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    assert_eq!(code(&r.ank("claude-code@ank", &["log", "first thing"])), 0);
    assert_eq!(code(&r.ank("claude-code@ank", &["log", "second thing"])), 0);

    // Still `marie@laptop`, who holds no claim on anything: the claim is what
    // writing needs, never reading.
    let before = r.task_text(ID);
    let out = r.ank("marie@laptop", &["log", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    let newest = text
        .find("second thing")
        .unwrap_or_else(|| panic!("the newer entry is missing:\n{text}"));
    let oldest = text
        .find("first thing")
        .unwrap_or_else(|| panic!("the older entry is missing:\n{text}"));
    assert!(newest < oldest, "newest first (§4):\n{text}");
    assert_eq!(
        r.task_text(ID),
        before,
        "a read that writes is not a read: {text}"
    );
    assert!(
        r.claim_ref(ID).is_some(),
        "reading neither takes nor drops the holder's claim"
    );

    // Scriptable like every other verb, and the same order.
    let out = r.ank("marie@laptop", &["log", ID, "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let j = stdout(&out);
    assert!(
        j.starts_with(&format!("{{\"task\":\"{ID}\",\"entries\":[")),
        "{j}"
    );
    assert!(
        j.find("second thing").unwrap() < j.find("first thing").unwrap(),
        "{j}"
    );

    // Only a task has a log, and the refusal names the verb that does answer.
    const ADR: &str = "ADR-0000000000ab";
    r.seed_adr(ADR, "Do not do X.", "src/**");
    let out = r.ank("marie@laptop", &["log", ADR]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(
        stderr(&out).contains(&format!("ank show {ADR}")),
        "{}",
        stderr(&out)
    );
}

/// The disambiguation of §4, exercised on all three of its branches. It is
/// stated rather than inferred precisely so that it can be asserted this way:
/// one question — does the argument resolve — and one answer.
#[test]
fn log_decides_between_reading_and_writing_by_what_resolves() {
    const OTHER: &str = "TASK-000000000002";
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.seed_task(OTHER, Some("Another verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);

    // Shaped like an id, resolving to nothing: a message.
    let out = r.ank("claude-code@ank", &["log", "TASK-000000000009"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        r.task_text(ID).contains("TASK-000000000009"),
        "{}",
        r.task_text(ID)
    );

    // An ambiguous prefix resolves to no single entity, so it is a message too.
    // "It resolved" is the whole test, and a second question — did it nearly
    // resolve — would be one an agent has to guess the answer to.
    let out = r.ank("claude-code@ank", &["log", "TASK-00000000000"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        r.task_text(ID)
            .lines()
            .any(|l| l.ends_with("TASK-00000000000")),
        "{}",
        r.task_text(ID)
    );

    // A message that also resolves: refused, naming both readings, writing
    // neither.
    let before = r.task_text(ID);
    let out = r.ank("claude-code@ank", &["log", ID, OTHER]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let e = stderr(&out);
    assert!(e.contains(OTHER), "the read it could have been: {e}");
    assert!(e.contains(ID), "the write it could have been: {e}");
    assert_eq!(r.task_text(ID), before, "a refusal writes nothing");

    // The redundant write form is untouched when the message is a message.
    let out = r.ank("claude-code@ank", &["log", ID, "a real message"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(r.task_text(ID).contains("a real message"));

    // And an id alone reads even for the agent holding it: what decides is the
    // argument, not who is asking.
    let out = r.ank("claude-code@ank", &["log", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("a real message"), "{}", stdout(&out));
    assert_eq!(
        stdout(&out).matches("a real message").count(),
        1,
        "reading appended an entry: {}",
        stdout(&out)
    );
}

#[test]
fn init_runs_where_there_is_no_ank_directory_yet() {
    // The one verb that precedes the foundation. Kept here because the reason
    // it bypasses `startup` is only observable from outside the process.
    let dir = std::env::temp_dir().join(format!("ank-cli-init-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    assert!(Command::new("git")
        .current_dir(&dir)
        .args(["init", "-q"])
        .status()
        .unwrap()
        .success());

    let out = Command::new(ANK)
        .arg("init")
        .arg(&dir)
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(Path::new(&dir).join(".ank/config.yml").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

/// §6 calls the index derived, disposable and gitignored. The first two were
/// true of what `init` produced; the third was true of no repository it had
/// ever produced, because it wrote no ignore rule at all. This repository was
/// not the counterexample -- it carries the line by hand, written before
/// `init` existed to write it, which is exactly how the gap survived.
///
/// Through the binary end to end, and deliberately not by reading
/// `.gitignore`: what has to be true is that *git* treats the file as ignored,
/// and a line present in a file it does not consult would satisfy any
/// assertion about the file's content while proving nothing.
#[test]
fn an_initialised_repo_leaves_the_index_ignored_and_never_untracked() {
    let dir = std::env::temp_dir().join(format!("ank-cli-ignore-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let git = |args: &[&str]| -> Output {
        Command::new("git")
            .current_dir(&dir)
            .args(args)
            .output()
            .expect("git must be installed: it is a hard dependency")
    };
    assert!(git(&["init", "-q", "-b", "main"]).status.success());

    // A `.gitignore` the user curated first: `init` has to append to it, not
    // replace it, and that is only observable when there is something to lose.
    std::fs::write(dir.join(".gitignore"), "/target\n").unwrap();

    let init = || -> Output {
        Command::new(ANK)
            .arg("init")
            .arg(&dir)
            .env("ANK_AGENT", "claude-code@ank")
            .current_dir(std::env::temp_dir())
            .output()
            .expect("the binary must have been built")
    };
    let out = init();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("wrote .gitignore"),
        "{}",
        stdout(&out)
    );

    // A command that builds the index, so the file under test actually exists.
    let out = Command::new(ANK)
        .args(["find", "--status", "open"])
        .arg("--repo")
        .arg(&dir)
        .env("ANK_AGENT", "claude-code@ank")
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        dir.join(".ank/index.db").exists(),
        "nothing built the index, so the assertion below would pass vacuously"
    );

    // Ignored, positively: `check-ignore` names a rule that matches.
    let ci = git(&["check-ignore", "-v", ".ank/index.db"]);
    assert!(
        ci.status.success(),
        "git does not consider the index ignored: {}{}",
        String::from_utf8_lossy(&ci.stdout),
        String::from_utf8_lossy(&ci.stderr)
    );

    // And untracked, negatively: `status` must not offer it. `-uall` because
    // the default collapses an untracked directory to its name, which would
    // hide the very path in question behind `.ank/`.
    let st = git(&["status", "--porcelain", "-uall"]);
    let st = String::from_utf8_lossy(&st.stdout).replace('\\', "/");
    assert!(
        !st.contains("index.db"),
        "the index is still offered for commit:\n{st}"
    );
    // The fixture is sound only if `status` sees the rest of `.ank/`.
    assert!(
        st.contains(".ank/config.yml"),
        "empty status proves nothing:\n{st}"
    );

    // The user's own rule survived, and a second init adds nothing.
    let gi = std::fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert!(gi.starts_with("/target\n"), "{gi:?}");

    let out = init();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        !stdout(&out).contains("wrote .gitignore"),
        "re-init rewrote it: {}",
        stdout(&out)
    );
    let again = std::fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert_eq!(gi, again, "re-init changed .gitignore");

    let _ = std::fs::remove_dir_all(&dir);
}

/// The bug this exists for, end to end and through the binary: a corpus that a
/// Windows clone handed back in CRLF was entirely unreadable, and every entity
/// was reported as "missing frontmatter" -- a diagnostic that sends the reader
/// looking for a delimiter sitting right there on line one.
///
/// The criterion names `ank check` and an exit code, so it is the process that
/// answers here, not the function.
#[test]
fn a_crlf_corpus_is_read_signalled_and_exits_zero_through_the_binary() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    let p = r.0.join(".ank/tasks").join(format!("{ID}.md"));
    let lf = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, lf.replace('\n', "\r\n")).unwrap();

    let out = r.ank("claude-code@ank", &["check"]);
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));

    assert_eq!(code(&out), 0, "CRLF alone must not fail the build: {text}");
    assert!(text.contains("CRLF"), "{text}");
    assert!(text.contains("git config core.autocrlf input"), "{text}");
    assert!(text.contains("signal"), "{text}");
    assert!(
        !text.contains("missing frontmatter"),
        "the wrong diagnostic came back: {text}"
    );
    assert!(!text.contains("non-canonical"), "{text}");

    // And the entity was genuinely read, not merely tolerated: `show` prints
    // the task it could not previously parse at all.
    let out = r.ank("claude-code@ank", &["show", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(String::from_utf8_lossy(&out.stdout).contains("Example task"));
}

/// The one write §3 permits after `done`, performed by a command.
///
/// Through the binary and reading the file back, because the append is the
/// thing being asserted: what has to be true is what ends up on disk, not the
/// module's agreement with itself. Before this verb the same operation was done
/// by opening the file, where an append and a substitution are the same gesture.
#[test]
fn attest_appends_a_proof_to_a_finished_task_and_never_substitutes() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task_with(ID, Some("A verifiable criterion."), &["ok"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    assert_eq!(code(&r.ank("claude-code@ank", &["done"])), 0);

    let before = r.task_text(ID);
    assert!(before.contains("status: done"), "{before}");
    let original = before
        .lines()
        .find(|l| l.contains("ref: local/"))
        .expect("done wrote a proof")
        .to_string();

    let head = r.head();
    let out = r.ank(
        "claude-code@ank",
        &["attest", ID, "--proof", &format!("commit:{head}")],
    );
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
    assert_eq!(code(&out), 0, "{text}");

    let after = r.task_text(ID);
    // Appended, never substituted: the entry `done` wrote is still there, and
    // the new one sits beside it.
    assert!(
        after.contains(&original),
        "the original proof was rewritten:\n{after}"
    );
    assert!(after.contains("type: commit"), "{after}");
    assert!(after.contains(&head), "{after}");
    // Version increments and the log records what was added.
    assert!(!after.contains("version: 1"), "{after}");
    assert!(after.contains("attested commit:"), "{after}");

    // A commit that does not exist is refused, exactly as `done` refuses it.
    let out = r.ank(
        "claude-code@ank",
        &[
            "attest",
            ID,
            "--proof",
            "commit:0000000000000000000000000000000000000000",
        ],
    );
    assert_eq!(code(&out), 5, "{}", stderr(&out));
    assert_eq!(r.task_text(ID), after, "nothing was written");
}

#[test]
fn attest_refuses_a_task_that_is_not_finished_and_names_the_verb_that_applies() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    // A prerequisite rather than a transition: `attest` changes no status, it
    // adds to a record that only exists once the task is closed out.
    let out = r.ank("claude-code@ank", &["attest", ID, "--proof", "assertion:x"]);
    let err = stderr(&out);
    assert_eq!(code(&out), 7, "{err}");
    assert!(err.contains("open"), "the message names the state: {err}");
    assert!(
        err.contains(&format!("ank done {ID}")),
        "the refusal names the verb that applies: {err}"
    );
    assert!(
        !r.task_text(ID).contains("proof:"),
        "a refusal writes nothing"
    );
}

/// Writes a `done` task carrying exactly `proofs`, which the seeding helpers
/// cannot express: they build open tasks, and `done` is the one status whose
/// proof list is the subject here.
fn seed_done(r: &Repo, id: &str, proofs: &str) {
    std::fs::write(
        r.0.join(".ank/tasks").join(format!("{id}.md")),
        format!(
            "---\nid: {id}\ntype: task\nslug: example\ntitle: Example task\n\
             created: 2026-07-28T00:00:00Z\nstatus: done\nscope:\n  - src/**\n\
             blocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
             criteria_by: creator\nproof:\n{proofs}schema: 1\nversion: 1\n---\n\nFree body.\n"
        ),
    )
    .unwrap();
}

const ATTESTED: &str = "TASK-00000000a77e";
const UNATTESTED: &str = "TASK-00000000c0dd";

/// `done` records what ran on the machine that ran it; `attest` anchors the
/// same criterion to a run anybody can re-read. Nothing used to notice a task
/// that never got the second one: it reads `done`, its proof list is not empty,
/// and `commit` is not weak, so every existing finding stayed silent.
///
/// Through the binary, and the negative half carries the weight: a signal that
/// fired on the attested task too would be reporting every finished task in the
/// corpus, which is a line readers learn to skip rather than a finding.
#[test]
fn check_reports_a_done_task_that_was_never_attested_and_spares_one_that_was() {
    let r = Repo::new();
    seed_done(&r, UNATTESTED, "  - type: commit\n    ref: abc1234\n");
    seed_done(
        &r,
        ATTESTED,
        "  - type: commit\n    ref: abc1234\n  - type: test\n    ref: \"991\"\n",
    );
    // The seeded scope has to match something tracked, or a dead-scope fault
    // fires and the exit code under test stops being about attestation.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/lib.rs"), "// x\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));

    // A signal and not a fault: the corpus is intact, the record is thin.
    // Exit 8 here would redden CI on the merge that introduced the task.
    assert_eq!(
        code(&out),
        0,
        "attestation is a signal, not a fault: {said}"
    );

    let reported: Vec<&str> = said
        .lines()
        .filter(|l| l.contains("no test proof"))
        .collect();
    assert_eq!(
        reported.len(),
        1,
        "exactly the unattested task should be named:\n{said}"
    );
    assert!(reported[0].contains(UNATTESTED), "{said}");
    assert!(
        reported[0].contains(&format!("ank attest {UNATTESTED}")),
        "a finding names the exact command that clears it: {said}"
    );
}

/// The gate, which is load-bearing and not decoration.
///
/// On a feature branch straight after `done`, attesting is impossible: no merge
/// run exists to cite yet. Reporting there would name work the reader cannot
/// do, and the completion ref already covers that window with a signal that
/// says the useful thing instead.
#[test]
fn an_unattested_task_is_not_reported_until_the_default_branch_carries_it() {
    let r = Repo::new();
    // A commit on `main` that does not carry the task, so the question asked of
    // the default branch is answerable and the answer is no. Without this, the
    // branch would carry no commit at all and the silence below would prove
    // only that git could not be asked.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/lib.rs"), "// x\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);

    r.git(&["checkout", "-q", "-b", "feature"]);
    seed_done(&r, UNATTESTED, "  - type: commit\n    ref: abc1234\n");
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "work"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(code(&out), 0, "{said}");
    assert!(
        !said.contains("no test proof"),
        "reported before the merge, when attesting is impossible:\n{said}"
    );

    // And it appears the moment the default branch catches up, so the silence
    // above is the gate and not the signal being broken outright.
    r.git(&["checkout", "-q", "main"]);
    r.git(&["merge", "-q", "--no-ff", "-m", "merge", "feature"]);
    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert!(
        said.contains("no test proof") && said.contains(UNATTESTED),
        "the default branch caught up and nothing was reported:\n{said}"
    );
}

/// A task created through the binary needs no hand finishing.
///
/// The assertion is about what lands on disk, so the file is read back rather
/// than the return value inspected: every task this repository created through
/// `ank new` had to be reopened afterwards to gain a `verify:` and a body, and
/// that reopening is the practice the task exists to end.
#[test]
fn a_task_created_by_new_declares_its_verifiers_and_carries_its_reasoning() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");

    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "A created task",
            "--scope",
            "src/**",
            "--criteria",
            "A verifiable criterion.",
            "--verify",
            "ok",
            "--body",
            "Why this exists.\n\nAnd the trap worth naming.",
        ],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let id = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .nth(1)
        .expect("created <id> <title>")
        .to_string();

    let text = r.task_text(&id);
    assert!(text.contains("verify: [ok]"), "{text}");
    assert!(text.contains("Why this exists."), "{text}");
    assert!(text.contains("And the trap worth naming."), "{text}");
    // Canonical shape: a blank line after the frontmatter, and a final newline.
    // A creation that emitted anything else would be reformatted by the first
    // rewrite, and the round-trip is byte-identical on canonical form.
    assert!(text.contains("---\n\nWhy this exists."), "{text:?}");
    assert!(text.ends_with("worth naming.\n"), "{text:?}");

    // Complete means claimable and then finishable without touching the file:
    // `done` runs the declared verifier instead of demanding a proof the agent
    // supplies itself.
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", &id])), 0);
    let out = r.ank("claude-code@ank", &["done"]);
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
    assert_eq!(code(&out), 0, "{text}");
    assert!(text.contains("running: ok"), "{text}");
    assert!(
        r.task_text(&id).contains("verifier: ok@"),
        "{}",
        r.task_text(&id)
    );
}

/// `supersedes` existed in the model, `check` enforced the chain both ways and
/// `accept` completed it, while `commands.rs` wrote `supersedes: None`
/// unconditionally — everything built around a value nothing could write.
///
/// Through the binary and reading the file, because what is asserted is what
/// lands on disk: a resolution that never reached the serializer would pass a
/// unit test on the parsed entity and leave the field absent.
#[test]
fn new_adr_writes_the_succession_it_declares() {
    let r = Repo::new();
    let an_adr = |title: &str, extra: &[&str]| {
        let mut argv = vec![
            "new",
            "adr",
            "--title",
            title,
            "--scope",
            "src/**",
            "--constraint",
            "A binding rule.",
        ];
        argv.extend_from_slice(extra);
        let out = r.ank("claude-code@ank", &argv);
        (
            code(&out),
            String::from_utf8_lossy(&out.stdout).to_string(),
            stderr(&out),
        )
    };
    let id_of = |stdout: &str| {
        stdout
            .split_whitespace()
            .nth(1)
            .expect("created <id> <title>")
            .to_string()
    };

    let (c, stdout, err) = an_adr("The replaced", &[]);
    assert_eq!(c, 0, "{err}");
    let replaced = id_of(&stdout);

    // A short prefix resolves, exactly as `--blocked-by` does.
    let (c, stdout, err) = an_adr("The replacement", &["--supersedes", &replaced[..9]]);
    assert_eq!(c, 0, "{err}");
    let replacement = id_of(&stdout);

    let text = r.adr_text(&replacement);
    assert!(
        text.contains(&format!("supersedes: {replaced}")),
        "the prefix is resolved to the full id on disk:\n{text}"
    );
    // Proposed, and never born accepted: the succession happens at `accept`.
    assert!(text.contains("status: proposed"), "{text}");

    // A reference matching nothing is refused here rather than surfacing in
    // `check` as a corpus fault nobody can attribute to the act.
    let (c, _, err) = an_adr("Dangling", &["--supersedes", "ADR-ffffffffffff"]);
    assert_ne!(c, 0, "an unknown reference must not reach the file");
    assert!(err.contains("ffffffffffff"), "{err}");

    // And the flag is refused on a task, never dropped.
    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "T",
            "--scope",
            "src/**",
            "--supersedes",
            &replaced,
        ],
    );
    assert_ne!(code(&out), 0, "a dropped flag teaches the caller it worked");
    assert!(stderr(&out).contains("--supersedes"), "{}", stderr(&out));
}

/// The two fields a plan actually changes on, and the two edits that were done
/// by hand this session because no command reached them.
///
/// Through the binary and reading the file, because what is asserted is that
/// the amendment lands on disk and that everything it did not name comes back
/// untouched — which is the whole difference between a verb and an editor.
#[test]
fn amend_adds_and_removes_without_disturbing_the_rest() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.seed_task("TASK-000000000002", Some("Another criterion."));
    r.seed_task("TASK-000000000003", Some("A third criterion."));
    let before = r.task_text(ID);

    // Nothing named is a refusal, not a silent no-op that bumps `version`.
    let out = r.ank("marie@laptop", &["amend", ID]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert_eq!(r.task_text(ID), before, "a refusal writes nothing");

    // A blocker added to a task that already exists, resolved by prefix.
    let out = r.ank(
        "marie@laptop",
        &[
            "amend",
            "0000000000 01",
            "--blocked-by",
            "TASK-000000000002",
            "--scope",
            "docs/**",
        ],
    );
    // The odd id above is deliberate: it must not resolve.
    assert_ne!(code(&out), 0, "a bad prefix is refused");

    let out = r.ank(
        "marie@laptop",
        &[
            "amend",
            ID,
            "--blocked-by",
            "TASK-000000000002",
            "--blocked-by",
            "TASK-000000000003",
            "--scope",
            "docs/**",
        ],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let text = r.task_text(ID);
    assert!(
        text.contains("blocked_by: [TASK-000000000002, TASK-000000000003]"),
        "{text}"
    );
    assert!(text.contains("  - src/**\n  - docs/**"), "{text}");
    assert!(text.contains("version: 2"), "the write increments: {text}");
    assert!(
        text.contains("amended: +blocked_by TASK-000000000002"),
        "the log says what changed: {text}"
    );
    assert!(text.contains("+scope docs/**"), "{text}");
    // Everything it did not name is still there, byte for byte.
    assert!(
        text.contains("done_criteria: |\n  A verifiable criterion."),
        "{text}"
    );

    // Removal is explicit, and removing what is not there is refused rather
    // than succeeding quietly.
    let out = r.ank(
        "marie@laptop",
        &["amend", ID, "--drop-blocked-by", "TASK-000000000003"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = r.task_text(ID);
    assert!(text.contains("blocked_by: [TASK-000000000002]"), "{text}");
    assert!(text.contains("-blocked_by TASK-000000000003"), "{text}");

    let out = r.ank("marie@laptop", &["amend", ID, "--drop-scope", "nowhere/**"]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("not in the scope"),
        "{}",
        stderr(&out)
    );

    // An entity attached to nothing is invisible, so the last glob cannot go.
    let out = r.ank(
        "marie@laptop",
        &[
            "amend",
            ID,
            "--drop-scope",
            "src/**",
            "--drop-scope",
            "docs/**",
        ],
    );
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(stderr(&out).contains("no scope"), "{}", stderr(&out));

    // The frozen field is refused by name, with the command that applies.
    let out = r.ank(
        "marie@laptop",
        &["amend", ID, "--criteria", "Anything at all."],
    );
    assert_eq!(code(&out), 6, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("frozen"), "{err}");
    assert!(
        err.contains("ank release"),
        "the command that applies: {err}"
    );
    assert!(
        r.task_text(ID).contains("A verifiable criterion."),
        "the criterion did not move"
    );
}

#[test]
fn new_refuses_a_verifier_that_config_does_not_declare() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    let out = r.ank(
        "claude-code@ank",
        &[
            "new", "task", "--title", "T", "--scope", "src/**", "--verify", "nope",
        ],
    );
    let err = stderr(&out);
    // Resolved at creation for the same reason --blocked-by is: a name that
    // matches nothing would otherwise surface at `done`, far from its cause.
    assert_eq!(code(&out), 7, "{err}");
    assert!(err.contains("nope"), "{err}");
    assert!(err.contains("ok"), "the hint names what is declared: {err}");

    // And an ADR has no verify field, so the flag is refused, never dropped.
    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "adr",
            "--title",
            "T",
            "--scope",
            "src/**",
            "--constraint",
            "A binding rule.",
            "--verify",
            "ok",
        ],
    );
    assert_ne!(code(&out), 0, "a dropped flag teaches the caller it worked");
    assert!(stderr(&out).contains("--verify"), "{}", stderr(&out));
}

/// A claim that lapsed and left the file behind.
///
/// Through the binary because the thing being read is the ref namespace, and a
/// unit test would assert the module's agreement with itself. The state is built
/// the way it actually arises — a real claim, then the ref goes away — rather
/// than by writing `status: in_progress` into a file by hand, which would test a
/// corpus nobody produces.
///
/// TASK-daf25ab8a9b7 sat in exactly this state through fifteen signals on this
/// repository's own corpus, and not one of them was about it.
#[test]
fn a_task_in_progress_with_no_claim_ref_is_signalled_and_never_fails_ci() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    assert!(r.claim_ref(ID).is_some(), "the claim must have been taken");
    r.git(&["update-ref", "-d", &format!("refs/ank/claims/{ID}")]);
    assert!(r.claim_ref(ID).is_none(), "the ref must be gone");
    assert!(
        r.task_text(ID).contains("status: in_progress"),
        "the file must still claim to be held"
    );

    let out = r.ank("claude-code@ank", &["check"]);
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
    assert_eq!(
        code(&out),
        0,
        "a stale record is not a corrupt corpus and must never fail CI: {text}"
    );
    assert!(text.contains("signal:"), "{text}");
    assert!(text.contains(ID), "the finding must name the task: {text}");
    assert!(text.contains("no claim ref"), "{text}");
    assert!(
        text.contains(&format!("ank claim {ID}")),
        "the finding must carry the command that picks it up: {text}"
    );

    // And it stops the moment something holds the task again, so the signal
    // tracks the state rather than the history.
    assert_eq!(code(&r.ank("codex@host-9", &["claim", ID])), 0);
    let out = r.ank("claude-code@ank", &["check"]);
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
    assert_eq!(code(&out), 0, "{text}");
    assert!(!text.contains("no claim ref"), "{text}");
}

// ---------------------------------------------------------------------------
// help (§9)
// ---------------------------------------------------------------------------

/// `help` outside any repository, which is the case that matters: the CLI it is
/// bundled with is installed as a skill, and SKILL.md sends the reader to
/// `ank help` before anything establishes where the reader is standing. A help
/// that demands a `.ank/` is a help withheld from the agent most likely to need
/// it.
///
/// `current_dir` is the system temp directory, which is not a git repository
/// and has no `.ank/`, and no `--repo` is passed. Anything that goes through
/// `startup` answers 9 or 1 from there.
fn help_from_nowhere(args: &[&str]) -> Output {
    Command::new(ANK)
        .args(args)
        .current_dir(std::env::temp_dir())
        .output()
        .expect("the binary must have been built")
}

#[test]
fn help_answers_outside_a_repository_and_lists_every_verb() {
    let out = help_from_nowhere(&["help"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(
        code(&out),
        0,
        "help must answer without a repository: {}",
        stderr(&out)
    );

    // Listed by hand here on purpose: cli.rs walks its own table, which would
    // agree with itself even if the table were wrong, and the criterion is
    // about what an agent reads.
    for verb in [
        "context", "claim", "log", "done", "release", "new", "find", "review", "accept", "close",
        "check", "show", "init", "help", "attest",
    ] {
        assert!(
            text.contains(&format!("ank {verb}")),
            "{verb} missing:\n{text}"
        );
    }

    // One flat listing (ADR-c656cbcc33a9). The headings were named after
    // callers -- agent loop, agent off-loop, human -- which is the two-surface
    // model speaking through the output an agent reads, and a CLI that refuses
    // on state and never on identity has no such grouping to print.
    for heading in ["agent loop", "off-loop", "human", "setup"] {
        assert!(
            !text.contains(heading),
            "'{heading}' still groups the listing:\n{text}"
        );
    }

    // Flat is not sorted: §4 puts the loop first, and that order is the only
    // structure left. Alphabetical would bury `context` between `close` and
    // `done`, so the order is asserted through the binary rather than trusted.
    let at = |needle: &str| {
        text.find(needle)
            .unwrap_or_else(|| panic!("{needle} missing:\n{text}"))
    };
    assert!(at("ank context") < at("ank done"), "{text}");
    assert!(at("ank done") < at("ank release"), "{text}");
    assert!(at("ank release") < at("ank review"), "{text}");
    assert!(at("ank review") < at("ank check"), "{text}");
    assert!(
        text.starts_with("ank context"),
        "a title or a heading precedes the first verb:\n{text}"
    );
    // And the flags each verb takes, which is the part §9 keeps out of
    // SKILL.md.
    for flag in [
        "--limit",
        "--criteria",
        "--ttl",
        "--proof",
        "--reason",
        "--scope",
    ] {
        assert!(text.contains(flag), "{flag} missing:\n{text}");
    }
    assert!(text.contains("--json"), "{text}");
}

#[test]
fn help_for_one_verb_answers_and_an_unknown_one_is_a_two() {
    let out = help_from_nowhere(&["help", "claim"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(text.contains("ank claim <id>"), "{text}");
    assert!(text.contains("--ttl"), "{text}");
    assert!(text.contains("--criteria"), "{text}");
    assert!(
        !text.contains("audience"),
        "the audience line is what ADR-c656cbcc33a9 removes, and it is the \
         line an agent reads about itself:\n{text}"
    );
    assert!(
        !text.contains("ank accept"),
        "one verb means one verb:\n{text}"
    );

    // The exit code is the thing being asserted, and 2 is "entity not found"
    // in the table of §4 -- the verb looked for did not exist.
    let out = help_from_nowhere(&["help", "clam"]);
    let err = stderr(&out);
    assert_eq!(
        code(&out),
        2,
        "stdout={} stderr={err}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        err.contains("clam"),
        "the message must name what was looked for: {err}"
    );
    assert!(err.contains("ank help"), "{err}");
    assert!(
        out.stdout.is_empty(),
        "a silent fallback to the general listing: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn help_json_reaches_the_process_intact() {
    let out = help_from_nowhere(&["help", "--json"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(text.starts_with("{\"verbs\":["), "{text}");
    assert!(text.trim_end().ends_with("]}"), "{text}");
    assert!(text.contains("\"name\":\"claim\""), "{text}");
    assert!(
        !text.contains("audience"),
        "the grouping survived into the scripted output:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// Dead constraints in the prose (ADR-c656cbcc33a9)
// ---------------------------------------------------------------------------

/// A comment citing a superseded ADR as the reason for a design is worse than
/// no comment: it hands the next reader a constraint that binds nobody, with
/// the authority of a decision record. Two of them went on asserting a frozen
/// agent surface -- at seven verbs in one, at eight in the other -- in module
/// headers and doc comments, long after the split they protected had been
/// dissolved and while the file around them had already stopped obeying it.
///
/// The needles are assembled from fragments so this file does not defeat its
/// own assertion by containing what it forbids. It reads as an affectation
/// until you picture the test failing on itself.
///
/// This is not a general ban on naming a superseded ADR: history is worth
/// writing down, and `.ank/` is where it is written. It is a ban on these two,
/// which have no live claim left to make anywhere in this crate.
#[test]
fn no_superseded_adr_is_cited_in_the_crate() {
    const DEAD: [&str; 2] = [
        concat!("ADR-", "2f8a61c04b7d"),
        concat!("ADR-", "3859eb46bdc3"),
    ];

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut walked = 0usize;
    for dir in ["src", "tests"] {
        for entry in std::fs::read_dir(root.join(dir)).expect("the crate has this directory") {
            let path = entry.expect("a readable entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a readable source file");
            for dead in DEAD {
                assert!(
                    !text.contains(dead),
                    "{} cites {dead}, which is superseded: name the ADR that \
                     binds today, or drop the citation",
                    path.display()
                );
            }
            walked += 1;
        }
    }
    // A walk that quietly found nothing would pass forever, which is the one
    // way a test like this fails at being a test.
    assert!(walked >= 8, "only {walked} source files walked");

    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("a readable manifest");
    for dead in DEAD {
        assert!(!manifest.contains(dead), "Cargo.toml cites {dead}");
    }
}

// ---------------------------------------------------------------------------
// edit
// ---------------------------------------------------------------------------
//
// Through the binary, and not only because the criterion says so: `edit` spends
// most of its life in a child process and in a file outside the corpus, and
// there is no in-process test that could reach either.

/// The seeded task, written back out of order and with two fields moved. What
/// comes out must be canonical form regardless of how the editor left it (§3).
const EDITED_TASK: &str = "---\ntype: task\nid: TASK-000000000001\nstatus: open\n\
     title: A better title\nslug: example\ncreated: 2026-07-28T00:00:00Z\n\
     scope:\n  - src/**\nblocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
     criteria_by: creator\nschema: 1\nversion: 1\n---\n\nRewritten body.\n";

#[test]
fn edit_writes_back_what_the_editor_saved_in_canonical_form() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let editor = r.editor_saving(EDITED_TASK);

    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some(&editor));
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);
    assert!(said.starts_with(&format!("edited {ID}")), "{said}");
    // Named field by field, so that a reader can tell a title fix from a scope
    // change without reaching for the diff.
    assert!(said.contains("title") && said.contains("body"), "{said}");
    assert!(said.contains("version 2"), "{said}");

    let on_disk = r.task_text(ID);
    // Canonical order, not the editor's: `id`, then `type`, then `slug`,
    // whatever the file that came back said.
    assert!(
        on_disk.starts_with(
            "---\nid: TASK-000000000001\ntype: task\nslug: example\ntitle: A better title\n"
        ),
        "{on_disk}"
    );
    assert!(on_disk.contains("\nversion: 2\n"), "{on_disk}");
    assert!(on_disk.ends_with("---\n\nRewritten body.\n"), "{on_disk}");
}

/// Code 9 and nothing guessed (§4). An editor chosen on the caller's behalf
/// would open something they never asked for, on a file they are about to
/// commit.
#[test]
fn edit_without_an_editor_is_an_environment_failure_and_touches_nothing() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let before = r.task_text(ID);

    let out = r.ank_edit("claude-code@ank", &["edit", ID], None);
    assert_eq!(code(&out), 9, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("EDITOR is not set"), "{err}");
    assert!(
        err.contains(&format!("EDITOR=vi ank edit {ID}")),
        "the exact command to run next, never generic help: {err}"
    );
    assert_eq!(r.task_text(ID), before, "the entity is untouched");
}

/// The clause the temporary copy exists for: nothing reaches `.ank/` until the
/// text has parsed, so a mistyped frontmatter costs a re-edit and never a
/// corrupt file. And the text survives the refusal, which is the difference
/// between a validation and a punishment.
#[test]
fn an_invalid_result_leaves_the_entity_untouched_and_says_why() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let before = r.task_text(ID);
    let broken = "---\nid: TASK-000000000001\ntype: task\ntitle: Half a file\n";
    let editor = r.editor_saving(broken);

    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some(&editor));
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("does not parse"), "{err}");
    assert!(err.contains("missing frontmatter"), "says why: {err}");
    assert_eq!(r.task_text(ID), before, "byte for byte");

    // The named file is real and holds what the editor saved. A message that
    // pointed at nothing would be worse than no message.
    let kept = err
        .split("kept at ")
        .nth(1)
        .and_then(|s| s.split(')').next())
        .expect("the refusal names where the text was kept");
    assert_eq!(
        std::fs::read_to_string(kept.trim()).expect("the kept file exists"),
        broken
    );
    let _ = std::fs::remove_file(kept.trim());
}

/// `done_criteria` is frozen at claim (§3), and the refusal names the command
/// that legally performs the change rather than the flag the caller reached for.
#[test]
fn a_claimed_criterion_is_refused_and_names_release() {
    let r = Repo::new().with_verifiers("");
    r.seed_task(ID, Some("A verifiable criterion."));
    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let before = r.task_text(ID);

    let weakened = EDITED_TASK.replace("A verifiable criterion.", "Something easier.");
    let editor = r.editor_saving(&weakened);
    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some(&editor));
    assert_eq!(code(&out), 6, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("frozen by the claim"), "{err}");
    assert!(err.contains("ank release --reason"), "{err}");
    assert_eq!(r.task_text(ID), before, "the entity is untouched");

    // The guard is on the field, not on the claim: the same task, the same
    // claim, an edit that leaves the criterion alone. Refusing this would make
    // `edit` useless exactly where the work happens.
    let editor = r.editor_saving(EDITED_TASK);
    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some(&editor));
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        r.task_text(ID).contains("A better title"),
        "{}",
        r.task_text(ID)
    );
}

/// `constraint` and `scope` are hashed into the signed ratification commit
/// (§8), so changing a ratified decision is a succession — and succession has
/// its own verb.
#[test]
fn a_ratified_constraint_is_refused_and_names_the_succession() {
    const ADR: &str = "ADR-0000000000ab";
    let r = Repo::new();
    r.enable_signing();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);
    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let before = r.adr_text(ADR);

    let altered = before.replace("Do not do X.", "Do not do Y.");
    let editor = r.editor_saving(&altered);
    let out = r.ank_edit("marie@laptop", &["edit", ADR], Some(&editor));
    assert_eq!(code(&out), 6, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("ratified"), "{err}");
    assert!(err.contains("ank new adr --supersedes"), "{err}");
    assert_eq!(r.adr_text(ADR), before, "the entity is untouched");

    // The body of an accepted ADR stays editable (§3): what is anchored is the
    // constraint and the scope, and the refusal must not reach past them.
    let reasoned = before.replace("Why.", "Why, at length.");
    let editor = r.editor_saving(&reasoned);
    let out = r.ank_edit("marie@laptop", &["edit", ADR], Some(&editor));
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("body"), "{}", stdout(&out));
}

/// An editor opened and closed without saving writes nothing at all — no
/// version bump, no rewrite. The alternative is a verb that dirties a file for
/// having been looked at.
#[test]
fn an_editor_that_saves_nothing_writes_nothing() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let before = r.task_text(ID);

    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some("true"));
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).starts_with(&format!("unchanged {ID}")),
        "{}",
        stdout(&out)
    );
    assert_eq!(r.task_text(ID), before);
}

/// An editor that fails is the environment failing, not the corpus refusing —
/// the same reading `verify` applies to a shell that cannot run what it is
/// given, and the same code.
#[test]
fn an_editor_that_exits_non_zero_leaves_the_entity_untouched() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let before = r.task_text(ID);

    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some("false"));
    assert_eq!(code(&out), 9, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("the editor exited 1"),
        "{}",
        stderr(&out)
    );
    assert_eq!(r.task_text(ID), before);
}

// ---------------------------------------------------------------------------
// new, interactive
// ---------------------------------------------------------------------------
//
// The `git commit` pattern: no flags, an editor (§4). The flag form is what
// SKILL.md teaches and what an agent uses, and the tests that matter most here
// are the ones asserting nothing about it moved.

/// The entities a repository holds, read off the disk rather than through the
/// tool: what has to be true is the state of the corpus, not `find`'s agreement
/// with itself.
fn entity_files(r: &Repo, sub: &str) -> Vec<String> {
    let dir = r.0.join(".ank").join(sub);
    let mut v: Vec<String> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|n| n.ends_with(".md"))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[test]
fn new_task_without_flags_opens_a_template_and_writes_what_comes_back() {
    let r = Repo::new();
    let editor = editor_filling("A task typed in an editor", "crates/**", "The reasoning.");

    let out = r.ank_edit("claude-code@ank", &["new", "task"], Some(&editor));
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("created TASK-")
            && stdout(&out).contains("A task typed in an editor"),
        "{}",
        stdout(&out)
    );

    let files = entity_files(&r, "tasks");
    assert_eq!(files.len(), 1, "exactly one task: {files:?}");
    let text = std::fs::read_to_string(r.0.join(".ank/tasks").join(&files[0])).unwrap();

    // The guidance rides in YAML comments and the parser drops them. A template
    // whose help text reached the corpus would put it in every `show` forever.
    assert!(!text.contains('#'), "the comments are stripped: {text}");
    assert!(text.contains("title: A task typed in an editor"), "{text}");
    assert!(
        text.contains("  - crates/**"),
        "canonical block scope: {text}"
    );
    // Derived, not typed: the template never asks for a slug.
    assert!(text.contains("slug: a-task-typed-in-an-editor"), "{text}");
    assert!(
        text.ends_with("The reasoning.\n"),
        "the body survives: {text}"
    );
    assert!(
        text.contains("status: open") && text.contains("version: 1"),
        "{text}"
    );
}

#[test]
fn new_adr_without_flags_opens_a_template_and_writes_what_comes_back() {
    let r = Repo::new();
    // The constraint is the ADR's mandatory field, and the template leaves it an
    // empty block for the caller to fill.
    let editor = editor_filling_and(
        "A decision typed in an editor",
        "docs/**",
        r"s|^  $|  Do not do X.|",
    );

    let out = r.ank_edit("marie@laptop", &["new", "adr"], Some(&editor));
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let files = entity_files(&r, "adr");
    assert_eq!(files.len(), 1, "exactly one adr: {files:?}");
    let text = std::fs::read_to_string(r.0.join(".ank/adr").join(&files[0])).unwrap();
    assert!(text.contains("Do not do X."), "{text}");
    // Never born accepted and never born anchored: ratification is a signed
    // commit produced by `accept`, and an ADR that arrived ratified would bind
    // before anyone agreed to it.
    assert!(text.contains("status: proposed"), "{text}");
    assert!(!text.contains("ratified:"), "{text}");
}

/// The clause that keeps the interactive form from being the hole in the wall:
/// it refuses what the flag form refuses, in the flag form's words and with the
/// flag form's code.
#[test]
fn a_template_saved_untouched_is_refused_and_creates_nothing() {
    let r = Repo::new();

    let out = r.ank_edit("claude-code@ank", &["new", "task"], Some("true"));
    assert_eq!(
        code(&out),
        7,
        "the same code the flag form gives an empty scope: {}",
        stderr(&out)
    );
    let err = stderr(&out);
    assert!(err.contains("empty scope"), "says why: {err}");
    assert!(
        err.contains("ank new task --title"),
        "names the flag form: {err}"
    );
    assert!(
        entity_files(&r, "tasks").is_empty(),
        "nothing is created: {:?}",
        entity_files(&r, "tasks")
    );
}

/// §4 requires this one by name: `$EDITOR` unset is an environment failure, and
/// what it names as the way through is the flag form.
#[test]
fn new_without_an_editor_names_the_flag_form() {
    let r = Repo::new();

    let out = r.ank_edit("claude-code@ank", &["new", "task"], None);
    assert_eq!(code(&out), 9, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("EDITOR is not set"), "{err}");
    assert_eq!(
        err.lines().nth(1).map(str::trim),
        Some(r#"-> ank new task --title "<t>" --scope "<glob>""#),
        "{err}"
    );

    let out = r.ank_edit("marie@laptop", &["new", "adr"], None);
    assert_eq!(code(&out), 9, "{}", stderr(&out));
    assert!(stderr(&out).contains("--constraint"), "{}", stderr(&out));
    assert!(entity_files(&r, "tasks").is_empty());
}

/// The property the whole trigger rule was chosen for. A script that forgot
/// `--scope` has to stop, not sit in an editor waiting for somebody who is not
/// there — so a partial invocation stays the flag form and fails as it always
/// did, with an editor set and willing.
#[test]
fn the_flag_form_is_untouched_by_the_interactive_one() {
    let r = Repo::new();
    // An editor that would happily create a valid entity if it were ever run.
    // Reaching it is the failure this test exists to catch.
    let editor = editor_filling("Should never be created", "src/**", "x");

    let out = r.ank_edit(
        "claude-code@ank",
        &["new", "task", "--title", "Half an invocation"],
        Some(&editor),
    );
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("a scope is required"),
        "{}",
        stderr(&out)
    );
    assert!(
        entity_files(&r, "tasks").is_empty(),
        "the editor was reached: {:?}",
        entity_files(&r, "tasks")
    );

    // And the complete flag form still writes exactly what it always wrote.
    let out = r.ank_edit(
        "claude-code@ank",
        &["new", "task", "--title", "Scripted", "--scope", "src/**"],
        Some(&editor),
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let files = entity_files(&r, "tasks");
    assert_eq!(files.len(), 1, "{files:?}");
    let text = std::fs::read_to_string(r.0.join(".ank/tasks").join(&files[0])).unwrap();
    assert!(text.contains("title: Scripted"), "{text}");
    assert!(text.contains("  - src/**"), "{text}");
}

/// The id hashes the act of creation (§3) and the file name carries it. Letting
/// the caller choose one would let them collide with an entity that exists, or
/// mint a reference nothing can resolve.
#[test]
fn a_template_that_comes_back_with_another_id_is_refused() {
    let r = Repo::new();
    let editor = editor_filling_and(
        "Renamed",
        "src/**",
        r"s|^id: TASK-.*|id: TASK-000000000009|",
    );

    let out = r.ank_edit("claude-code@ank", &["new", "task"], Some(&editor));
    assert_eq!(code(&out), 6, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("cannot be chosen"),
        "{}",
        stderr(&out)
    );
    assert!(entity_files(&r, "tasks").is_empty());
}

/// Reaching the editor does not mean nothing was said. A flag that is not one of
/// the mandatory ones carries into the template rather than being dropped.
#[test]
fn the_flags_that_were_given_are_carried_into_the_template() {
    let r = Repo::new().with_verifiers("verifiers:\n  unit:\n    run: 'true'\n    timeout: 1m\n");
    let editor = editor_filling("Pre-filled", "src/**", "x");

    let out = r.ank_edit(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--criteria",
            "The binary answers.",
            "--verify",
            "unit",
        ],
        Some(&editor),
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let files = entity_files(&r, "tasks");
    assert_eq!(files.len(), 1, "{files:?}");
    let text = std::fs::read_to_string(r.0.join(".ank/tasks").join(&files[0])).unwrap();
    assert!(text.contains("The binary answers."), "{text}");
    assert!(text.contains("verify: [unit]"), "{text}");
    // Set by the creator, because that is who typed it — the same thing the flag
    // form records.
    assert!(text.contains("criteria_by: creator"), "{text}");
}

/// The references are resolved at the point of creation, exactly as the flag
/// form resolves `--blocked-by`: an unknown one would otherwise surface in
/// `check`, long afterwards, as a corpus error nobody can attribute to the act.
#[test]
fn a_blocker_that_does_not_exist_is_refused_at_creation() {
    let r = Repo::new();
    let editor = editor_filling_and(
        "Blocked on nothing",
        "src/**",
        r"s|^blocked_by: .*|blocked_by: [TASK-000000000404]|",
    );

    let out = r.ank_edit("claude-code@ank", &["new", "task"], Some(&editor));
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("TASK-000000000404"), "{err}");
    assert!(
        err.contains("ank find"),
        "the exact command to run next: {err}"
    );
    assert!(entity_files(&r, "tasks").is_empty());
}

// ---------------------------------------------------------------------------
// Color: the guarantee is negative (§4, ADR-962c25797569)
// ---------------------------------------------------------------------------

/// Every verb worth styling, exercised against a corpus that actually has
/// something to style: a claimed task, an ADR, a blocker, a finding.
///
/// Kept as one fixture and one list because the property under test is
/// universal — "no verb" is the claim, so a list that quietly omits one is the
/// hole. `--version` and `help` are in it for the same reason they answer
/// before the foundation: they are reachable when nothing else is.
fn styled_surface() -> Vec<Vec<&'static str>> {
    vec![
        vec!["context"],
        vec!["status"],
        vec!["find", "Example"],
        vec!["find", "nothing-matches-this"],
        vec!["show", ID],
        // The ADR too: its `constraint` is a block scalar, which is the shape
        // the entity painter has to walk past without touching, and a task's
        // `show` alone never reaches it.
        vec!["show", "ADR-0000000000ab"],
        vec!["log", ID],
        vec!["graph"],
        vec!["scope", "src"],
        vec!["check"],
        vec!["review"],
        vec!["help"],
        vec!["--version"],
    ]
}

fn color_fixture() -> Repo {
    let r = Repo::new();
    r.seed_task(ID, Some("a criterion to freeze"));
    r.seed_task("TASK-000000000002", Some("another criterion"));
    r.seed_adr("ADR-0000000000ab", "a rule that binds", "src/**");
    r.blocked("TASK-000000000002", &[ID]);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    // A live claim: markers, warnings and the execution mode of `context` are
    // all unreachable without one, and they are exactly what carries colour.
    r.ank("claude-code@ank", &["claim", ID]);
    r
}

/// The one assertion the whole feature exists to satisfy.
///
/// A spawned process writes to a pipe, so this suite *is* the piped case — the
/// same shape an agent shelling out to `ank` sees. An escape byte anywhere in
/// either stream is the defect ADR-962c25797569 forbids, and `--json` is
/// checked beside the plain form because it is the one surface that must stay
/// clean even at a terminal.
#[test]
fn no_verb_writes_an_escape_sequence_to_a_pipe() {
    let r = color_fixture();

    for base in styled_surface() {
        for json in [false, true] {
            let mut args = base.clone();
            if json {
                args.push("--json");
            }
            let out = r.ank("claude-code@ank", &args);
            assert!(
                !out.stdout.contains(&0x1b),
                "{args:?} put an escape sequence on stdout: {:?}",
                stdout(&out)
            );
            assert!(
                !out.stderr.contains(&0x1b),
                "{args:?} put an escape sequence on stderr: {:?}",
                stderr(&out)
            );
        }
    }
}

/// Structure ships in the bytes, and `--json` carries none of it.
///
/// The other half of ADR-0c8ab846d262, and the half a test can only make in a
/// pipe: this suite spawns the binary, so its stdout *is* the pipe an agent
/// reads. Colour must be absent from it and the connectors must be present —
/// the two assertions point in opposite directions on purpose, because a
/// gate that confused the two layers would satisfy either one alone.
#[test]
fn a_pipe_receives_the_drawing_and_never_an_escape_sequence() {
    let r = color_fixture();

    let drawn = r.ank("claude-code@ank", &["graph"]);
    let said = stdout(&drawn);
    assert!(
        !drawn.stdout.contains(&0x1b),
        "graph coloured a pipe: {said:?}"
    );
    assert!(
        said.contains("└── ") || said.contains("├── "),
        "the connectors are text, so a pipe gets them: {said}"
    );

    // `show` draws the same relation from one task's point of view.
    let shown = stdout(&r.ank("claude-code@ank", &["show", ID]));
    assert!(
        shown.contains("UNBLOCKS (1)") && shown.contains("└── "),
        "show draws its edges: {shown}"
    );

    // The machine surface carries neither layer. Asserted over the whole
    // alphabet rather than over one connector: a new glyph reaching `--json`
    // through some other verb is exactly what this is here to catch.
    for verb in [
        vec!["graph", "--json"],
        vec!["show", ID, "--json"],
        vec!["find", "Example", "--json"],
        vec!["context", "--json"],
    ] {
        let out = r.ank("claude-code@ank", &verb);
        let j = stdout(&out);
        assert!(!out.stdout.contains(&0x1b), "{verb:?} coloured json: {j:?}");
        for glyph in ['│', '├', '└', '─'] {
            assert!(!j.contains(glyph), "{verb:?} drew {glyph:?} into json: {j}");
        }
    }
}

/// The held row is marked, and the margin it is drawn in was already there.
#[test]
fn a_listing_marks_the_row_the_caller_holds() {
    let r = color_fixture();

    // `color_fixture` leaves the claim with claude-code@ank.
    let mine = stdout(&r.ank("claude-code@ank", &["find", "Example"]));
    let theirs = stdout(&r.ank("someone-else@ank", &["find", "Example"]));

    assert!(
        mine.lines().any(|l| l.starts_with("* TASK-")),
        "the holder sees their own row marked: {mine}"
    );
    assert!(
        theirs.lines().all(|l| !l.starts_with("* ")),
        "and nobody else does: {theirs}"
    );
    // Same width for both readers: the marker is spent out of the margin, not
    // added to it, so a listing does not reflow depending on who is asking.
    for (a, b) in mine.lines().zip(theirs.lines()) {
        assert_eq!(a.len(), b.len(), "{a:?} and {b:?} are not the same width");
    }
}

/// A value that is valid for the flag, so that what is measured is the flag
/// and not the parse of its argument.
///
/// Measured the hard way first: a generic `x` makes `--limit` a code 1 and
/// `--type` a code 1, neither of which is a refusal of the flag -- it is the
/// verb correctly rejecting a value that is not a number and not a kind. A test
/// that read those as refusals would have failed on four verbs that are right.
fn valid_value(flag: &str) -> &'static str {
    match flag {
        "--limit" => "5",
        "--type" => "task",
        "--status" => "open",
        "--ttl" => "1h",
        "--proof" => "test:1",
        "--scope" | "--drop-scope" => "src/**",
        "--blocked-by" | "--drop-blocked-by" => "TASK-000000000001",
        "--supersedes" => "ADR-000000000001",
        "--verify" => "cargo-test",
        "--criteria" => "A measurable thing.",
        "--reason" => "a reason",
        "--title" => "A title",
        "--constraint" => "A rule.",
        "--body" => "Some prose.",
        other => panic!("no valid value declared for {other}: add one rather than guess"),
    }
}

/// The flags a verb's own help offers, read off the human output because that
/// is the surface the claim is about.
fn listed_flags(r: &Repo, verb: &str) -> Vec<String> {
    let out = stdout(&r.ank("claude-code@ank", &["help", verb]));
    out.lines()
        .find(|l| l.trim_start().starts_with("flags:"))
        .map(|l| {
            l.split_whitespace()
                .filter(|t| t.starts_with("--"))
                .map(|t| t.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// **A flag `ank help` offers is a flag the verb can actually be given** (§9).
///
/// The defect this exists for: `amend` listed `--criteria` among its flags and
/// refused it unconditionally, by name and by design. Help did not merely say
/// too little there, it made an offer the verb rejects -- and the refusal's own
/// hint pointed at `ank release`, which a task that was never claimed cannot
/// run. It was found by reading `human.rs`, which is the recovery route that
/// does not exist in a repository where ank arrives as a binary and a SKILL.md.
///
/// The shape is the one `msrv-tight` uses: a negative test that attributes the
/// drift rather than leaving it for a human to notice two revisions later.
///
/// **The invariant.** Against a fixed repository state, adding one flag the
/// help lists must not change the verb's exit code. A flag that is refused by
/// name fires before the verb looks at the repository, so it replaces the
/// ordinary answer -- `amend 9999` is a 2, and `amend 9999 --criteria ...` was a
/// 6, which is exactly the signature.
///
/// **One exception, and it is the flag doing its job.** When the baseline is a
/// 7 -- a missing prerequisite -- a flag is allowed to change the code, because
/// supplying the prerequisite is what `close --reason` and `release --reason`
/// are for. Measured rather than assumed: those two are the only verbs where it
/// happens.
#[test]
fn every_flag_the_help_offers_can_be_given_to_the_verb() {
    let r = color_fixture();
    // `sh -c true` on all three platforms: git supplies the shell on Windows.
    // Without it `new` opens an editor and the run has no deterministic end.
    let env = [("EDITOR", Some("true"))];

    // Every verb §4 lists, with a positional that resolves to nothing so the
    // baseline is the verb's ordinary "not found" rather than real work.
    let verbs: [(&str, &[&str]); 20] = [
        ("context", &[]),
        ("claim", &["TASK-999999999999"]),
        ("show", &["TASK-999999999999"]),
        ("log", &["TASK-999999999999"]),
        ("done", &["TASK-999999999999"]),
        ("release", &["TASK-999999999999"]),
        ("new", &["task"]),
        ("find", &["nothing-matches-this"]),
        ("status", &[]),
        ("review", &[]),
        ("accept", &["ADR-999999999999"]),
        ("close", &["TASK-999999999999"]),
        ("amend", &["TASK-999999999999"]),
        ("attest", &["TASK-999999999999"]),
        ("edit", &["TASK-999999999999"]),
        ("graph", &[]),
        ("scope", &["src"]),
        ("check", &[]),
        ("init", &[]),
        ("help", &["find"]),
    ];

    let mut walked = 0;
    for (verb, positionals) in verbs {
        walked += 1;
        let flags = listed_flags(&r, verb);
        if flags.is_empty() {
            continue;
        }
        let mut base_args = vec![verb];
        base_args.extend_from_slice(positionals);
        let baseline = code(&r.ank_env("claude-code@ank", &base_args, &env));

        for flag in &flags {
            let value = valid_value(flag);
            let mut args = base_args.clone();
            args.push(flag);
            args.push(value);
            let got = code(&r.ank_env("claude-code@ank", &args, &env));
            assert!(
                got == baseline || baseline == 7,
                "`ank {verb}` offers {flag} and then answers {got} where it \
                 answers {baseline} without it: either the verb refuses a flag \
                 its help advertises, or the help advertises a flag the verb \
                 refuses"
            );
        }
    }
    assert_eq!(
        walked, 20,
        "a verb was added and this walk did not learn of it"
    );
}

/// One fact, one string, whichever verb prints it.
///
/// `context` reads the claim refs and every other listing read the index, so a
/// claimed task said `[claimed:who]` under one verb and `[in_progress]` under
/// four others — one state wearing two words, chosen by whichever verb the
/// reader happened to type. Asserted through the binary because the divergence
/// was between whole commands and not inside one function: each of these opens
/// the corpus its own way, and only running them proves they agree.
#[test]
fn every_verb_says_the_same_thing_about_a_claimed_task() {
    let r = color_fixture();

    // `color_fixture` leaves ID claimed by claude-code@ank, and the marker
    // names the holder whoever is asking — so a stranger reads the same words.
    let held = format!("[claimed:{}]", "claude-code@ank");

    for args in [
        vec!["context"],
        vec!["find", "Example"],
        vec!["scope", "src"],
        vec!["graph"],
        // The reverse edge of the task ID blocks: `show` reaches ID through
        // another task's `BLOCKED BY`, which is the fourth listing that used to
        // print the index's word instead.
        vec!["show", "TASK-000000000002"],
    ] {
        let said = stdout(&r.ank("someone-else@ank", &args));
        assert!(
            said.contains(&held),
            "`ank {}` does not say {held}: {said}",
            args.join(" ")
        );
        assert!(
            !said.contains("[in_progress]"),
            "`ank {}` still prints the index's word: {said}",
            args.join(" ")
        );
    }
}

/// The transition grammar of §4, walked through the binary.
///
/// Every one of these lines was bare before TASK-4601ed18d84e, and none of them
/// can live in `styled_surface`: that list is replayed against one fixture, and
/// each line here is produced by a verb that mutates. So they are walked once,
/// in the loop's own order — a task created, taken, logged, released, retaken,
/// finished and attested; a second one amended and closed.
///
/// Two assertions per line, and the second is the one that would survive a
/// wrong colour. Absence of an escape byte proves the pipe is clean; the shape
/// proves the grammar is the one the specification declares, because a call
/// site that painted the wrong token would still be escape-free here.
#[test]
fn every_transition_line_reads_one_grammar_and_stays_plain_in_a_pipe() {
    let r = Repo::new();
    r.seed_task(ID, Some("a criterion to freeze"));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let run = |args: &[&str]| -> String {
        let out = r.ank("claude-code@ank", args);
        assert_eq!(code(&out), 0, "{args:?}: {}", stderr(&out));
        assert!(
            !out.stdout.contains(&0x1b),
            "{args:?} coloured a pipe: {:?}",
            stdout(&out)
        );
        assert!(
            !out.stderr.contains(&0x1b),
            "{args:?} coloured stderr: {:?}",
            stderr(&out)
        );
        stdout(&out)
    };

    let created = run(&[
        "new",
        "task",
        "--title",
        "A second task",
        "--scope",
        "src/**",
        "--criteria",
        "The binary answers.",
    ]);
    assert!(
        created.starts_with("created TASK-"),
        "created names what it made: {created:?}"
    );
    let second = created
        .split_whitespace()
        .nth(1)
        .expect("created <id>")
        .to_string();

    let claimed = run(&["claim", &second]);
    assert!(claimed.starts_with("claimed TASK-"), "{claimed:?}");

    let logged = run(&["log", &second, "something learned"]);
    assert!(logged.starts_with("logged on TASK-"), "{logged:?}");

    let released = run(&["release", &second, "--reason", "the criterion is wrong"]);
    assert!(
        released.starts_with("released TASK-") && released.trim_end().ends_with("-> open"),
        "a release names the state it lands on: {released:?}"
    );

    run(&["claim", &second]);
    let finished = run(&["done", &second, "--proof", "test:ci-run-1"]);
    assert!(finished.trim_end().ends_with("-> done"), "{finished:?}");

    let attested = run(&["attest", &second, "--proof", "test:ci-run-2"]);
    assert!(attested.starts_with("attested TASK-"), "{attested:?}");

    let amended = run(&["amend", ID, "--scope", "docs/**"]);
    assert!(amended.starts_with("amended TASK-"), "{amended:?}");

    let closed = run(&["close", ID, "--reason", "superseded by the second"]);
    assert!(
        closed.starts_with("closed TASK-") && closed.trim_end().ends_with("-> closed"),
        "{closed:?}"
    );
}

/// The error envelope travels the same rule, and it is the one line that does
/// not go through the writer every verb shares.
#[test]
fn a_refusal_is_plain_in_a_pipe_too() {
    let r = color_fixture();

    // One refusal per stream-producing shape: an unknown entity, a held claim,
    // and a flag error caught before any verb runs.
    for args in [
        vec!["claim", "TASK-00000000dead"],
        vec!["claim", "TASK-000000000002"],
        vec!["find", "bug", "-st", "task"],
        vec!["context", "--limit", "not-a-number"],
    ] {
        let out = r.ank("someone-else@ank", &args);
        assert_ne!(code(&out), 0, "{args:?} was supposed to refuse");
        assert!(
            !out.stderr.contains(&0x1b),
            "{args:?} coloured a refusal into a pipe: {:?}",
            stderr(&out)
        );
        assert!(
            stderr(&out).starts_with("error["),
            "{args:?} lost the envelope: {:?}",
            stderr(&out)
        );
    }
}

/// The detection rule read from the other side: nothing in the environment can
/// turn colour *on*, because the terminal test has already failed.
///
/// This is what catches a wiring inversion. `NO_COLOR` implemented backwards,
/// or a Windows allowlist consulted instead of the terminal check, would leave
/// every assertion above green and change the bytes here.
#[test]
fn no_environment_makes_a_pipe_colored() {
    let r = color_fixture();

    let environments: [&[(&str, Option<&str>)]; 6] = [
        &[("NO_COLOR", None), ("TERM", None), ("WT_SESSION", None)],
        &[("NO_COLOR", Some("1"))],
        &[("NO_COLOR", Some(""))],
        &[("TERM", Some("dumb"))],
        &[("TERM", Some("xterm-256color"))],
        // The Windows allowlist, set on every platform: it is only ever an
        // additional condition, never a substitute for the terminal itself.
        &[("WT_SESSION", Some("1")), ("ANSICON", Some("1"))],
    ];

    for base in styled_surface() {
        let reference = r.ank_env(
            "claude-code@ank",
            &base,
            &[("NO_COLOR", None), ("TERM", None), ("WT_SESSION", None)],
        );
        for env in environments {
            let out = r.ank_env("claude-code@ank", &base, env);
            assert!(
                !out.stdout.contains(&0x1b),
                "{base:?} under {env:?} coloured a pipe"
            );
            assert_eq!(
                out.stdout, reference.stdout,
                "{base:?} answered differently under {env:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// `ank config` (§4, ADR-e64dfaafd578)
// ---------------------------------------------------------------------------
//
// Through the binary, and not only through the module, because every clause of
// the criterion is phrased about `ank config <key>` -- and because the sharpest
// of them is a claim about dispatch: the verb answers on a `config.yml` that
// fails `startup`, which no unit test on the writer could reach.

/// Trimmed stdout of one invocation, which is what a caller reads.
fn said(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

fn erred(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// A `config.yml` carrying every awkward form at once: comments on their own
/// line and after a value, blank lines, verifiers out of alphabetical order, a
/// quoted `run`, and a verifier with no `timeout`.
const AWKWARD: &str = "\
# Reviewed like code, and it stays reviewable.
schema: 1

claim_ttl_max: 2h   # renewed by ank log
default_branch: main

verifiers:
  # Out of alphabetical order on purpose.
  fmt-check:
    run: \"cargo fmt --check\"
  cargo-test:
    run: cargo test --workspace -q
    timeout: 30m
";

#[test]
fn config_reads_the_value_in_effect_and_marks_a_resolved_default() {
    let r = Repo::new();
    r.set_config(AWKWARD);
    let say = |args: &[&str]| said(&r.ank("claude-code@ank", args));

    assert_eq!(say(&["config", "claim_ttl_max"]), "2h");
    assert_eq!(say(&["config", "default_branch"]), "main");
    // A quoted value reads as its value, not as its spelling.
    assert_eq!(
        say(&["config", "verifiers.fmt-check.run"]),
        "cargo fmt --check"
    );
    assert_eq!(say(&["config", "verifiers.cargo-test.timeout"]), "30m");

    // Absent from the file and resolved by the tool, said to be so: the
    // difference between "the tool's value" and "this repository's value" is
    // the whole question a reader is asking, and a release that moves a default
    // moves the first and not the second.
    assert_eq!(say(&["config", "context_budget"]), "8000 (default)");
    assert_eq!(
        say(&["config", "verifiers.fmt-check.timeout"]),
        "10m (default)"
    );

    // Absent with nothing to resolve.
    r.set_config("schema: 1\n");
    assert_eq!(say(&["config", "default_branch"]), "(unset)");

    // The marker is on the human surface alone: --json splits the two, which
    // is what a script reads.
    r.set_config(AWKWARD);
    let json = say(&["config", "context_budget", "--json"]);
    assert!(json.contains("\"value\":\"8000\""), "{json}");
    assert!(json.contains("\"source\":\"default\""), "{json}");
    let json = say(&["config", "claim_ttl_max", "--json"]);
    assert!(json.contains("\"source\":\"file\""), "{json}");
}

#[test]
fn config_writes_the_key_and_no_byte_beside_it() {
    let r = Repo::new();
    r.set_config(AWKWARD);

    let out = r.ank("claude-code@ank", &["config", "claim_ttl_max", "4h"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert_eq!(said(&out), "claim_ttl_max 2h -> 4h");

    // Every line but the one named is byte-identical, comment column included.
    let after = r.config_text();
    let moved: Vec<(&str, &str)> = AWKWARD
        .lines()
        .zip(after.lines())
        .filter(|(a, b)| a != b)
        .collect();
    assert_eq!(
        moved,
        vec![(
            "claim_ttl_max: 2h   # renewed by ank log",
            "claim_ttl_max: 4h   # renewed by ank log"
        )],
        "more than the named key moved"
    );
    assert_eq!(AWKWARD.lines().count(), after.lines().count());

    // No default was materialised on the way: context_budget was absent and
    // stays absent, and fmt-check still declares no timeout.
    assert!(!after.contains("context_budget"), "{after}");
    let fmt = after.split("fmt-check:").nth(1).unwrap();
    assert!(!fmt.split("cargo-test:").next().unwrap().contains("timeout"));

    // Nested values are addressed by dotted path, and a new verifier is
    // declared by writing its run -- which is what the six hints now name.
    let out = r.ank(
        "claude-code@ank",
        &["config", "verifiers.audit.run", "cargo audit"],
    );
    assert!(out.status.success(), "{}", erred(&out));
    assert_eq!(said(&out), "verifiers.audit.run (unset) -> cargo audit");
    assert!(r.config_text().contains("  audit:\n    run: cargo audit\n"));

    // Writing what the file already says touches nothing at all.
    let before = r.config_text();
    let out = r.ank(
        "claude-code@ank",
        &["config", "verifiers.audit.run", "cargo audit"],
    );
    assert_eq!(said(&out), "verifiers.audit.run cargo audit (unchanged)");
    assert_eq!(r.config_text(), before);
}

#[test]
fn config_unset_removes_the_key_and_gives_the_file_back() {
    let r = Repo::new();
    r.set_config(AWKWARD);

    let out = r.ank("claude-code@ank", &["config", "--unset", "default_branch"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert_eq!(said(&out), "default_branch main -> (unset)");
    assert!(!r.config_text().contains("default_branch"));

    // Set then unset is the identity, byte for byte -- and the short form of
    // §4's table does the same thing as the long one.
    r.ank("claude-code@ank", &["config", "default_branch", "main"]);
    r.ank("claude-code@ank", &["config", "-u", "default_branch"]);
    let stripped = AWKWARD.replace("default_branch: main\n", "");
    assert_eq!(r.config_text(), stripped);

    // A whole verifier is what --unset addresses, which is what makes
    // declaring one reversible.
    r.set_config(AWKWARD);
    r.ank(
        "claude-code@ank",
        &["config", "verifiers.audit.run", "cargo audit"],
    );
    let out = r.ank("claude-code@ank", &["config", "--unset", "verifiers.audit"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert_eq!(r.config_text(), AWKWARD);

    // And `run` alone is refused, naming the command that does the job.
    let out = r.ank(
        "claude-code@ank",
        &["config", "--unset", "verifiers.cargo-test.run"],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(
        erred(&out).contains("ank config --unset verifiers.cargo-test"),
        "{}",
        erred(&out)
    );
    assert_eq!(r.config_text(), AWKWARD, "a refusal wrote to the file");
}

#[test]
fn config_refuses_an_unknown_key_by_name_and_writes_nothing() {
    let r = Repo::new();
    r.set_config(AWKWARD);

    let out = r.ank("claude-code@ank", &["config", "budget_context", "10"]);
    assert_eq!(out.status.code(), Some(1));
    let err = erred(&out);
    assert!(err.contains("budget_context"), "{err}");
    for key in [
        "schema",
        "context_budget",
        "claim_ttl_max",
        "default_branch",
        "verifiers.<name>.run",
        "verifiers.<name>.timeout",
    ] {
        assert!(err.contains(key), "{key} missing from the refusal: {err}");
    }
    assert_eq!(r.config_text(), AWKWARD);

    // Known to the parser, structured, and refused by name rather than guessed
    // at -- a different message from "no such key".
    for key in ["roles", "identities", "roles.agent.can"] {
        let out = r.ank("claude-code@ank", &["config", key, "x"]);
        assert_eq!(out.status.code(), Some(1), "{key}");
        assert!(erred(&out).contains("structured"), "{key}: {}", erred(&out));
    }

    // A timeout cannot declare a verifier, and the refusal names what can.
    let out = r.ank(
        "claude-code@ank",
        &["config", "verifiers.nope.timeout", "5m"],
    );
    assert_eq!(out.status.code(), Some(7));
    assert!(
        erred(&out).contains("ank config verifiers.nope.run"),
        "{}",
        erred(&out)
    );
    assert_eq!(r.config_text(), AWKWARD);
}

#[test]
fn config_refuses_a_write_that_would_leave_the_file_unreadable() {
    let r = Repo::new();
    r.set_config(AWKWARD);

    // Each of these parses as YAML and fails the configuration's own reading:
    // a duration in no unit it knows, and a schema it does not support.
    for (key, value) in [("claim_ttl_max", "30w"), ("schema", "2")] {
        let out = r.ank("claude-code@ank", &["config", key, value]);
        assert_eq!(out.status.code(), Some(1), "{key} was accepted");
        assert!(erred(&out).contains("unreadable"), "{key}: {}", erred(&out));
        assert_eq!(r.config_text(), AWKWARD, "{key} left the file changed");
    }

    // A block scalar is refused before any of that: flattening it would move
    // the definition hash that anchors historical proofs.
    r.set_config("schema: 1\nverifiers:\n  ci:\n    run: |\n      cargo test\n");
    let before = r.config_text();
    let out = r.ank(
        "claude-code@ank",
        &["config", "verifiers.ci.run", "cargo test -q"],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(erred(&out).contains("verifiers.ci.run"), "{}", erred(&out));
    assert_eq!(r.config_text(), before);
}

#[test]
fn config_is_the_one_verb_a_config_that_does_not_parse_does_not_stop() {
    let r = Repo::new();
    let broken = "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\nbudget_context: 10\n";
    r.set_config(broken);

    // `startup` loads the configuration for every other verb, so the file
    // fails all of them -- `check`, the one an agent would reach for, included.
    for args in [
        &["check"][..],
        &["context"][..],
        &["status"][..],
        &["find", "anything"][..],
    ] {
        let out = r.ank("claude-code@ank", args);
        assert_eq!(
            out.status.code(),
            Some(1),
            "{args:?} ran on a config.yml that does not parse"
        );
    }

    // The verb that exists to repair the file is the one the broken file does
    // not stop. It reads,
    let out = r.ank("claude-code@ank", &["config", "claim_ttl_max"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert_eq!(said(&out), "2h");

    // and it writes, saying that the file is still unreadable for a reason it
    // was not asked about. Refusing here instead would be refusing exactly
    // where the verb is needed.
    let out = r.ank("claude-code@ank", &["config", "claim_ttl_max", "4h"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert!(r.config_text().contains("claim_ttl_max: 4h"));
    assert!(said(&out).contains("warning:"), "{}", said(&out));
    assert!(said(&out).contains("budget_context"), "{}", said(&out));
}

#[test]
fn the_errors_that_named_the_file_now_name_the_command() {
    // ADR-01b6dd05f0db closed `.ank/` to agents and stopped at the
    // configuration, which left these telling their caller to open a file the
    // same tool forbids them to open (ADR-e64dfaafd578).
    let r = Repo::new();

    // `new`, on a --verify that matches nothing. The criterion calls the two
    // verify sites "new's and amend's"; they are in fact both `new`'s -- the
    // flag form here, and the `verify:` filled into the $EDITOR template,
    // which `commands::check_verifiers` guards and a unit test covers. `amend`
    // takes no --verify at all.
    let out = r.ank(
        "claude-code@ank",
        &[
            "new", "task", "--title", "T", "--scope", "src/**", "--verify", "nope",
        ],
    );
    assert_eq!(out.status.code(), Some(7), "{}", erred(&out));
    let err = erred(&out);
    assert!(err.contains("ank config verifiers.nope.run"), "{err}");
    assert!(
        !err.contains("under verifiers: in .ank/config.yml"),
        "it still tells the caller to open the file: {err}"
    );

    // `done`, on a task declaring a verifier the configuration does not.
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: git --version\n");
    r.seed_task_with(
        "TASK-000000000c01",
        Some("A verifiable criterion."),
        &["ok"],
    );
    r.ank("claude-code@ank", &["claim", "TASK-000000000c01"]);
    r.set_config("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n");
    let out = r.ank("claude-code@ank", &["done", "TASK-000000000c01"]);
    assert_eq!(out.status.code(), Some(9), "{}", erred(&out));
    assert!(
        erred(&out).contains("ank config verifiers.ok.run"),
        "{}",
        erred(&out)
    );

    // The four sites that ask for a default branch. There is no
    // `default_branch` and no `refs/remotes/origin/HEAD` to fall back on, so
    // each of them has to name the command that sets one.
    let r = Repo::new();
    r.set_config("schema: 1\nclaim_ttl_max: 2h\n");
    std::fs::write(r.0.join("seed.txt"), "x").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["-c", "commit.gpgsign=false", "commit", "-qm", "seed"]);

    r.seed_adr("ADR-000000000c02", "A binding rule.", "src/**");
    for args in [
        &["status"][..],
        &["check"][..],
        &["context"][..],
        &["accept", "ADR-000000000c02"][..],
    ] {
        let out = r.ank("claude-code@ank", args);
        let text = format!("{}{}", said(&out), erred(&out));
        assert!(
            text.contains("ank config default_branch"),
            "{args:?} does not name the command: {text}"
        );
        assert!(
            !text.contains("\"default_branch: <name>\""),
            "{args:?} still quotes the line to add by hand: {text}"
        );
    }
}

#[test]
fn help_answers_about_config_the_way_it_answers_about_every_verb() {
    let r = Repo::new();
    let text = said(&r.ank("claude-code@ank", &["help", "config"]));
    assert!(text.contains("ank config <key> [<value>]"), "{text}");
    // The short form of §4's table, on the surface that teaches one verb.
    assert!(text.contains("-u, --unset"), "{text}");
    // What it refuses, with the code -- the question is asked before the call.
    assert!(text.contains("(1)") && text.contains("(7)"), "{text}");
    // The key set, which is the one thing a caller cannot guess.
    assert!(text.contains("verifiers.<name>.run"), "{text}");

    // And the flat listing stays flat: one line for the verb, no heading.
    let listing = said(&r.ank("claude-code@ank", &["help"]));
    assert!(listing.contains("ank config <key> [<value>]"), "{listing}");
    assert!(listing.contains("--unset"), "{listing}");
}
