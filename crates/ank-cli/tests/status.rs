//! `status` pays for the answer it gives (ADR-f3d1dea65d84), through the binary.
//!
//! The verb used to walk both storage layouts and parse every entity file in
//! the corpus so that two integers could be printed. What it does now is ask
//! the index, which already holds the parse, and pay the full price only when
//! the state the answer depends on has moved. Both halves of that sentence are
//! tested here, and the second is the one that matters: **a cache whose key is
//! wrong makes the verb lie**, and every test below that changes something and
//! asserts the counters moved is a test of the key rather than of the speed.
//!
//! Through the binary, and it has to be. The cost is a process cost — an index
//! opened, git run, a corpus walked or not walked — and the measurement of it
//! is only honest at the surface a reader actually uses. An integration test is
//! also the only place `CARGO_BIN_EXE_ank` is defined (`ank-cli` has no library
//! target), so there is no unit test that could spawn it even if we wanted one.

mod scratch;

use std::ffi::OsStr;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// A corpus of at least this many entities, which is what the criterion asks
/// the measurement to be made on: below a thousand, a walk of every file is
/// cheap enough that a verb doing it looks like a verb that does not.
const ENTITIES: usize = 1000;

/// What every test that is not about the cost builds instead.
///
/// **The size is the measurement's, not the behaviour's.** Whether a counter
/// follows an edit is the same question over eight entities as over a thousand,
/// and building a thousand of them six times over made the one test that does
/// measure something share a disk with five that did not — which is a
/// measurement of cargo's parallelism.
const FEW: usize = 8;

/// An identifier outside any fixture, for the tests that add one.
const SPARE: usize = 90_000;

/// The wall the verb answers inside, on a warm index. The criterion's number.
///
/// **Still an order of magnitude under what it replaces**, which is the margin
/// that makes it a wall and not a stopwatch: the same fixture cost about two
/// seconds before this change, measured with an *optimised* build against the
/// unoptimised one the suite runs. Observed at 84ms on the macOS runner, 156ms
/// on Linux, and 154-213ms on a four-core machine already carrying other
/// builds, taking the floor of nine runs.
const WALL: u128 = 250;

/// git's global and system configuration, for every process this suite spawns.
///
/// The same isolation `tests/cli.rs` explains at length: a machine that signs
/// commits by default would otherwise decide whether these fixtures can commit
/// at all. Written positively rather than left to a default, so the fixture
/// declares what it depends on.
fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::root().join(format!("ank-status-it-gitconfig-{}", std::process::id()));
        std::fs::write(&p, "[commit]\n\tgpgsign = false\n").unwrap();
        p
    })
    .as_path()
}

fn spawn(program: impl AsRef<OsStr>) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    c.env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config);
    c
}

/// A repository carrying a corpus of `ENTITIES` tasks.
///
/// The entity files are written rather than claimed into existence: a thousand
/// invocations of `ank new task` would cost a thousand processes to build a
/// fixture whose only interesting property is its size. What the tests below
/// assert is always asked of the binary; only the corpus is forged.
struct Corpus(PathBuf);

impl Corpus {
    /// A corpus small enough that building it is not itself the experiment.
    fn new() -> Corpus {
        Corpus::of(FEW)
    }

    fn of(entities: usize) -> Corpus {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let p = scratch::root().join(format!(
            "ank-status-it-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(p.join(".ank/entities")).unwrap();
        let c = Corpus(p);
        c.git(&["init", "-q", "-b", "main"]);
        c.git(&["config", "user.email", "test@ank.local"]);
        c.git(&["config", "user.name", "Test"]);
        c.git(&["config", "core.autocrlf", "false"]);
        c.git(&["config", "commit.gpgsign", "false"]);
        std::fs::write(
            c.0.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        // The index is derived, disposable and gitignored (§6). A fixture that
        // tracked it would turn the commit below into a commit of a cache.
        std::fs::write(c.0.join(".gitignore"), ".ank/index.db\n").unwrap();
        // A file the seeded scopes actually match. Without it every task in the
        // corpus is attached to nothing, which is a thousand dead-scope signals
        // and a thousand walks of git history to explain them — a fixture that
        // measures a defect it invented.
        std::fs::create_dir_all(c.0.join("src")).unwrap();
        std::fs::write(c.0.join("src/lib.rs"), "// seed\n").unwrap();
        for i in 0..entities {
            c.seed_task(&id(i), "open");
        }
        // Committed, because the drift comparison is against the default branch
        // and a corpus that was never committed is one every entity differs
        // from — a thousand signals about a state the fixture did not mean.
        c.git(&["add", "-A"]);
        c.git(&["commit", "-qm", "seed"]);
        c
    }

    fn git(&self, args: &[&str]) -> String {
        let out = spawn("git")
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

    fn ank(&self, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .current_dir(&self.0)
            .env("ANK_AGENT", "reader@fixture")
            .output()
            .expect("the binary under test must run")
    }

    /// The document, with a run that failed reported rather than parsed.
    fn json(&self, args: &[&str]) -> String {
        let out = self.ank(args);
        // `check` exits 8 on findings and that is not a failure here: a corpus
        // with signals is the corpus these tests are about.
        let code = out.status.code().expect("the process exits, not signals");
        assert!(
            code == 0 || code == 8,
            "ank {args:?} exited {code}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    fn counters(&self) -> (u64, u64) {
        let doc = self.json(&["status", "--json"]);
        (number(&doc, "\"faults\":"), number(&doc, "\"signals\":"))
    }

    fn entity(&self, id: &str) -> PathBuf {
        self.0.join(".ank/entities").join(format!("{id}.md"))
    }

    fn seed_task(&self, id: &str, status: &str) {
        std::fs::write(
            self.entity(id),
            format!(
                "---\nid: {id}\ntype: task\nslug: example\ntitle: Example task\n\
                 created: 2026-07-28T00:00:00Z\nstatus: {status}\nscope:\n  - src/**\n\
                 blocked_by: []\ndone_criteria: |\n  It works.\ncriteria_by: creator\n\
                 schema: 1\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
    }

    /// Writes a claim record straight onto its ref, touching no file.
    ///
    /// **The whole of the refs half of the key rests on this being possible.**
    /// `ank claim` moves the task file as well as the ref, so a test built on
    /// it could never say which of the two the answer had followed. A record
    /// written here changes `refs/ank/*` and nothing else, which is exactly the
    /// state the check reads out of the refs and out of no file at all.
    fn seed_claim(&self, id: &str, expires: &str) {
        let record = format!(
            "state: claim\nholder: someone@elsewhere\ntask: {id}\n\
             claimed: 2026-07-28T00:00:00Z\nexpires: {expires}\ncriteria: abcdefabcdefabcd\n\
             constraints: abcdefabcdefabcd\n"
        );
        let mut child = spawn("git")
            .current_dir(&self.0)
            .args(["hash-object", "-w", "--stdin"])
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
        assert!(out.status.success(), "hash-object failed");
        let blob = String::from_utf8_lossy(&out.stdout).trim().to_string();
        self.git(&["update-ref", &format!("refs/ank/claims/{id}"), &blob]);
    }

    fn delete_claim(&self, id: &str) {
        self.git(&["update-ref", "-d", &format!("refs/ank/claims/{id}")]);
    }
}

impl Drop for Corpus {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A well-formed identifier for the nth seeded task.
fn id(n: usize) -> String {
    format!("TASK-{:012x}", 0x1000_0000_0000u64 + n as u64)
}

/// The number that follows `name` in a JSON document.
///
/// Read out of the bytes rather than through a parser, because this suite
/// declares no dependencies and the shape being asserted is exactly the one a
/// script matching on the key would see.
fn number(doc: &str, name: &str) -> u64 {
    let at = doc
        .find(name)
        .unwrap_or_else(|| panic!("{name} is in the document: {doc}"));
    doc[at + name.len()..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or_else(|_| panic!("{name} carries a number: {doc}"))
}

/// The fastest of several runs, which is the only honest reading of a wall
/// clock on a shared machine.
///
/// A test that reported the *median* would report the other tests cargo is
/// running beside it; a test that reported one run would report whichever
/// scheduler slice it landed in. The floor is the measurement — it is the run
/// that was allowed to do its work — and it is the one a reader can reproduce.
fn fastest(corpus: &Corpus, args: &[&str], runs: usize) -> u128 {
    let mut best = u128::MAX;
    for _ in 0..runs {
        let at = Instant::now();
        corpus.json(args);
        best = best.min(at.elapsed().as_millis());
    }
    best
}

// ---------------------------------------------------------------------------
// The cost
// ---------------------------------------------------------------------------

/// Both cost claims, over one corpus, because building a thousand entities is
/// the expensive part of asking either of them.
#[test]
fn status_answers_a_thousand_entities_in_under_a_quarter_second() {
    let c = Corpus::of(ENTITIES);
    // Warm, which is the state the criterion measures: the index exists and
    // agrees with the files. A cold index pays for its own construction once,
    // and that cost belongs to whatever verb ran first.
    c.json(&["status", "--json"]);

    let status = fastest(&c, &["status", "--json"], 9);

    // **The ratio first, because it is the claim and it holds anywhere.**
    // `check` is the verb whose answer *is* the read of the corpus, and
    // ADR-f3d1dea65d84 says in as many words that it goes on costing what it
    // costs. A `status` still doing the same reading could only be a hair away
    // from it, whatever the machine — and a ratio, unlike a wall, cancels the
    // machine out.
    let check = fastest(&c, &["check", "--json"], 2);
    assert!(
        status * 2 < check,
        "status took {status}ms and check {check}ms: status is still reading the corpus"
    );

    // **The criterion's number, where that number measures the verb rather than
    // the runner.** It is not asserted on Windows, and the reason is measured
    // rather than assumed. `ank status --json` spawns thirteen git processes —
    // two `git --version`, two `rev-parse --git-common-dir`, two `rev-parse
    // --show-toplevel`, three `for-each-ref refs/ank/` (`claim::on_task`,
    // `context::plane`, and the enumeration the memoised verdict needs for its
    // key), and one each of `symbolic-ref HEAD`, `symbolic-ref
    // refs/remotes/origin/HEAD`, `rev-parse <branch>^{commit}` and `rev-list
    // --max-parents=0` — counted with a shim on the PATH. Process creation
    // costs roughly 25ms on a Windows runner against 2-3ms on Linux, so those
    // thirteen are about 325ms before the verb reads anything: remove the
    // corpus read entirely, as this change does, and Windows is still at the
    // wall. Eleven of the thirteen live in `git.rs`, `repo.rs`, `claim.rs` and
    // `context.rs`, and TASK-5690eae1e008 carries them.
    //
    // And the floor is not only high there, it moves: the same commit measured
    // 376ms and then 612ms on two runs of the same runner image. A wall set
    // above that would report the scheduler, and a test that reports the
    // scheduler is one nobody can act on. The ratio above is asserted on
    // Windows in its place, and it is the stronger claim.
    if !cfg!(windows) {
        assert!(
            status < WALL,
            "status --json over {ENTITIES} entities took {status}ms, wall is {WALL}ms"
        );
    }
}

// ---------------------------------------------------------------------------
// What the answer says
// ---------------------------------------------------------------------------

#[test]
fn the_counters_are_the_ones_check_reports() {
    let c = Corpus::new();
    // A task that says it is in progress with nothing holding it, so the corpus
    // carries a signal and the equality below is asserted over a number that is
    // not zero. Zero equals zero for reasons that have nothing to do with this.
    c.seed_task(&id(0), "in_progress");

    // `check` prunes and `status` does not, so the comparison is made after
    // `check` has had its chance: anything prunable is gone before either
    // number is read, and what is left is one corpus with one verdict.
    c.json(&["check", "--json"]);
    let checked = c.json(&["check", "--json"]);
    let (faults, signals) = c.counters();

    assert_eq!(faults, number(&checked, "\"faults\":"));
    assert_eq!(signals, number(&checked, "\"signals\":"));
    assert!(
        signals > 0,
        "the fixture carries a signal to compare: {checked}"
    );
}

#[test]
fn the_counters_stay_numbers_under_the_keys_they_had() {
    let c = Corpus::new();
    let doc = c.json(&["status", "--json"]);
    // The machine contract lets a document gain a field within a version and
    // never lose, rename or retype one. Speed was bought from where the answer
    // is computed, so `faults` and `signals` are still numbers, still present,
    // and still never null — which is what a caller matching on the key sees.
    for key in ["\"faults\":", "\"signals\":", "\"unmerged\":"] {
        let at = doc.find(key).unwrap_or_else(|| panic!("{key} in {doc}"));
        let value = &doc[at + key.len()..];
        assert!(
            value.starts_with(|ch: char| ch.is_ascii_digit()),
            "{key} carries a number and not {}",
            &value[..value.len().min(20)]
        );
    }
}

// ---------------------------------------------------------------------------
// The key: the files
// ---------------------------------------------------------------------------

#[test]
fn an_entity_written_edited_or_removed_moves_the_counters() {
    let c = Corpus::new();
    let (_, before) = c.counters();

    // Written: a task that says it is in progress with nothing holding it.
    c.seed_task(&id(SPARE), "in_progress");
    let (_, written) = c.counters();
    assert!(
        written > before,
        "a task in progress with no claim ref is a signal: {before} -> {written}"
    );

    // Edited: the same file, one field, and the signal it was carrying goes.
    c.seed_task(&id(SPARE), "open");
    let (_, edited) = c.counters();
    assert!(edited < written, "the edit is read: {written} -> {edited}");

    // Removed, and the corpus is the one the fixture started with — the same
    // files, byte for byte, so the same numbers to the digit.
    std::fs::remove_file(c.entity(&id(SPARE))).unwrap();
    assert_eq!(
        c.counters().1,
        before,
        "the file is gone and so is what it was signalling"
    );
}

// ---------------------------------------------------------------------------
// The key: the refs
// ---------------------------------------------------------------------------

#[test]
fn a_claim_ref_appearing_moves_the_counters_with_no_file_touched() {
    let c = Corpus::new();
    let (_, before) = c.counters();
    let files = c.git(&["status", "--porcelain"]);

    // A ref addressed to a task that does not exist. Nothing on disk moves —
    // this is the half of the corpus that lives in `refs/ank/*` and in no file
    // at all, which is why the files alone cannot be the key.
    c.seed_claim("TASK-ffffffffffff", "2099-01-01T00:00:00Z");
    let (_, orphaned) = c.counters();
    assert!(
        orphaned > before,
        "an orphan claim ref is a signal: {before} -> {orphaned}"
    );
    assert_eq!(
        files,
        c.git(&["status", "--porcelain"]),
        "the working tree is untouched, so only the refs can have said this"
    );

    c.delete_claim("TASK-ffffffffffff");
    assert_eq!(
        c.counters().1,
        before,
        "the ref is gone and so is what it was signalling"
    );
}

#[test]
fn a_claim_ref_going_stale_moves_the_counters_with_no_file_touched() {
    let c = Corpus::new();
    // The task says it is held, and a live record holds it: no signal either
    // way. Both are written by hand because what is being tested is the pair,
    // and `claim` would move the file and the ref together.
    c.seed_task(&id(0), "in_progress");
    c.seed_claim(&id(0), "2099-01-01T00:00:00Z");
    let (_, live) = c.counters();
    let files = c.git(&["status", "--porcelain"]);

    // The same ref, the same task file, an expiry in the past: this is a claim
    // that lapsed, byte for byte, and the record is the only thing that records
    // the passage of time. Forged rather than waited for — the tolerance on top
    // of the TTL is minutes, and a suite runs in one.
    c.seed_claim(&id(0), "2020-01-01T00:00:00Z");
    let (_, stale) = c.counters();
    assert!(
        stale > live,
        "an expired claim is a signal: {live} -> {stale}"
    );
    assert_eq!(
        files,
        c.git(&["status", "--porcelain"]),
        "the working tree is untouched, so only the refs can have said this"
    );

    // And back: the ref returns to live, the counter returns with it. A key
    // that only ever noticed the first change would pass everything above.
    c.seed_claim(&id(0), "2099-01-01T00:00:00Z");
    assert_eq!(c.counters().1, live, "the claim reads live again");
}
