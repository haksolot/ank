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

/// The plane the criterion is measured over: five hundred coordination refs.
const REFS: usize = 500;

/// An expiry no run of this suite reaches, so every seeded claim is a live one.
const LIVE: &str = "2099-01-01T00:00:00Z";

/// The plane the signature measurement is over: ratified decisions, on the
/// corpus that carries many of them.
const RATIFICATIONS: usize = 24;

/// How much smaller the other corpus is. The criterion asks for a factor of at
/// least three between the two, because two counts that are equal over a
/// difference of one prove nothing about a count that grows.
const FACTOR: usize = 6;

/// A `gpgsig` header no keyring produced and none can check, ending its own
/// line so that the blank line after it is the one that opens the message.
///
/// A signed commit carries this header whatever the format — `gpgsig-sha256` in
/// a SHA-256 repository, and an SSH signature uses the same key — and it is the
/// only thing that tells a commit nobody signed apart from one this machine
/// cannot verify. What is inside it is deliberately not decodable: the fixture
/// is about the header being *there*, and a runner with gpg and one without
/// reach the same verdict over bytes gpg cannot make sense of.
const FORGED_SIGNATURE: &str = concat!(
    "gpgsig -----BEGIN PGP SIGNATURE-----\n",
    " \n",
    " bm90IGEgc2lnbmF0dXJl\n",
    " -----END PGP SIGNATURE-----\n",
);

/// What git prefixes the argument list of every process it starts with, under
/// `GIT_TRACE`.
const MARK: &str = "trace: built-in: git ";

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

    /// `count` claim refs, written in two processes rather than two each.
    ///
    /// The plane the criterion names carries five hundred of them, and a
    /// fixture that spawned a thousand processes to build one would spend more
    /// on the forging than on the measurement. `hash-object --stdin-paths`
    /// writes every blob in one process and answers one name per line in order;
    /// `update-ref --stdin` creates every ref in one transaction. Neither is a
    /// shortcut around the state: what lands is exactly what [`seed_claim`]
    /// lands, ref for ref.
    ///
    /// [`seed_claim`]: Corpus::seed_claim
    fn seed_claims(&self, count: usize) {
        let dir = self.0.join("records");
        std::fs::create_dir_all(&dir).unwrap();
        let mut paths = String::new();
        let mut ids = Vec::new();
        for n in 0..count {
            let id = id(n);
            let path = dir.join(&id);
            std::fs::write(
                &path,
                format!(
                    "state: claim\nholder: someone@elsewhere\ntask: {id}\n\
                     claimed: 2026-07-28T00:00:00Z\nexpires: {LIVE}\ncriteria: abcdefabcdefabcd\n\
                     constraints: abcdefabcdefabcd\n"
                ),
            )
            .unwrap();
            paths.push_str(&path.display().to_string());
            paths.push('\n');
            ids.push(id);
        }
        let blobs = self.stdin_git(&["hash-object", "-w", "--stdin-paths"], &paths);
        let mut plan = String::new();
        for (id, blob) in ids.iter().zip(blobs.lines()) {
            plan.push_str(&format!("create refs/ank/claims/{id} {}\n", blob.trim()));
        }
        self.stdin_git(&["update-ref", "--stdin"], &plan);
        // The records are a fixture input, not corpus content: left in the tree
        // they would be untracked files under a corpus every drift comparison
        // reads.
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// `count` ratified decisions, half of them ratified by a commit carrying a
    /// signature header and half by a commit carrying none.
    ///
    /// **The commits are written as objects rather than committed**, and that
    /// is what makes the signed half possible at all: a header no keyring can
    /// produce on demand is one `hash-object -t commit` writes verbatim. It is
    /// the same act `git commit` performs — a tree, a parent, the message — and
    /// the branch is moved onto the chain at the end, over the tree it already
    /// had, so nothing in the worktree differs from what `main` carries.
    ///
    /// The two halves are the two answers `commit_carries_signature` exists to
    /// tell apart: `%G?` says `N` for both a commit nobody signed and a
    /// signature git could not attempt, and only the object says which. The
    /// forged block is deliberately not decodable — whether gpg is installed on
    /// the runner, and what it makes of the bytes, decides nothing here, because
    /// the statuses it can reach for them (`N` with the header there, `E`) are
    /// the same verdict.
    fn seed_ratifications(&self, count: usize) {
        // A file the decisions' scope matches, outside the one the seeded tasks
        // carry: a scope matching nothing is a dead scope, and a scope shared
        // with an open task charges its constraint to that task's budget. This
        // fixture means neither.
        std::fs::create_dir_all(self.0.join("doc")).unwrap();
        std::fs::write(self.0.join("doc/note.md"), "// seed\n").unwrap();
        // Declared, or every verdict is `None`: with no key in the file, §8's
        // advisory mode is what `signature_state` answers, and it answers it
        // before it ever asks about a commit.
        std::fs::write(
            self.0.join(".ank/allowed_signers"),
            "marie@example.org gpg 0123456789ABCDEF0123456789ABCDEF01234567\n",
        )
        .unwrap();
        for n in 0..count {
            let id = adr_id(n);
            std::fs::write(
                self.entity(&id),
                format!(
                    "---\nid: {id}\ntype: adr\nslug: example\ntitle: A decision\n\
                     created: 2026-07-20T00:00:00Z\nstatus: accepted\nratified: {anchor}\n\
                     scope:\n  - doc/**\nconstraint: |\n  Decision {n} binds doc.\n\
                     schema: 1\nversion: 1\n---\n\nWhy.\n",
                    anchor = anchor(n)
                ),
            )
            .unwrap();
        }
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", "decisions"]);

        // One tree for the whole chain, which is HEAD's: a ratification records
        // the anchor in its message, and these carry no write of their own.
        let tree = self.git(&["rev-parse", "HEAD^{tree}"]);
        let mut parent = self.git(&["rev-parse", "HEAD"]);
        for n in 0..count {
            let id = adr_id(n);
            let signed = n % 2 == 0;
            let header = if signed { FORGED_SIGNATURE } else { "" };
            let object = format!(
                "tree {tree}\nparent {parent}\n\
                 author Test <test@ank.local> 1753660800 +0000\n\
                 committer Test <test@ank.local> 1753660800 +0000\n{header}\n\
                 ratify {id}\n\nconstraint+scope: {anchor}\nby: human:marie\n",
                anchor = anchor(n)
            );
            parent = self
                .stdin_git(&["hash-object", "-t", "commit", "-w", "--stdin"], &object)
                .trim()
                .to_string();
        }
        self.git(&["update-ref", "refs/heads/main", &parent]);
    }

    /// A git command fed on standard input, with its standard output returned.
    fn stdin_git(&self, args: &[&str], input: &str) -> String {
        let mut child = spawn("git")
            .current_dir(&self.0)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    /// Every git process one invocation of the binary starts, in order, as the
    /// argument list git itself reports.
    ///
    /// **`GIT_TRACE` and not a shim on the `PATH`**, which is how the thirteen
    /// were first counted. A shim is a script, and a script is not what
    /// `Command::new("git")` finds on Windows — where this measurement matters
    /// most, process creation costing about 25ms there against 2-3ms on Linux.
    /// git's own trace writes one `trace: built-in:` line per process it starts,
    /// to a file, on all three platforms, and it is git saying it rather than us
    /// inferring it.
    fn git_processes(&self, args: &[&str]) -> (Vec<String>, String) {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let log = scratch::root().join(format!(
            "ank-status-trace-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let out = spawn(ANK)
            .args(args)
            .current_dir(&self.0)
            .env("ANK_AGENT", "reader@fixture")
            .env("GIT_TRACE", &log)
            .output()
            .expect("the binary under test must run");
        let code = out.status.code().expect("the process exits, not signals");
        assert!(
            code == 0 || code == 8,
            "ank {args:?} exited {code}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = std::fs::read_to_string(&log).unwrap_or_default();
        let _ = std::fs::remove_file(&log);
        let started: Vec<String> = text
            .lines()
            .filter_map(|l| l.split_once(MARK))
            .map(|(_, argv)| argv.trim().to_string())
            .collect();
        assert!(
            !started.is_empty(),
            "git wrote no trace to {}: the measurement did not happen",
            log.display()
        );
        (started, String::from_utf8_lossy(&out.stdout).to_string())
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

/// A well-formed identifier for the nth seeded decision.
fn adr_id(n: usize) -> String {
    format!("ADR-{:012x}", 0x2000_0000_0000u64 + n as u64)
}

/// The anchor the nth seeded decision claims to have been ratified with.
///
/// **Not the hash of its constraint**, and it cannot be: the normalisation and
/// the hash live in the binary, this suite declares no dependency on it, and
/// re-deriving either here would be a second implementation to keep true. So
/// every seeded decision reads as altered since ratification, which is a fault
/// this fixture carries deliberately. It changes nothing the measurement below
/// asks: the freeze and the signature are two independent readings of the same
/// commit, and it is the second one that is being counted.
fn anchor(n: usize) -> String {
    format!("{:012x}", 0xabc0_0000_0000u64 + n as u64)
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
    // rather than assumed. `ank status --json` spawned thirteen git processes
    // when this test was written — two `git --version`, two `rev-parse
    // --git-common-dir`, two `rev-parse --show-toplevel`, three `for-each-ref
    // refs/ank/`, and one each of `symbolic-ref HEAD`, `symbolic-ref
    // refs/remotes/origin/HEAD`, `rev-parse <branch>^{commit}` and `rev-list
    // --max-parents=0` — and process creation costs roughly 25ms on a Windows
    // runner against 2-3ms on Linux, which put about 325ms in front of the verb
    // before it read anything. TASK-5690eae1e008 took eleven of those thirteen
    // out; what is left is asserted below, by counting rather than by timing.
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

/// `cat-file -p <object name>`, which is a record read one process at a time.
///
/// Told apart from `cat-file -p <rev>:<path>`, which is how the index reads the
/// default branch and is a different question about a different thing.
fn reads_one_object(argv: &str) -> bool {
    argv.strip_prefix("cat-file -p ")
        .is_some_and(|name| name.len() == 40 && name.chars().all(|c| c.is_ascii_hexdigit()))
}

/// The plane is read once, and reading it costs what it costs for one ref
/// (TASK-5690eae1e008).
///
/// **Counted, not timed, and that is the point.** The claim is about processes:
/// three readers of `refs/ank/*` inside one invocation, each enumerating the
/// namespace and then asking `cat-file` per ref, and two questions — the
/// version, the repository — asked once per caller instead of once. A count is
/// the same number on Linux, macOS and Windows, where the floor of the same
/// commit on the same runner image measured 376ms and then 612ms; a wall there
/// reports the scheduler, and nobody can act on the scheduler.
#[test]
fn status_asks_git_no_question_twice_and_reads_the_plane_in_one_batch() {
    let many = Corpus::new();
    many.seed_claims(REFS);
    // Warm, which is the state this measures: the index exists, the verdict is
    // memoised against a plane that has stopped moving, and the root commit has
    // been read. A cold run pays for all three, once, and that cost belongs to
    // whatever verb ran first.
    many.json(&["status", "--json"]);
    many.json(&["status", "--json"]);
    let (started, _) = many.git_processes(&["status", "--json"]);
    let count = |p: &dyn Fn(&str) -> bool| started.iter().filter(|a| p(a)).count();

    assert_eq!(
        count(&|a| a == "version"),
        1,
        "the version is asked once: {started:#?}"
    );
    assert_eq!(
        count(&|a| a.starts_with("rev-parse --path-format=absolute")),
        1,
        "where the repository is, is asked once: {started:#?}"
    );
    assert_eq!(
        count(&|a| a.starts_with("for-each-ref") && a.contains("refs/ank/")),
        1,
        "the namespace is enumerated once: {started:#?}"
    );
    // One batch for the records, never one process per ref: `cat-file -p <sha>`
    // is the shape of the read this replaces, and `rev-parse --verify` of a
    // claim ref is the lookup that preceded it.
    assert_eq!(
        count(&reads_one_object),
        0,
        "no record is read object by object: {started:#?}"
    );
    assert_eq!(
        count(&|a| a.starts_with("rev-parse --verify --quiet refs/ank/")),
        0,
        "no claim ref is resolved one at a time: {started:#?}"
    );
    assert_eq!(
        count(&|a| a.starts_with("rev-list --max-parents=0")),
        0,
        "the root commit is not walked for a second time: {started:#?}"
    );

    // **Five hundred refs cost exactly what one costs.** The two invocations
    // are the same invocation over a plane three orders of magnitude apart, so
    // the sequences are compared whole rather than counted: a process that
    // appeared once per ref would show up as a longer list, and one that moved
    // would show up as a different list.
    let one = Corpus::new();
    one.seed_claims(1);
    one.json(&["status", "--json"]);
    one.json(&["status", "--json"]);
    let (single, _) = one.git_processes(&["status", "--json"]);
    assert_eq!(
        started, single,
        "{REFS} coordination refs cost more git than one does"
    );
}

/// A ratification costs no process of its own, and the verdict is the same
/// verdict (TASK-fc0334201ccf).
///
/// **Counted over two corpora, and never timed.** `check` and `review` each
/// asked git for one commit object per ratification — forty-seven of the
/// sixty-four processes they started on this repository, one `cat-file commit`
/// per decision that has been ratified, and forty-seven is not a constant. The
/// question is one bit per commit and `cat-file --batch` answers all of them at
/// once, which is what the coordination plane already does two call sites away.
///
/// The claim is invariance and not a ceiling: what is asserted is that the two
/// counts are *equal* over corpora [`FACTOR`] apart, never that either is some
/// particular number. A ceiling would be a number to revise at every change,
/// and a duration would be a measurement of the runner — process creation costs
/// about 25ms on a Windows runner against 2-3ms on Linux, and the floor of one
/// commit on one runner image has measured 376ms and then 612ms.
///
/// And the verdicts are asserted beside the count, on both corpora, because a
/// batch that read the wrong bytes would be perfectly invariant and perfectly
/// wrong: a commit carrying a `gpgsig` header nobody can check is `Unchecked`
/// and counted for the corpus, a commit carrying none is `Absent` and a fault
/// per entity, and the batch is the only thing that now tells them apart.
#[test]
fn every_ratification_signature_is_read_in_one_batch() {
    let many = Corpus::new();
    many.seed_ratifications(RATIFICATIONS);
    let few = Corpus::new();
    few.seed_ratifications(RATIFICATIONS / FACTOR);

    // Warm, and both the same way: a cold run builds the index and asks git for
    // the signatures it has no verdict for yet, and a comparison between a cold
    // corpus and a warm one would be a comparison of two different questions.
    for c in [&many, &few] {
        c.json(&["check", "--json"]);
        c.json(&["check", "--json"]);
        c.json(&["review", "--json"]);
    }

    for verb in ["check", "review"] {
        let (over_many, _) = many.git_processes(&[verb, "--json"]);
        let (over_few, _) = few.git_processes(&[verb, "--json"]);
        assert_eq!(
            over_many.len(),
            over_few.len(),
            "{RATIFICATIONS} ratifications cost {verb} more git than \
             {} do:\nmany {over_many:#?}\nfew {over_few:#?}",
            RATIFICATIONS / FACTOR
        );
        assert_eq!(
            over_many.iter().filter(|a| reads_one_commit(a)).count(),
            0,
            "a commit is still read on its own: {over_many:#?}"
        );
    }

    // The verdicts, through the binary and on both corpora, so that neither the
    // batch nor the fallback beside it can be reading bytes that say something
    // else.
    for (c, count) in [(&many, RATIFICATIONS), (&few, RATIFICATIONS / FACTOR)] {
        let doc = c.json(&["check", "--json"]);
        let unsigned = count / 2;
        assert_eq!(
            doc.matches("its ratification commit is not signed").count(),
            unsigned,
            "{unsigned} ratifications carry no signature header: {doc}"
        );
        for n in 0..count {
            let ratified_by_a_signature = n % 2 == 0;
            let named = doc.contains(&format!(
                "\"subject\":\"{}\",\"message\":\"its ratification commit is not signed",
                adr_id(n)
            ));
            assert_eq!(
                named,
                !ratified_by_a_signature,
                "{} is reported by the object its ratification is: {doc}",
                adr_id(n)
            );
        }
        // The other half, counted once for the corpus rather than once per
        // entity, which is what `check` does with a signature it cannot check.
        assert!(
            doc.contains(&format!(
                "{} ratification signature(s) could not be checked here",
                count - unsigned
            )),
            "the signatures that are there are counted and not refused: {doc}"
        );
    }
}

/// `cat-file commit <object name>`, which is one ratification read one process
/// at a time.
fn reads_one_commit(argv: &str) -> bool {
    argv.strip_prefix("cat-file commit ")
        .is_some_and(|name| name.len() == 40 && name.chars().all(|c| c.is_ascii_hexdigit()))
}

/// The corpus is keyed on its root commit, and the history is walked to find it
/// once per clone rather than once per invocation (TASK-5690eae1e008).
///
/// `rev-list --max-parents=0` has to traverse everything to know it has found
/// every root, so it is the one question here whose cost grows with the
/// repository. What removes it is not a smaller walk but no walk: the answer is
/// kept in the worktree's git directory, where the history that produced it
/// lives.
#[test]
fn the_root_commit_is_walked_once_and_then_read_from_where_it_was_kept() {
    let c = Corpus::new();
    // A history rather than a commit, so that a walk is something the answer
    // could have come from.
    for n in 0..20 {
        c.git(&[
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            &format!("commit {n}"),
        ]);
    }
    let root = c.git(&["rev-list", "--max-parents=0", "HEAD"]);

    let walk = |a: &str| a.starts_with("rev-list --max-parents=0");
    let (cold, first) = c.git_processes(&["status", "--json"]);
    assert_eq!(
        cold.iter().filter(|a| walk(a)).count(),
        1,
        "the history is read once: {cold:#?}"
    );
    let (warm, again) = c.git_processes(&["status", "--json"]);
    assert_eq!(
        warm.iter().filter(|a| walk(a)).count(),
        0,
        "and never again: {warm:#?}"
    );

    // The same answer both times, and the one git gives.
    let keyed = format!("\"corpus\":\"{root}\"");
    assert!(first.contains(&keyed), "{keyed} is in {first}");
    assert!(again.contains(&keyed), "{keyed} is in {again}");
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
