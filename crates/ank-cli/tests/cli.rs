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
    // happens to sit under, and no dependencies so the build needs no registry.
    std::fs::write(
        dir.join("Cargo.toml"),
        "[workspace]\n\n[package]\nname = \"stamp\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
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
            .args(["run", "-q"])
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
    // Empty parentheses would satisfy the shape and answer nothing.
    let commit = said
        .rsplit_once('(')
        .and_then(|(_, c)| c.strip_suffix(')'))
        .unwrap_or("");
    assert!(
        commit.len() >= 4,
        "a sha or the word `unknown`, never a blank: {said}"
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
