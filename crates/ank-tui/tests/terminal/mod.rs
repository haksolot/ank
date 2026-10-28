//! A terminal, a corpus and a driven `ank tui`, shared by the suites that need
//! all three.
//!
//! **Why it is here rather than in `src/`.** The suites are modules of one
//! integration test target, `tests/main.rs`, which declares this module once
//! and every suite reaches it as `crate::terminal` -- and a pseudo-terminal in
//! `ank-tui`'s own `src/` is not an option:
//! `tests/dependencies.rs` forbids this crate a foreign symbol and forbids it
//! `unsafe`, which is the whole of what ADR-ac6be1ebe9aa bought by taking
//! crossterm. A test may name what the crate may not, and that file's
//! `sources()` reads only `src/`, which is exactly the exemption it is written
//! to give. So the smallest terminal that can answer these questions lives in a
//! test module, and nothing above it is duplicated at all.
//!
//! **Why the binary is found rather than named.** `CARGO_BIN_EXE_ank` is
//! defined only for the package that declares the binary, and that is
//! `ank-cli`. So it is looked for beside the test executable, which is where
//! cargo puts it, and the assertion when it is missing names the command that
//! builds it rather than passing on nothing.
//!
//! `#[cfg(unix)]` for the reason `ank-cli`'s suite gives: a pseudo-terminal on
//! Windows is ConPTY, and reaching it means the console API this workspace does
//! not otherwise call. What runs on all three platforms is the layout, the
//! keystroke mapping and the render, in `src/view.rs`.

// Each suite uses the part of this module it needs, and what no suite reaches
// today is kept for the next one rather than deleted.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

// ---------------------------------------------------------------------------
// The binary, and a corpus for it to read
// ---------------------------------------------------------------------------

/// The `ank` this workspace just built.
///
/// Beside the test executable's own directory: cargo puts an integration test
/// in `<target>/<profile>/deps/` and a binary in `<target>/<profile>/`.
pub fn ank() -> PathBuf {
    let mut at = std::env::current_exe().expect("a test executable has a path");
    at.pop();
    if at.file_name().is_some_and(|n| n == "deps") {
        at.pop();
    }
    let binary = at.join("ank");
    assert!(
        binary.is_file(),
        "the ank binary is not at {}: this suite drives the process, so build it \
         first (cargo test --workspace, or cargo build -p ank-cli)",
        binary.display()
    );
    binary
}

/// A scratch repository nothing else uses, removed when the test ends.
pub struct Repo(pub PathBuf);

impl Drop for Repo {
    /// **Only ever under [`scratch::root`], and the check is not a formality.**
    /// The field is public, so any path at all can be put in one -- and a
    /// `Drop` that removed whatever it was handed took a working tree off this
    /// machine while this suite was being read (LOG, TASK-73b1eea6cecb). What
    /// this type is for is a corpus this suite made, so the removal is stated
    /// over that and a path from anywhere else is left exactly where it is.
    fn drop(&mut self) {
        if self.0.starts_with(scratch::root()) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

pub const AGENT: &str = "claude-code/opus-5+panel-suite";

/// Every git repository inside a fixture, found rather than listed.
///
/// A directory holding a `HEAD` file and an `objects` directory is one,
/// whether it is the `.git` beside a working tree or a bare corpus. Found,
/// because a list would have to be maintained, and what is being guarded
/// against is exactly a repository nobody remembered to enrol.
fn repositories_under(root: &std::path::Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if dir.join("HEAD").is_file() && dir.join("objects").is_dir() {
            found.push(dir);
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(e.path());
            }
        }
    }
    found
}

/// What git itself answers for one key of one repository, `None` when the key
/// is unset -- which is the state this asserts against, since an unset
/// `maintenance.auto` means maintenance is on.
fn config_of(git_dir: &std::path::Path, key: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("--git-dir")
        .arg(git_dir)
        .args(["config", "--get", key])
        .output()
        .expect("git must be installed: it is a hard dependency");
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// A fixture repository is not maintained under the test.
///
/// Read back out of a freshly built fixture rather than grepped out of this
/// file, and every repository under it is found rather than named: every
/// fixture in every suite that declares this module is built by
/// `Repo::seeded`, so a fifth one added later is held to this without anyone
/// remembering to enrol it. It runs once per suite because the module is
/// compiled into each of them, and each of them then answers for the corpus it
/// builds.
///
/// Measured on 2026-08-30 in run 33284185681: git repacked a fixture between
/// two fingerprints of it -- `objects/maintenance.lock`, a `tmp_pack` and six
/// loose objects in the first, a multi-pack-index, two packs and `info/refs`
/// in the second -- and a test asserting that a read writes nothing failed on
/// one platform of three. Ank had written nothing (TASK-fc6bef21e268).
#[test]
fn a_fixture_repository_is_not_maintained_under_the_test() {
    let repo = Repo::seeded();
    let repos = repositories_under(&repo.0);
    assert!(
        !repos.is_empty(),
        "no repository found under {}: this asserts nothing",
        repo.0.display()
    );
    for git_dir in repos {
        let at = git_dir.display();
        assert_eq!(
            config_of(&git_dir, "gc.auto").as_deref(),
            Some("0"),
            "gc.auto at {at}"
        );
        assert_eq!(
            config_of(&git_dir, "maintenance.auto").as_deref(),
            Some("false"),
            "maintenance.auto at {at}"
        );
    }
}

/// A reader holds its own terminal and no other (TASK-4952001e9596).
///
/// A second session is opened before the reader and kept open across the
/// spawn, which is the condition under which a master leaked at all: before
/// `pty::open` set close-on-exec, the reader inherited that master and its
/// own, and every `ank` it ran inherited both in turn. The count is taken off
/// `/proc/<pid>/fd` because that is the kernel's own list of what the child
/// holds, which is why this runs on Linux only; the fix it pins is the same
/// call on every Unix.
/// A reader whose terminal is gone ends by itself (TASK-69e774ef9d64).
///
/// The terminal is closed from the far side with the reader idle on it, which
/// is what a suite dying under a session does. Nothing is sent and no signal
/// arrives: a child of this suite controls no terminal, so no `SIGHUP` is
/// raised for it. Before the fix the reader read end of file in a loop for as
/// long as it was let, about 1.3 million times a second.
///
/// **The bound is a deadline and not a measurement.** What is asserted is that
/// the reader ends at all; thirty seconds is there so that a reader that never
/// does fails here, named, rather than hanging the suite, and it is killed
/// before the failure is reported so that it does not outlive the run.
#[test]
fn a_reader_whose_terminal_hangs_up_ends_by_itself() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    let mut child = abandoned(&repo, 120, 30);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().expect("the reader can be waited for") {
            // How it ended is not the point: a reader whose terminal is gone
            // has nowhere left to say so, and the teardown it tries may not
            // finish cleanly. That it ended is.
            let _ = status;
            return;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("the reader was still running 30 s after its terminal hung up");
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

/// A session dropped without being quit takes its reader with it
/// (TASK-69e774ef9d64).
///
/// The drop is the one a panicking test performs, so a test that fails between
/// opening a session and quitting it leaves nothing running. The pid is asked
/// after the drop with `kill -0`, which answers on Linux and macOS alike; and
/// where it is still there, it is killed before the failure is reported.
#[test]
fn a_session_dropped_without_quitting_leaves_no_reader() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    let live = Live::open(&repo, 120, 30);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    let pid = live.child.id().to_string();
    drop(live);
    let alive = Command::new("kill")
        .args(["-0", &pid])
        .stderr(std::process::Stdio::null())
        .status()
        .expect("kill runs")
        .success();
    if alive {
        let _ = Command::new("kill").args(["-9", &pid]).status();
    }
    assert!(
        !alive,
        "the reader {pid} outlived the session it was opened by"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn a_reader_holds_no_terminal_but_its_own() {
    let repo = Repo::seeded();
    let (_elsewhere, _its_path) = pty::open();
    let live = Live::open(&repo, 120, 30);
    live.until("the session to open", |t| t.contains("2 ENTITIES"));
    let held: Vec<String> = std::fs::read_dir(format!("/proc/{}/fd", live.child.id()))
        .expect("the child's descriptor table is listed under /proc")
        .map(|entry| {
            std::fs::read_link(entry.expect("an entry").path())
                .map(|target| target.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect();
    let own = held.iter().filter(|t| t.starts_with("/dev/pts/")).count();
    assert!(
        own >= 3,
        "the reader's own terminal is not on 0, 1 and 2, so this measures nothing: {held:?}"
    );
    let masters = held.iter().filter(|t| t.ends_with("/ptmx")).count();
    assert_eq!(
        masters, 0,
        "the reader inherited {masters} pseudo-terminal master(s): {held:?}"
    );
    live.quit();
}

/// Every terminal opened at once is told its own slave (TASK-adea229723df).
///
/// Several threads open sessions together, a handful each held at a time, and
/// once every thread holds its handful the slave each was told is asked again
/// for the same master, by one thread alone: `TIOCGPTN` on Linux, the kernel's
/// own number for the descriptor, and `ptsname_r` elsewhere. Read out of
/// `ptsname(3)`'s static buffer, two threads asking at once could both copy
/// the answer meant for one of them, and two sessions then shared a terminal:
/// one `q` written, one reader exits and the other waits forever, which is
/// the hang seen once in `dashboard.rs` on 2026-10-14. The barrier is what
/// makes it measurable: every round starts every thread on its `open` at the
/// same instant, which is the race the suites only ever ran into by luck.
///
/// **No thread can leave the others waiting.** A barrier that one thread
/// never reaches holds the rest for ever, and a panic is how a thread stops
/// reaching it: measured on Linux under `ulimit -n 60`, the first version of
/// this test, which panicked on a failed open, was still waiting after 60
/// seconds, and macos-latest held the job for 26 minutes. So an open that
/// fails is recorded rather than unwound, every thread reaches every barrier,
/// and one flag read after the barrier stops them all on the same round, on a
/// failure or on the deadline alike. The window held is sized off the system's
/// own ceiling, an eighth of it across every thread and never more than eight
/// each, since macOS hands out a few hundred terminals where Linux hands out
/// thousands; eight thousand opens per thread on Linux, where the race was
/// measured (red 9 runs of 10 against the `ptsname(3)` read, 1 to 7 crossed
/// per 64000), and a thousand elsewhere.
///
/// **A refused open is retried, and only one refused every time fails.** On
/// macos-latest, run 39069664222, one `posix_openpt` of 8000 failed with 7
/// masters held per thread and a ceiling of 511: a refusal that is not the
/// table full and not what this test is for, which is the path each master
/// is told. So a refusal is counted and printed with its errno, the open is
/// tried again after a short wait, and the test fails only on an open
/// refused `ATTEMPTS` times running.
#[test]
fn every_terminal_opened_at_once_is_told_its_own_slave() {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Barrier, Mutex, MutexGuard};
    use std::time::{Duration, Instant};

    const THREADS: usize = 8;
    #[cfg(target_os = "linux")]
    const PER_THREAD: usize = 8000;
    #[cfg(not(target_os = "linux"))]
    const PER_THREAD: usize = 1000;
    /// Tries per open before a refusal is a failure, waiting 2, 4, 8 and 16 ms
    /// between them.
    const ATTEMPTS: u32 = 5;
    let ceiling = pty::ceiling();
    let each = (ceiling.unwrap_or(256) / 8 / THREADS).clamp(1, 8);
    let rounds = PER_THREAD.div_ceil(each);
    let began = Instant::now();
    let deadline = began + Duration::from_secs(120);

    // A poisoned lock is taken anyway: nothing here may panic and strand a
    // thread short of a barrier.
    fn take<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
        lock.lock().unwrap_or_else(|held| held.into_inner())
    }
    let start = Barrier::new(THREADS);
    let checked = Barrier::new(THREADS);
    let stop = AtomicBool::new(false);
    let opened = AtomicUsize::new(0);
    let refused = Mutex::new(Vec::<String>::new());
    let failed = Mutex::new(Vec::<String>::new());
    let crossed = Mutex::new(Vec::<(PathBuf, PathBuf)>::new());
    let held = Mutex::new(Vec::<(std::fs::File, PathBuf)>::new());
    std::thread::scope(|scope| {
        for _ in 0..THREADS {
            let (start, checked, stop, opened) = (&start, &checked, &stop, &opened);
            let (refused, failed, crossed, held) = (&refused, &failed, &crossed, &held);
            scope.spawn(move || {
                for _ in 0..rounds {
                    start.wait();
                    // Set by the leader before it reached this barrier, so
                    // every thread reads the same answer on the same round.
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    let mut mine = Vec::with_capacity(each);
                    'open: for _ in 0..each {
                        for attempt in 1..=ATTEMPTS {
                            match pty::try_open() {
                                Ok(pair) => {
                                    mine.push(pair);
                                    continue 'open;
                                }
                                Err(error) if attempt < ATTEMPTS => {
                                    take(refused).push(error.to_string());
                                    std::thread::sleep(Duration::from_millis(1 << attempt));
                                }
                                Err(error) => {
                                    take(failed).push(error.to_string());
                                    break 'open;
                                }
                            }
                        }
                    }
                    opened.fetch_add(mine.len(), Ordering::Relaxed);
                    take(held).extend(mine);
                    // One thread re-asks while every master is held, so no
                    // number has been given back to the kernel and handed on.
                    if checked.wait().is_leader() {
                        for (master, told) in take(held).drain(..) {
                            match pty::asked_again(&master) {
                                Ok(own) if own != told => take(crossed).push((told, own)),
                                Ok(_) => {}
                                Err(error) => take(failed).push(error.to_string()),
                            }
                        }
                        if !take(failed).is_empty() || Instant::now() > deadline {
                            stop.store(true, Ordering::Release);
                        }
                    }
                }
            });
        }
    });
    // Printed whether or not the test passes: `.config/nextest.toml` shows
    // this test's output on success, so a refusal macOS recovered from is
    // still read off the job log.
    let refused = refused
        .into_inner()
        .unwrap_or_else(|held| held.into_inner());
    eprintln!(
        "{} open(s) refused and retried, the system's ceiling being {ceiling:?} and {each} \
         held per thread: {refused:?}",
        refused.len()
    );
    let failed = failed.into_inner().unwrap_or_else(|held| held.into_inner());
    assert!(
        failed.is_empty(),
        "{} of the opens were refused {ATTEMPTS} times running, the system's ceiling \
         being {ceiling:?} and {each} held per thread: {failed:?}",
        failed.len()
    );
    let opened = opened.into_inner();
    assert!(
        opened >= THREADS * PER_THREAD,
        "{opened} of {} opens done in {:?}, the deadline: this asserts too little",
        THREADS * PER_THREAD,
        began.elapsed()
    );
    let crossed = crossed
        .into_inner()
        .unwrap_or_else(|held| held.into_inner());
    assert!(
        crossed.is_empty(),
        "{} of {opened} terminals were told another's slave (told, own): {crossed:?}",
        crossed.len()
    );
}

/// The terminal every session but [`Live::dumb`] is opened on.
///
/// Stated rather than inherited: this suite runs under whatever the developer
/// has exported, and a reader that reads `TERM` -- which this one does, for
/// its palette and for its border set alike -- would otherwise draw one frame
/// on one machine and another on the next.
pub const TERM: &str = "xterm-256color";

/// What the listing says while it has nothing, which is what a screen drawn
/// before the corpus was read says (TASK-fff0a98511b2).
///
/// One constant because it is the barrier every suite here rests on: the
/// reader draws first and reads after, so "a panel is on the screen" stopped
/// meaning "the corpus was read" and this is what took its place.
pub const UNREAD: &str = "the corpus has not been read";

/// Deliberately wider than the entities panel at either window, so that a
/// frame which overflowed rather than fitted would be visible as a row past
/// the right edge.
const TASK_TITLE: &str =
    "A task whose title is wider than any panel this reader draws at forty columns";

/// One root per test process, swept by the next run (TASK-553740e7af11).
///
/// **A second copy of what `crates/ank-cli/tests/scratch/mod.rs` does**, and
/// deliberately: two crates cannot share a test helper without a dev-dependency
/// between them, and this crate's dependency tree is itself asserted by
/// `tests/dependencies.rs`. Adding a crate to be tidy here would have to be
/// declared there, which is a worse trade than thirty duplicated lines.
///
/// The reasoning is written out once, where the original lives. In short: a
/// `Drop` cannot run on `SIGKILL`, so the run that cleans up is the next one,
/// and a root's `.owner` lock is free exactly when its owner is gone.
pub mod scratch {
    use std::fs::{self, File};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::OnceLock;

    const PREFIX: &str = "ank-it-";
    const OWNER: &str = ".owner";
    static HELD: OnceLock<File> = OnceLock::new();

    pub fn dir(what: &str) -> PathBuf {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let p = root().join(format!("{what}-{}", SEQ.fetch_add(1, Ordering::Relaxed)));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).expect("the root must be writable");
        p
    }

    /// Public because the two places this suite spawns the binary hand it to
    /// the child as what "temporary" means (TASK-ec85b1561855).
    pub fn root() -> &'static Path {
        static ROOT: OnceLock<PathBuf> = OnceLock::new();
        ROOT.get_or_init(|| {
            let base = std::env::temp_dir();
            let mine = base.join(format!("{PREFIX}{}", std::process::id()));
            let _ = fs::remove_dir_all(&mine);
            fs::create_dir_all(&mine).expect("the temporary directory must be writable");
            let lock = File::create(mine.join(OWNER)).expect("the root takes its own lock");
            lock.try_lock()
                .expect("a fresh root cannot already be held");
            let _ = HELD.set(lock);
            sweep(&base, &mine);
            mine
        })
        .as_path()
    }

    fn sweep(base: &Path, mine: &Path) {
        let Ok(entries) = fs::read_dir(base) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let named = p
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(PREFIX));
            if p == mine || !named || !p.is_dir() {
                continue;
            }
            let Ok(file) = File::options().read(true).write(true).open(p.join(OWNER)) else {
                continue;
            };
            if file.try_lock().is_ok() {
                let _ = file.unlock();
                drop(file);
                let _ = fs::remove_dir_all(&p);
            }
        }
    }
}

/// Tell a child of this suite that "temporary" means this run's own root
/// (TASK-ec85b1561855).
///
/// The same one line `crates/ank-cli/tests/cli.rs::spawn` grew for the same
/// reason, on the same scheme (TASK-553740e7af11): a child that inherits the
/// machine's temporary directory writes beside every other run on it, and
/// nothing collects what it leaves. Measured on 2026-08-29, `cargo test
/// --workspace` into an empty temporary directory left one file that no run
/// sweeps -- the scratch file `ank new task` keeps when the fake `$EDITOR`
/// this suite spawns hands back something invalid.
///
/// **What is not being fixed is that the file is written.** `editor::kept`
/// answers a typo without discarding the twenty minutes around it, and the
/// refusal names the path: a file swept between the caller reading that
/// message and opening it would be the promise broken. It is still written,
/// still kept, still named. Only the directory the child calls temporary
/// moves, into the root the next run already collects.
///
/// All three names, because `std::env::temp_dir` reads `TMP` and `TEMP` on
/// Windows and `TMPDIR` everywhere else: setting one of the three would leave
/// the child on the runner's own temporary directory on the platform where
/// this suite runs last.
fn in_this_runs_root(command: &mut Command) {
    let root = scratch::root();
    command
        .env("TMPDIR", root)
        .env("TMP", root)
        .env("TEMP", root);
}

/// Waits until the wall clock has moved to a later second than the one it
/// is in now, so that what is written next carries a `created` later than
/// everything written before the call.
fn next_second() {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is past the epoch");
    let into = Duration::from_nanos(u64::from(now.subsec_nanos()));
    std::thread::sleep(Duration::from_secs(1) - into + Duration::from_millis(5));
}

/// The corpus [`Repo::graphed`] seeds, with every identifier it wrote
/// (TASK-643611645dea).
///
/// The identifiers are kept rather than found again, because what the suite
/// asserts is *which* row stands above which, and a test that looked the
/// tasks up by title would be reading the frame to learn what the frame is
/// supposed to show.
pub struct Graphed {
    pub repo: Repo,
    /// Three tasks, each blocked by the one before it: the first is a root.
    pub chain: [String; 3],
    /// A top, a left and a right both blocked by the top, and a bottom
    /// blocked by both.
    pub diamond: [String; 4],
    /// A task closed with a dependent still open.
    pub closed: String,
    /// The dependent.
    pub dependent: String,
    /// A task with no edge at all.
    pub lone: String,
    /// A task under way (TASK-9ea4dc65755b): claimed through the CLI, so
    /// `graph --json` lists it as `in_progress`, and blocked by the first of
    /// the chain, which gives that root two dependents.
    pub under_way: String,
}

impl Graphed {
    /// Every task the fixture wrote that is not finished, the one under way
    /// among them: the rows the graph shows by default, less the closed
    /// blocker it shows beside them.
    pub fn open(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self.chain.iter().map(String::as_str).collect();
        out.extend(self.diamond.iter().map(String::as_str));
        out.push(&self.dependent);
        out.push(&self.lone);
        out.push(&self.under_way);
        out
    }
}

impl Repo {
    /// A repository with a corpus and nothing in it: what every fixture here
    /// starts from.
    fn bare(what: &str) -> Repo {
        let root = scratch::dir(what);
        let repo = Repo(root);
        repo.git(&["init", "--initial-branch=main"]);
        repo.git(&["config", "user.email", "suite@example.invalid"]);
        repo.git(&["config", "user.name", "The Suite"]);
        // Signing off: a throwaway corpus inherits whatever the machine has
        // globally otherwise, and a suite whose commits are signed on one
        // machine and not the next is a suite that reports the machine.
        repo.git(&["config", "commit.gpgsign", "false"]);
        // Maintenance off. A repository git repacks behind the suite is a
        // corpus that moves under whatever reads it, and a reader driven
        // through a pseudo-terminal is slow enough to give it the window
        // (TASK-fc6bef21e268).
        repo.git(&["config", "gc.auto", "0"]);
        repo.git(&["config", "maintenance.auto", "false"]);
        std::fs::create_dir_all(repo.0.join("src")).unwrap();
        std::fs::write(repo.0.join("src/lib.rs"), "// code\n").unwrap();
        repo.ank(&["init"]);
        repo
    }

    /// One task written through the CLI, blocked by whatever `blocked_by`
    /// names, and the identifier it was given.
    pub fn new_task(&self, title: &str, blocked_by: &[&str]) -> String {
        let mut args = vec![
            "new",
            "task",
            "--title",
            title,
            "--scope",
            "src/**",
            "--criteria",
            "The graph draws this row where its edges put it.",
        ];
        for id in blocked_by {
            args.push("--blocked-by");
            args.push(id);
        }
        args.push("--json");
        let made = self.stdout(&args);
        let ids = ids_of(&made);
        assert_eq!(ids.len(), 1, "one task was written: {made}");
        ids[0].clone()
    }

    /// The corpus the graph is measured on (TASK-643611645dea): a chain of
    /// three, a diamond, one task closed with a dependent still open, and one
    /// task with no edge, and (TASK-9ea4dc65755b) one under way, blocked by the
    /// chain's first -- eleven tasks, nine of them open, one in progress and
    /// one closed, every one of them written through the CLI.
    ///
    /// The closed task is closed after its dependent names it, so the edge
    /// is one `new` accepted rather than one `amend` added, and the blocker
    /// is finished the way a blocker is finished in a real corpus: later.
    ///
    /// The task under way is written on a later second than everything
    /// before it (TASK-de02dbcf9965): the graph orders two blocks of one
    /// size by the `created` of their newest task, and `created` is written
    /// to the second, so ten tasks written inside one second would leave
    /// the chain's block and the diamond's tied and the order the hash's.
    pub fn graphed() -> Graphed {
        let repo = Repo::bare("graphed");
        let chain_1 = repo.new_task("The first of a chain", &[]);
        let chain_2 = repo.new_task("The second of a chain", &[&chain_1]);
        let chain_3 = repo.new_task("The third of a chain", &[&chain_2]);
        let top = repo.new_task("The top of a diamond", &[]);
        let left = repo.new_task("The left of a diamond", &[&top]);
        let right = repo.new_task("The right of a diamond", &[&top]);
        let bottom = repo.new_task("The bottom of a diamond", &[&left, &right]);
        let closed = repo.new_task("A blocker that was closed", &[]);
        let dependent = repo.new_task("A dependent of the closed one", &[&closed]);
        let lone = repo.new_task("A task with no edge", &[]);
        // Written with no blocker, claimed, and only then given its edge:
        // `claim` refuses a task whose blocker is not finished (exit 7), and
        // `amend` accepts an edge on a task already under way.
        next_second();
        let under_way = repo.new_task("A task under way", &[]);
        repo.ank(&["claim", &under_way]);
        repo.ank(&["amend", &under_way, "--blocked-by", &chain_1]);
        repo.ank(&[
            "close",
            &closed,
            "--reason",
            "closed so that its dependent is drawn ready",
        ]);
        Graphed {
            repo,
            chain: [chain_1, chain_2, chain_3],
            diamond: [top, left, right, bottom],
            closed,
            dependent,
            lone,
            under_way,
        }
    }

    pub fn seeded() -> Repo {
        let repo = Repo::bare("panels");
        repo.ank(&[
            "new",
            "adr",
            "--title",
            "Every byte shown is a byte the CLI printed",
            "--scope",
            "src/**",
            "--constraint",
            "The reader reaches the corpus by running the CLI.",
        ]);
        repo.ank(&[
            "new",
            "task",
            "--title",
            TASK_TITLE,
            "--scope",
            "src/**",
            "--criteria",
            "The frame names this entity and the body arrives whole.",
        ]);
        repo
    }

    /// The seeded corpus, plus `extra` tasks, for the suites that are about
    /// what a corpus costs rather than what it says (TASK-fff0a98511b2).
    ///
    /// **Stamped rather than asked for, and the reason is arithmetic.** Every
    /// other fixture here is built by running `ank new`, which is the honest
    /// way and is what keeps the two entities of [`Repo::seeded`] provably
    /// well-formed. A thousand of them is a thousand process spawns against a
    /// corpus that grows under each one, which is minutes -- so this takes the
    /// one entity `new` wrote and copies it, changing the three fields that
    /// have to differ. What is being fixed is the *size* of a corpus, and a
    /// size is the one property a copy preserves exactly.
    ///
    /// Writing under `.ank/` directly is a liberty this module already takes
    /// ([`Repo::corpus`] reads every file there) and one no source of the crate
    /// has: the reader reaches the corpus by running the CLI, and a test may
    /// name what the crate may not.
    pub fn crowded(extra: usize) -> Repo {
        let repo = Repo::seeded();
        let entities = repo.0.join(".ank").join("entities");
        let seed = std::fs::read_dir(&entities)
            .expect("the corpus has an entities directory")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .find(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("TASK-"))
            })
            .expect("the seeded corpus carries a task");
        let template = std::fs::read_to_string(&seed).expect("the task file is readable");
        let was = template
            .split_once("id: ")
            .and_then(|(_, rest)| rest.split_once('\n'))
            .map(|(id, _)| id.to_string())
            .expect("an entity file names its own id");
        for i in 0..extra {
            // A twelve-hex identifier in a range `new` does not draw from, so a
            // stamped entity can never collide with a seeded one.
            let id = format!("TASK-{:012x}", 0x7000_0000_0000u64 + i as u64);
            let body = template
                .replace(&was, &id)
                .replace("slug: ", &format!("slug: crowd-{i}-"))
                .replace("title: ", &format!("title: {i} "));
            std::fs::write(entities.join(format!("{id}.md")), body)
                .expect("a stamped entity is writable");
        }
        repo
    }

    /// The one read this reader makes on opening, made once beforehand.
    ///
    /// [`Repo::warm`] cannot be used on a crowded corpus: it asks `show` once
    /// per entity, which is the thing this fixture has a thousand of. What has
    /// to be warm is `.ank/index.db`, and one `find` writes it.
    pub fn warm_find(&self) {
        let _ = self.stdout(&["find", "--json"]);
    }

    pub fn git(&self, args: &[&str]) -> Output {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("git must be on PATH for this suite");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }

    /// The short form of the task this corpus carries, which is what a listing
    /// prints and what the prompt takes.
    pub fn task(&self) -> String {
        short_of(&self.only(&["--type", "task"]))
    }

    /// The standard output of one call, as text.
    pub fn stdout(&self, args: &[&str]) -> String {
        String::from_utf8_lossy(&self.ank(args).stdout).to_string()
    }

    /// The one full identifier a `find` filter leaves, read out of the `--json`
    /// document rather than off the human page.
    pub fn only(&self, filter: &[&str]) -> String {
        let mut args = vec!["find"];
        args.extend_from_slice(filter);
        args.push("--json");
        let doc = self.stdout(&args);
        let ids = ids_of(&doc);
        assert_eq!(ids.len(), 1, "{filter:?} left {ids:?} in\n{doc}");
        ids[0].clone()
    }

    /// Every read the reader makes, made once beforehand.
    ///
    /// `.ank/index.db` is the CLI's own cache and it is written the first time
    /// a corpus is searched. Warming it before a snapshot is what separates
    /// "the session wrote something" from "the first read built a cache".
    pub fn warm(&self) {
        let _ = self.stdout(&["find", "--json"]);
        let _ = self.stdout(&["status", "--json"]);
        let _ = self.stdout(&["scope", "src/**", "--json"]);
        for id in ids_of(&self.stdout(&["find", "--json"])) {
            let _ = self.stdout(&["show", &id, "--json"]);
        }
    }

    /// Every file under `.ank/`, path and bytes, as one comparable value.
    ///
    /// Bytes and not modification times: "byte for byte unchanged" is a claim
    /// about content, and a corpus whose files were rewritten identically is a
    /// corpus that did not move.
    pub fn corpus(&self) -> Vec<(String, Vec<u8>)> {
        let root = self.0.join(".ank");
        let mut out = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("the corpus directory must be readable") {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    let name = path
                        .strip_prefix(&root)
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    out.push((name, std::fs::read(&path).unwrap()));
                }
            }
        }
        out.sort();
        out
    }

    /// Every `refs/ank/*` this repository carries, by name and by what it
    /// points at.
    ///
    /// The target and not only the name: a claim renewed in place keeps its
    /// name and moves its object, and a comparison on names alone would call
    /// that no change at all.
    pub fn refs(&self) -> String {
        String::from_utf8_lossy(
            &self
                .git(&[
                    "for-each-ref",
                    "--format=%(refname) %(objectname)",
                    "refs/ank/",
                ])
                .stdout,
        )
        .to_string()
    }

    pub fn ank(&self, args: &[&str]) -> Output {
        let out = self.tried(args, &[]);
        assert!(
            out.status.success(),
            "ank {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }

    /// One call, with whatever else the caller wants in the environment, and no
    /// assertion about how it went.
    ///
    /// Separate from [`Repo::ank`] because the suites that need it are asking
    /// about a refusal: `ank new adr --title x --scope y` is *supposed* to exit
    /// non-zero, and that is the fact being measured rather than a failure to
    /// report (TASK-d832452630d2).
    pub fn tried(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut command = Command::new(ank());
        command
            .args(args)
            .current_dir(&self.0)
            .env("ANK_AGENT", AGENT)
            .env("NO_COLOR", "1");
        in_this_runs_root(&mut command);
        for (name, value) in env {
            command.env(name, value);
        }
        command.output().expect("the binary must have been built")
    }
}

// ---------------------------------------------------------------------------
// The editor, and the sentinel that says it ran
// ---------------------------------------------------------------------------

/// An `$EDITOR` that writes a file and exits (TASK-d832452630d2).
///
/// It writes and does not edit, deliberately: what is being measured is whether
/// the editor was *reached*, and a script that filled the template in would
/// make the reader's failure look like a success.
///
/// Here rather than in one suite because two now need it (TASK-e8da6a00564a).
/// `ank new` opens an editor when every mandatory flag is absent and `ank edit`
/// opens one when no field is named at all, and both criteria are measured the
/// same way: point `$EDITOR` at this, and find the sentinel absent.
pub struct Editor {
    pub script: std::path::PathBuf,
    sentinel: std::path::PathBuf,
}

impl Editor {
    pub fn beside(repo: &Repo) -> Editor {
        use std::os::unix::fs::PermissionsExt;
        let script = repo.0.join("editor.sh");
        let sentinel = repo.0.join("the-editor-ran");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\nprintf 'the editor ran on %s\\n' \"$1\" > {}\n",
                sentinel.display()
            ),
        )
        .expect("the script must be writable");
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
            .expect("the script must be executable");
        Editor { script, sentinel }
    }

    pub fn env(&self) -> Vec<(&str, &str)> {
        vec![("EDITOR", self.script.to_str().expect("a UTF-8 path"))]
    }

    pub fn ran(&self) -> bool {
        self.sentinel.is_file()
    }

    pub fn forget(&self) {
        let _ = std::fs::remove_file(&self.sentinel);
    }

    /// The assertion these suites turn on, made wherever it is worth making:
    /// the reader never reached an editor.
    pub fn never_ran(&self, when: &str) {
        assert!(
            !self.ran(),
            "the reader opened $EDITOR {when}, into a child with no stdin and a \
             terminal in raw mode on the alternate screen: {}",
            self.sentinel.display()
        );
    }
}

/// Every `"id":"..."` of a document, in the order it carries them.
pub fn ids_of(doc: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = doc;
    while let Some(at) = rest.find("\"id\":\"") {
        rest = &rest[at + 6..];
        let end = rest.find('"').expect("an id is a closed string");
        out.push(rest[..end].to_string());
        rest = &rest[end..];
    }
    out
}

/// The short form every listing prints: the kind, then four characters.
pub fn short_of(id: &str) -> String {
    let (kind, rest) = id.split_once('-').expect("an identifier has a kind");
    format!("{kind}-{}", &rest[..4])
}

/// The title of the document open on one entity, as a frame carries it
/// (TASK-a9cd362ee5e2): `BODY` with no digit in front, then the short
/// identifier three spaces on.
///
/// Asked as one string rather than as `BODY` and the identifier apart: the
/// identifier is on the listing too, so a frame still showing the listing
/// would carry both halves and answer for a document that never opened.
pub fn body_naming(short: &str) -> String {
    format!("{BODY}{short}")
}

/// The document's title, up to where its identifier starts.
///
/// `BODY` alone is a word a document's own prose may carry; the three spaces
/// after it are the title's.
pub const BODY: &str = "BODY   ";

/// A screen of the ring's title on a wide frame while that panel has the
/// focus: the focus marker, then the digit and the name (ADR-ac6be1ebe9aa).
pub fn focused(title: &str) -> String {
    format!("{}{title}", ank_tui::view::BOXES.focus())
}

/// One pseudo-terminal suite at a time, across every suite that takes it.
///
/// **Kept until macOS is measured without it, and no longer because of
/// `ptsname(3)`.** The lock was first taken because `pty::open` read
/// the slave's path out of `ptsname(3)`'s static buffer, so two sessions opened
/// at once could be told one terminal (LOG-3b0bc419c884). It reads it with
/// `ptsname_r` now, and `every_terminal_opened_at_once_is_told_its_own_slave`
/// pins that (TASK-adea229723df). With that cause gone the lock and the
/// `pty-serial` group of `.config/nextest.toml` were removed, and on Linux,
/// over 10 runs of the 125 tests under each runner, the only failures were in
/// suites that never took the lock. On macos-latest the
/// `cargo nextest run --workspace --profile ci` step then never finished: run
/// 39060806985 (head 21c9e74) sat 46 minutes until a push superseded it, run
/// 39064235334 (head 8c1954e) 30 minutes until it was cancelled, while
/// ubuntu-latest and windows-latest passed both. So both came back, on every
/// platform: the hang is not diagnosed, and ten Linux runs do not show Linux
/// needs neither. Those two heads also carried
/// `every_terminal_opened_at_once_is_told_its_own_slave`, and a third, 4dda9b9,
/// hung the same way with the lock and the group back: the hang was that
/// test's, which left its threads at a barrier when one open failed. So the
/// removal was never measured on macOS, and the lock stays until it is.
///
/// One lock and not one per suite: the suites are modules of one test binary,
/// so under `cargo test` they share one process and a lock per suite would
/// let two of them open a session at once. Under nextest every test is its own
/// process and this lock serialises nothing, so `.config/nextest.toml` names
/// the same tests in its `pty-serial` group.
pub static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

// ---------------------------------------------------------------------------
// The terminal
// ---------------------------------------------------------------------------

/// Every byte one CLI verb wrote to a terminal it painted for
/// (ADR-f0665c68d03d).
///
/// Not a [`Live`] session: a verb like `find` prints and exits, so there is no
/// screen to settle and no key to send. stdout alone is the pseudo-terminal,
/// which is the condition the CLI paints on, and `NO_COLOR` is removed for the
/// reason [`Live::painting`] removes it. The run's root is the one every other
/// child here gets, so the verb reads the corpus it was pointed at.
pub fn printed_at_a_terminal(repo: &Repo, args: &[&str]) -> Vec<u8> {
    use std::io::Read;

    let (mut master, slave_path) = pty::open();
    pty::resize(&slave_path, 200, 50);
    let mut command = Command::new(ank());
    command
        .args(args)
        .current_dir(&repo.0)
        .env("ANK_AGENT", AGENT)
        .env("TERM", TERM)
        .env_remove("NO_COLOR");
    in_this_runs_root(&mut command);
    let mut child = command
        .stdin(std::process::Stdio::null())
        .stdout(pty::stdio(pty::slave(&slave_path)))
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("the binary must have been built");
    // The parent's copy of the slave goes with the command, so the master
    // reads end-of-stream once the child is done with its own.
    drop(command);
    let drain = std::thread::spawn(move || {
        let mut raw = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            match master.read(&mut buf) {
                // Linux answers EIO once the last slave is closed; macOS
                // answers zero. Both mean the verb is done.
                Ok(0) | Err(_) => return raw,
                Ok(n) => raw.extend_from_slice(&buf[..n]),
            }
        }
    });
    let status = child.wait().expect("the verb must end");
    assert!(status.success(), "{args:?} failed at a terminal: {status}");
    drain.join().expect("the drain must not panic")
}

mod pty {
    use std::ffi::CStr;
    use std::fs::File;
    use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};
    use std::os::raw::{c_char, c_int, c_ulong};
    use std::path::{Path, PathBuf};

    extern "C" {
        fn posix_openpt(flags: c_int) -> c_int;
        fn grantpt(fd: c_int) -> c_int;
        fn unlockpt(fd: c_int) -> c_int;
        fn ptsname_r(fd: c_int, buffer: *mut c_char, length: usize) -> c_int;
        fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
        fn fcntl(fd: c_int, command: c_int, ...) -> c_int;
    }

    #[cfg(target_os = "macos")]
    extern "C" {
        fn sysctlbyname(
            name: *const c_char,
            value: *mut std::ffi::c_void,
            length: *mut usize,
            new_value: *mut std::ffi::c_void,
            new_length: usize,
        ) -> c_int;
    }

    const O_RDWR: c_int = 2;
    /// `fcntl`'s set-descriptor-flags command and the one flag it has, the
    /// same two numbers on Linux and macOS.
    const F_SETFD: c_int = 2;
    const FD_CLOEXEC: c_int = 1;

    /// Not POSIX, and spelled per platform because it is per platform: an
    /// `ioctl` number encodes the direction and the size of its argument, and
    /// the two kernels encode them differently.
    #[cfg(target_os = "linux")]
    const TIOCSWINSZ: c_ulong = 0x5414;
    #[cfg(not(target_os = "linux"))]
    const TIOCSWINSZ: c_ulong = 0x8008_7467;
    /// The slave's number, Linux's own ioctl and the one `ptsname_r` is built
    /// on there: `_IOR('T', 0x30, unsigned int)`.
    #[cfg(target_os = "linux")]
    const TIOCGPTN: c_ulong = 0x8004_5430;

    #[repr(C)]
    struct WinSize {
        rows: u16,
        columns: u16,
        x_pixels: u16,
        y_pixels: u16,
    }

    /// The master side as a `File`, closed on exec, and the path of the
    /// slave to hand the child.
    ///
    /// **Close-on-exec is set here because `posix_openpt` does not set it**
    /// (TASK-4952001e9596). Every descriptor this harness otherwise hands a
    /// child is `O_CLOEXEC` already -- `OpenOptions` opens the slaves that
    /// way, `File::try_clone` duplicates the drain's copy that way, and the
    /// three the child is given are moved onto 0, 1 and 2 by `Command` --
    /// so the master was the one descriptor that crossed an exec. It crossed
    /// into every reader spawned while the session was open, and from there
    /// into every `ank` the reader ran: measured on 2026-10-17, two to five
    /// `/dev/ptmx` links per reader on the dashboard suite alone. None of
    /// them was the hang seen once on 2026-10-14 (the log of the task has
    /// the measurement that was), but a child holding another session's
    /// terminal keeps it allocated after that session ended, and a
    /// descriptor nobody meant to hand over is not one to keep handing over.
    ///
    /// `fcntl(F_SETFD)` rather than an `O_CLOEXEC` bit in the open flags:
    /// the flag is documented for `posix_openpt` on Linux and merely passed
    /// through on macOS, where the two numbers below are the same as here.
    /// Set before anything else is done with the descriptor, so there is no
    /// spawn it could miss.
    pub fn open() -> (File, PathBuf) {
        try_open().unwrap_or_else(|failed| {
            panic!(
                "a pseudo-terminal could not be opened: {failed} (EAGAIN or ENOSPC from \
                 posix_openpt is the system's table of them full)"
            )
        })
    }

    /// [`open`], answering the error rather than panicking on it.
    ///
    /// **Nothing here retries.** A full table answers `EAGAIN` or `ENOSPC`
    /// and that answer is handed back at once: a caller that has to stay in
    /// step with other threads, as
    /// `every_terminal_opened_at_once_is_told_its_own_slave` does, can then
    /// say so and stop, where a panic in one of its threads would leave the
    /// others waiting on a barrier for ever.
    pub fn try_open() -> std::io::Result<(File, PathBuf)> {
        // SAFETY: the four calls are the POSIX pseudo-terminal sequence, in
        // order, and every return is checked before the next is made. The
        // descriptor is owned by `master` from the moment it exists, so an
        // early return closes it. The slave's name is read into a buffer of
        // this call's, see [`named`].
        unsafe {
            let fd = posix_openpt(O_RDWR);
            if fd < 0 {
                return Err(refusal("posix_openpt"));
            }
            let master = File::from_raw_fd(fd);
            if fcntl(fd, F_SETFD, FD_CLOEXEC) != 0 {
                return Err(refusal("fcntl(F_SETFD, FD_CLOEXEC)"));
            }
            if grantpt(fd) != 0 {
                return Err(refusal("grantpt"));
            }
            if unlockpt(fd) != 0 {
                return Err(refusal("unlockpt"));
            }
            let path = named(fd)?;
            Ok((master, path))
        }
    }

    /// The error `call` just left in `errno`, with the call named and the raw
    /// number spelled out beside the platform's text for it: on macos-latest
    /// one refusal printed as "Unknown error", which names nothing.
    fn refusal(call: &str) -> std::io::Error {
        let error = std::io::Error::last_os_error();
        let code = error.raw_os_error().unwrap_or_default();
        std::io::Error::new(error.kind(), format!("{call}: errno {code}, {error}"))
    }

    /// The slave's path, read with `ptsname_r` into a buffer this frame owns.
    ///
    /// **Never `ptsname(3)`: it answers out of one static buffer per
    /// process**, and two threads opening a terminal at once could both copy
    /// the path meant for one of them, so two sessions attached to one slave
    /// (TASK-adea229723df: 8 threads, 32000 opens, one to six crossed per run
    /// before this). `ptsname_r` is in glibc and in macOS since 10.13.4; it
    /// answers zero on success on both, an error number on one and `-1` on
    /// the other, which is why only zero is read.
    fn named(master: c_int) -> std::io::Result<PathBuf> {
        let mut buffer = [0 as c_char; 128];
        // SAFETY: the buffer is this frame's and its length is the one given.
        if unsafe { ptsname_r(master, buffer.as_mut_ptr(), buffer.len()) } != 0 {
            return Err(refusal("ptsname_r"));
        }
        // SAFETY: on success `ptsname_r` wrote a terminated string into it.
        let name = unsafe { CStr::from_ptr(buffer.as_ptr()) };
        Ok(PathBuf::from(
            name.to_str().expect("a device path is UTF-8"),
        ))
    }

    /// The slave of `master`, asked again by a road that owns its answer:
    /// the kernel's number for the descriptor on Linux, `ptsname_r` into a
    /// buffer of this frame's elsewhere. The instrument of
    /// `every_terminal_opened_at_once_is_told_its_own_slave`.
    #[cfg(target_os = "linux")]
    pub fn asked_again(master: &File) -> std::io::Result<PathBuf> {
        let mut number: u32 = 0;
        // SAFETY: the descriptor is open and owned by `master`, and the third
        // argument points at an `unsigned int`, which is what TIOCGPTN writes.
        if unsafe { ioctl(master.as_raw_fd(), TIOCGPTN, &mut number) } != 0 {
            return Err(refusal("TIOCGPTN"));
        }
        Ok(PathBuf::from(format!("/dev/pts/{number}")))
    }

    #[cfg(not(target_os = "linux"))]
    pub fn asked_again(master: &File) -> std::io::Result<PathBuf> {
        named(master.as_raw_fd())
    }

    /// How many pseudo-terminals the system hands out at once, where it says:
    /// `/proc/sys/kernel/pty/max` on Linux (4096 by default) and
    /// `kern.tty.ptmx_max` on macOS (511 on recent systems, a few hundred on
    /// older ones). `None` where neither is read.
    pub fn ceiling() -> Option<usize> {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/sys/kernel/pty/max")
                .ok()?
                .trim()
                .parse()
                .ok()
        }
        #[cfg(target_os = "macos")]
        {
            let mut value: c_int = 0;
            let mut length = std::mem::size_of::<c_int>();
            // SAFETY: the name is a terminated string, and `value` and
            // `length` are this frame's, sized for the `int` the key holds.
            let asked = unsafe {
                sysctlbyname(
                    c"kern.tty.ptmx_max".as_ptr(),
                    (&mut value as *mut c_int).cast(),
                    &mut length,
                    std::ptr::null_mut(),
                    0,
                )
            };
            (asked == 0 && value > 0).then_some(value as usize)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            None
        }
    }

    /// The window, stated on the slave side.
    ///
    /// On the slave and never on the master: macOS answers `-1` to a general
    /// tty ioctl on the cloning device, and a window size is a property of the
    /// terminal the program is looking at anyway.
    pub fn resize(slave_path: &Path, columns: u16, rows: u16) {
        let tty = slave(slave_path);
        let size = WinSize {
            rows,
            columns,
            x_pixels: 0,
            y_pixels: 0,
        };
        // SAFETY: the descriptor is open and owned by `tty`, and the third
        // argument is a pointer to a `winsize`, which is what TIOCSWINSZ reads.
        let set = unsafe { ioctl(tty.as_raw_fd(), TIOCSWINSZ, &size) };
        assert_eq!(
            set,
            0,
            "the window could not be set: {}",
            std::io::Error::last_os_error()
        );
    }

    pub fn slave(path: &Path) -> File {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .expect("the slave side of a pseudo-terminal must open")
    }

    pub fn stdio(file: File) -> std::process::Stdio {
        // SAFETY: the descriptor is owned by `file`, which gives it up here, so
        // exactly one owner reaches the child.
        unsafe { std::process::Stdio::from_raw_fd(file.into_raw_fd()) }
    }
}

/// A terminal, as far as this suite needs one.
///
/// The reader draws by diffing: ratatui writes the cells that changed and moves
/// the cursor over the ones that did not, so `2 ENTITIES` reaches the wire as
/// `2 ENTIT`, a cursor move and `IES`. A substring search over the bytes would
/// be asserting something untrue of them, so they are applied to a grid and
/// every assertion here is made against what a person would be looking at.
///
/// The smallest emulator that is honest about this stream: cursor position, the
/// two erases, and text. Everything else a CSI can say -- colour, attributes,
/// the alternate buffer, the cursor's visibility -- moves no character, so it
/// is consumed and dropped rather than half understood.
struct Screen {
    grid: Vec<Vec<char>>,
    columns: usize,
    rows: usize,
    x: usize,
    y: usize,
    /// Bytes of a sequence whose end has not arrived yet: a terminal hands over
    /// a frame in whatever pieces it likes, and half a `MoveTo` is not a
    /// character.
    pending: Vec<u8>,
    /// Everything the session wrote, kept alongside the grid.
    ///
    /// The grid is what a person would have been looking at and it is what
    /// almost every assertion is about. `NO_COLOR` is the exception and it is
    /// exactly the other question: whether an escape sequence was on the wire
    /// at all. A grid cannot answer that, because an emulator's whole job is to
    /// consume the sequences and show what is left.
    raw: Vec<u8>,
}

impl Screen {
    fn new(columns: u16, rows: u16) -> Screen {
        Screen {
            grid: vec![vec![' '; columns as usize]; rows as usize],
            columns: columns as usize,
            rows: rows as usize,
            x: 0,
            y: 0,
            pending: Vec::new(),
            raw: Vec::new(),
        }
    }

    fn feed(&mut self, bytes: &[u8]) {
        self.raw.extend_from_slice(bytes);
        self.pending.extend_from_slice(bytes);
        let taken = self.apply();
        self.pending.drain(..taken);
    }

    fn apply(&mut self) -> usize {
        let bytes = std::mem::take(&mut self.pending);
        let mut at = 0;
        while at < bytes.len() {
            match bytes[at] {
                0x1b => match escape(&bytes[at..]) {
                    // Not all here yet: stop, and keep it for the next read.
                    None => break,
                    Some((len, csi)) => {
                        if let Some((params, final_byte, private)) = csi {
                            self.csi(&params, final_byte, private);
                        }
                        at += len;
                    }
                },
                b'\r' => {
                    self.x = 0;
                    at += 1;
                }
                b'\n' => {
                    self.y = (self.y + 1).min(self.rows.saturating_sub(1));
                    at += 1;
                }
                first => {
                    let width = utf8_width(first);
                    if at + width > bytes.len() {
                        break;
                    }
                    if let Ok(s) = std::str::from_utf8(&bytes[at..at + width]) {
                        for c in s.chars() {
                            self.put(c);
                        }
                    }
                    at += width;
                }
            }
        }
        self.pending = bytes;
        at
    }

    fn put(&mut self, c: char) {
        if self.y < self.rows && self.x < self.columns {
            self.grid[self.y][self.x] = c;
        }
        self.x += 1;
        if self.x >= self.columns {
            self.x = 0;
            self.y = (self.y + 1).min(self.rows.saturating_sub(1));
        }
    }

    fn csi(&mut self, params: &[usize], final_byte: u8, private: bool) {
        if private {
            return;
        }
        let at = |i: usize, default: usize| params.get(i).copied().unwrap_or(default);
        match final_byte {
            b'H' | b'f' => {
                self.y = at(0, 1).saturating_sub(1).min(self.rows.saturating_sub(1));
                self.x = at(1, 1)
                    .saturating_sub(1)
                    .min(self.columns.saturating_sub(1));
            }
            b'J' => match at(0, 0) {
                0 => {
                    for x in self.x..self.columns {
                        self.grid[self.y][x] = ' ';
                    }
                    for y in self.y + 1..self.rows {
                        self.grid[y] = vec![' '; self.columns];
                    }
                }
                1 => {
                    for y in 0..self.y {
                        self.grid[y] = vec![' '; self.columns];
                    }
                    for x in 0..=self.x.min(self.columns - 1) {
                        self.grid[self.y][x] = ' ';
                    }
                }
                _ => self.grid = vec![vec![' '; self.columns]; self.rows],
            },
            b'K' => match at(0, 0) {
                0 => {
                    for x in self.x..self.columns {
                        self.grid[self.y][x] = ' ';
                    }
                }
                1 => {
                    for x in 0..=self.x.min(self.columns - 1) {
                        self.grid[self.y][x] = ' ';
                    }
                }
                _ => self.grid[self.y] = vec![' '; self.columns],
            },
            // Colour, attributes, scroll regions, anything else: no character
            // moves, so nothing here does either.
            _ => {}
        }
    }

    /// What is on the screen now, one row per line, trailing space cut.
    fn text(&self) -> String {
        self.grid
            .iter()
            .map(|row| row.iter().collect::<String>().trim_end().to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

/// One escape sequence at the head of `bytes`: how long it is, and the CSI it
/// was if it was one. `None` means it is not all here yet.
#[allow(clippy::type_complexity)]
fn escape(bytes: &[u8]) -> Option<(usize, Option<(Vec<usize>, u8, bool)>)> {
    match bytes.get(1)? {
        b'[' => {
            let private = bytes.get(2).is_some_and(|b| b"<=>?".contains(b));
            let from = 2 + usize::from(private);
            let mut at = from;
            while bytes
                .get(at)
                .is_some_and(|b| b.is_ascii_digit() || *b == b';')
            {
                at += 1;
            }
            let final_byte = *bytes.get(at)?;
            let params = bytes[from..at]
                .split(|b| *b == b';')
                .filter_map(|p| std::str::from_utf8(p).ok()?.parse().ok())
                .collect();
            Some((at + 1, Some((params, final_byte, private))))
        }
        // An operating-system command runs to a bell or a string terminator.
        b']' => {
            let mut at = 2;
            while at < bytes.len() {
                if bytes[at] == 0x07 {
                    return Some((at + 1, None));
                }
                if bytes[at] == 0x1b && bytes.get(at + 1) == Some(&b'\\') {
                    return Some((at + 2, None));
                }
                at += 1;
            }
            None
        }
        // Anything else is two bytes: `ESC c`, `ESC (B` is three but is not on
        // this stream, and a byte consumed wrongly would show up as a character
        // nobody wrote.
        _ => Some((2, None)),
    }
}

fn utf8_width(first: u8) -> usize {
    match first {
        b if b < 0x80 => 1,
        b if b >> 5 == 0b110 => 2,
        b if b >> 4 == 0b1110 => 3,
        b if b >> 3 == 0b11110 => 4,
        _ => 1,
    }
}

// ---------------------------------------------------------------------------
// The pulse
// ---------------------------------------------------------------------------

/// Every character the pulse of a task in progress is drawn with, in both
/// alphabets (TASK-54bb5cddc671): `· • ●` and `. o O`, read off the layout's
/// own table.
pub fn pulse_glyphs() -> Vec<char> {
    use ank_tui::graph::{Alphabet, Glyph};
    let phases = [
        Glyph::InProgress0,
        Glyph::InProgress1,
        Glyph::InProgress2,
        Glyph::InProgress3,
    ];
    let mut out: Vec<char> = Vec::new();
    for alphabet in [Alphabet::Boxes, Alphabet::Ascii] {
        for phase in phases {
            let c = alphabet.glyph(phase);
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

/// Whether two samples of one screen differ in nothing but cells where both
/// carry a glyph of the pulse (TASK-54bb5cddc671): the same lines, each the
/// same width, and every character equal or both the pulse's.
///
/// **Masked where they differ and nowhere else**, so a screen whose only
/// motion is a task in progress breathing counts as settled, and a screen
/// where anything else moved does not. In the ASCII alphabet the pulse is
/// `.`, `o` and `O`, which prose carries too; a frame whose only difference
/// was one of those letters turning into another would be read as settled,
/// and no screen this reader draws changes that way.
pub fn same_but_the_pulse(a: &str, b: &str) -> bool {
    let pulse = pulse_glyphs();
    let (a, b): (Vec<&str>, Vec<&str>) = (a.lines().collect(), b.lines().collect());
    a.len() == b.len()
        && a.iter().zip(&b).all(|(x, y)| {
            let (x, y): (Vec<char>, Vec<char>) = (x.chars().collect(), y.chars().collect());
            x.len() == y.len()
                && x.iter()
                    .zip(&y)
                    .all(|(p, q)| p == q || (pulse.contains(p) && pulse.contains(q)))
        })
}

// ---------------------------------------------------------------------------
// A driven session
// ---------------------------------------------------------------------------

/// `ank tui` on a real terminal of a stated size, drivable while it runs.
pub struct Live {
    child: std::process::Child,
    /// The master side, written to by [`Live::send`]. An `Option` so that the
    /// two endings can close it before they join the drain, which a type with
    /// a [`Drop`] could not do by moving the field out.
    writer: Option<std::fs::File>,
    /// The slave side's device, kept so the window can be moved under a
    /// running session ([`Live::resize`], TASK-643611645dea).
    slave_path: PathBuf,
    screen: std::sync::Arc<std::sync::Mutex<Screen>>,
    /// Every distinct frame this session painted, in order.
    ///
    /// **Sampling misses a frame that does not last.** [`Live::until`] looks at
    /// the screen every twenty-five milliseconds, so a state the reader passes
    /// through faster than that is never seen: measured on macos-latest, a test
    /// waiting for the pre-read notice timed out against a screen that already
    /// carried all 1202 rows. The reader got faster this week and the window
    /// closed. Recorded here instead, so a frame that existed is answerable
    /// whether or not it is still up.
    seen: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    drain: Option<std::thread::JoinHandle<()>>,
}

impl Live {
    /// A session that may not paint, which is what every suite but the one
    /// about painting wants: `NO_COLOR` makes the frames the characters they
    /// are and nothing else.
    pub fn open(repo: &Repo, columns: u16, rows: u16) -> Live {
        Live::opened(repo, columns, rows, false, TERM, &[])
    }

    /// A session that may paint (TASK-6cd41d23b7d1).
    ///
    /// `NO_COLOR` is *removed* from the child's environment rather than left
    /// unset here, and `TERM` is stated: this suite inherits whatever the
    /// developer running it has exported, and a test whose subject is colour
    /// must not be a test that reports the machine.
    pub fn painting(repo: &Repo, columns: u16, rows: u16) -> Live {
        Live::opened(repo, columns, rows, true, TERM, &[])
    }

    /// A session on a terminal that has declared it can render nothing rich
    /// (ADR-ac6be1ebe9aa).
    ///
    /// `TERM=dumb` is the whole of the declaration, and it is what puts the
    /// reader on its ASCII border set and its plain palette at once -- one
    /// probe, two answers. `NO_COLOR` is left set as [`Live::open`] sets it,
    /// because it is beside the point here: a terminal this poor was getting
    /// the plain palette either way.
    pub fn dumb(repo: &Repo, columns: u16, rows: u16) -> Live {
        Live::opened(repo, columns, rows, false, "dumb", &[])
    }

    /// A session with something else in its environment (TASK-d832452630d2).
    ///
    /// `$EDITOR` above all: the criterion for the form is measured by pointing
    /// it at a script that writes a sentinel and finding the sentinel absent,
    /// and a variable the harness could not set would leave that unmeasurable.
    pub fn with(repo: &Repo, columns: u16, rows: u16, env: &[(&str, &str)]) -> Live {
        Live::opened(repo, columns, rows, false, TERM, env)
    }

    /// A session started somewhere other than the repository, with the address
    /// flags the caller names after `tui` (TASK-ad407ccdae3a).
    ///
    /// The cwd is the point: a reader started inside the repository finds its
    /// corpus whatever it does with `--repo`, so only one started outside it
    /// measures whether the flag reached the children it spawns.
    pub fn addressed(cwd: &Path, args: &[&str], columns: u16, rows: u16) -> Live {
        Live::spawned(cwd, args, columns, rows, false, TERM, &[])
    }

    fn opened(
        repo: &Repo,
        columns: u16,
        rows: u16,
        colour: bool,
        term: &str,
        env: &[(&str, &str)],
    ) -> Live {
        Live::spawned(&repo.0, &[], columns, rows, colour, term, env)
    }

    fn spawned(
        cwd: &Path,
        args: &[&str],
        columns: u16,
        rows: u16,
        colour: bool,
        term: &str,
        env: &[(&str, &str)],
    ) -> Live {
        use std::io::Read;

        let (master, slave_path) = pty::open();
        // The child's streams are opened before the window is set: a
        // pseudo-terminal whose last slave descriptor closes is one that hung
        // up, and how permanently depends on the platform.
        let (stdin, stdout, stderr) = (
            pty::slave(&slave_path),
            pty::slave(&slave_path),
            pty::slave(&slave_path),
        );
        pty::resize(&slave_path, columns, rows);
        let mut command = Command::new(ank());
        command
            .arg("tui")
            .args(args)
            .current_dir(cwd)
            .env("ANK_AGENT", AGENT)
            .env("TERM", term);
        in_this_runs_root(&mut command);
        match colour {
            true => command.env_remove("NO_COLOR"),
            false => command.env("NO_COLOR", "1"),
        };
        for (name, value) in env {
            command.env(name, value);
        }
        let child = command
            .stdin(pty::stdio(stdin))
            .stdout(pty::stdio(stdout))
            .stderr(pty::stdio(stderr))
            .spawn()
            .expect("the binary must have been built");

        let screen = std::sync::Arc::new(std::sync::Mutex::new(Screen::new(columns, rows)));
        let seen: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let into = std::sync::Arc::clone(&screen);
        let recorded = std::sync::Arc::clone(&seen);
        let mut reader = master.try_clone().expect("the master side clones");
        // Drained on a thread of its own: a session that painted more than the
        // pseudo-terminal's buffer holds while nobody was reading would
        // deadlock, and a deadlock in a suite is a timeout with no message.
        let drain = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    // Linux answers EIO once the last slave is closed; macOS
                    // answers zero. Both mean the session is over.
                    Ok(0) | Err(_) => return,
                    Ok(n) => {
                        let text = {
                            let mut screen = into.lock().unwrap();
                            screen.feed(&buf[..n]);
                            screen.text()
                        };
                        // Distinct frames only: a chunk that repaints nothing
                        // visible is not a frame, and the history is read by a
                        // linear search.
                        let mut history = recorded.lock().unwrap();
                        if history.last().map(|last| last != &text).unwrap_or(true) {
                            history.push(text);
                        }
                    }
                }
            }
        });
        Live {
            child,
            writer: Some(master),
            slave_path,
            screen,
            seen,
            drain: Some(drain),
        }
    }

    /// The window, moved under the running session (TASK-643611645dea).
    ///
    /// **The terminal's size changes; whether the reader is told is the
    /// terminal's business.** `TIOCSWINSZ` signals the terminal's foreground
    /// process group, and a child this suite spawned has no controlling
    /// terminal to be in the foreground of, so no `SIGWINCH` reaches it.
    /// What the reader does instead is read the window off the terminal
    /// before every frame it draws, so the next key it answers is answered
    /// at the new size -- which is what a caller here sends after this, and
    /// what it then waits for on the frame.
    ///
    /// The emulator's grid is not moved: it keeps the columns the session
    /// opened with, so a frame drawn narrower leaves the old cells past its
    /// right edge standing. A caller measures the new edge, not the old.
    pub fn resize(&self, columns: u16, rows: u16) {
        pty::resize(&self.slave_path, columns, rows);
    }

    /// **Returns the frame the predicate matched on**, which is the only frame
    /// a caller waiting for a transient state can safely assert about. Reading
    /// the terminal again afterwards asks a second question: the screen may
    /// have moved on between the match and the read, and the assertion then
    /// fails on a frame that was right when it was seen. Measured on
    /// 2026-08-30, twice on one branch: the reader finished its read between
    /// `until` and a following `now()`, so a test waiting for the unread notice
    /// captured the populated list instead and reported that the first frame
    /// had already read the corpus.
    ///
    /// Most of the hundred and seventy-odd callers wait for a state that stays,
    /// and drop this. It costs them nothing.
    /// The first frame this session ever painted that satisfies `done`,
    /// whether or not it is still on screen.
    ///
    /// **For a state that passes.** [`Live::until`] samples, so it answers
    /// about states that last; this reads the recorded history, so it answers
    /// about states that happened. Use it only when the thing asked about is
    /// transient by nature, such as the frame drawn before a read lands: for
    /// anything that stays, `until` is the honest question, and searching
    /// history there would match a frame from before the keystroke under test.
    pub fn ever(&self, what: &str, done: impl Fn(&str) -> bool) -> String {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(hit) = self
                .seen
                .lock()
                .unwrap()
                .iter()
                .find(|frame| done(frame))
                .cloned()
            {
                return hit;
            }
            if std::time::Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        panic!(
            "no frame this session painted ever {what}; it painted {} distinct \
             frames, the last being:\n{}",
            self.seen.lock().unwrap().len(),
            self.screen.lock().unwrap().text()
        );
    }

    /// The screen once `done` holds on it, and once it is the same screen as
    /// the sample before, the pulse masked (TASK-76e1fccf8b67).
    ///
    /// **A screen that satisfies a needle may be one the terminal has only
    /// half handed over.** The title, the count and the cursor arrive in the
    /// first chunk of a repaint, and the rows under them may still be the
    /// frame the key replaced: `a_dumb_terminal_draws_the_graph_in_the_layouts_ascii_alphabet`
    /// read 8 rows of 11 that way, #646 read 9 on macOS, and TASK-aeea26cac0f9
    /// a key line not yet rewritten. Measured before this, three runs of the
    /// suite: of 759 waits each, 111, 103 and 106 returned a screen that had
    /// changed on the very next sample. A torn frame lasts between two reads
    /// of the drain and never across two samples, so two equal ones are a
    /// frame the reader finished writing, at the cost of one more sample.
    pub fn until(&self, what: &str, done: impl Fn(&str) -> bool) -> String {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let mut previous = String::new();
        while std::time::Instant::now() < deadline {
            let seen = self.screen.lock().unwrap().text();
            if done(&seen) && same_but_the_pulse(&seen, &previous) {
                return seen;
            }
            previous = seen;
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        panic!(
            "timed out waiting for {what}:\n{}",
            self.screen.lock().unwrap().text()
        );
    }

    /// The screen as it is, once it has stopped moving.
    ///
    /// Settled first, because a screen read the instant a needle appeared is
    /// half a screen: the reader writes a frame in one call but a terminal
    /// hands it over in whatever pieces it likes.
    ///
    /// **And a screen that has not read the corpus has not settled**
    /// (TASK-fff0a98511b2). The reader draws its opening frame before it spawns
    /// anything, so on a small corpus the frame that follows it can arrive
    /// inside one of the polls below -- and two equal samples of the opening
    /// frame would hand a suite an empty listing that is about to be filled.
    /// It is not a state anybody wants to assert on: every caller of this asks
    /// what a drawn screen says about a corpus, and there is exactly one
    /// suite that is about the opening itself, which reads [`Live::now`]
    /// instead.
    ///
    /// Stated here rather than at each barrier, because that is where it holds
    /// for every suite at once: there are twenty `until("the session to open")`
    /// calls in this crate and one of them going unfixed is a flake nobody
    /// reproduces.
    ///
    /// **Settled modulo the pulse** (TASK-54bb5cddc671). A task in progress on
    /// the graph changes its cell four times a second, so two samples two
    /// hundred milliseconds apart of a screen that has otherwise stopped are
    /// as often different as equal. They are compared with the pulse's cells
    /// masked ([`same_but_the_pulse`]), and what comes back is the second
    /// sample, at whichever phase it was taken: a caller comparing two
    /// frames where a task pulses compares them the same way.
    pub fn frame(&self) -> String {
        let mut last = String::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while std::time::Instant::now() < deadline {
            let now = self.screen.lock().unwrap().text();
            if !now.trim().is_empty() && same_but_the_pulse(&now, &last) && !now.contains(UNREAD) {
                return now;
            }
            last = now;
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        panic!("the screen never stopped moving:\n{last}");
    }

    /// The screen as it is at this instant, settled or not.
    ///
    /// The one thing [`Live::frame`] cannot give, and the suite about what a
    /// session costs to open is the one that needs it: the frame it measures is
    /// by construction the one drawn before the corpus was read.
    pub fn now(&self) -> String {
        self.screen.lock().unwrap().text()
    }

    pub fn send(&mut self, bytes: &str) {
        use std::io::Write;
        let writer = self.writer.as_mut().expect("the session is still open");
        writer
            .write_all(bytes.as_bytes())
            .expect("the terminal must accept a keystroke");
        writer.flush().unwrap();
    }

    /// A press of the left button at a point on the screen, and the release
    /// after it (TASK-dd9747e5e305).
    ///
    /// **SGR, which is the encoding the reader asked the terminal for.**
    /// `EnableMouseCapture` turns on `?1006`, and it is the one encoding whose
    /// coordinates are decimal text rather than bytes offset by thirty-two --
    /// so it carries a column past two hundred and twenty-three, and so a suite
    /// can spell one by hand. The parameters are one-based, which is what a
    /// terminal counts in and what the reader's crossterm subtracts back out.
    ///
    /// The release is sent because a terminal sends one. Nothing in this reader
    /// answers it, and a suite that left it out would be asserting on a stream
    /// no terminal produces.
    pub fn tap(&mut self, column: u16, row: u16) {
        self.send(&format!("\x1b[<0;{};{}M", column + 1, row + 1));
        self.send(&format!("\x1b[<0;{};{}m", column + 1, row + 1));
    }

    /// Every distinct frame this session has painted so far, in order
    /// (TASK-54bb5cddc671): what a suite counting the frames a clock drew
    /// reads, before and after the interval it measures.
    pub fn painted(&self) -> Vec<String> {
        self.seen.lock().unwrap().clone()
    }

    /// The session, waited for until it ends by itself, and how it ended
    /// (TASK-933fdb42c7d9).
    ///
    /// For a session something other than `q` on the keyboard is meant to end
    /// -- a tap on the key line's quit -- so nothing is sent: a keystroke here
    /// would be the very press the test is asking whether the tap made.
    pub fn exited(mut self) -> std::process::ExitStatus {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(status) = self
                .child
                .try_wait()
                .expect("the session can be waited for")
            {
                drop(self.writer.take());
                if let Some(drain) = self.drain.take() {
                    drain.join().expect("the drain must not panic");
                }
                return status;
            }
            if std::time::Instant::now() >= deadline {
                let _ = self.child.kill();
                panic!(
                    "the session never ended by itself:\n{}",
                    self.screen.lock().unwrap().text()
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
    }

    /// Every byte the session wrote, sequences included.
    pub fn raw(&self) -> Vec<u8> {
        self.screen.lock().unwrap().raw.clone()
    }

    pub fn quit(self) {
        let _ = self.ended();
    }

    /// The session quit, and every byte it wrote, the teardown included.
    ///
    /// Separate from [`Live::quit`] because what a reader writes on its way out
    /// is only readable once it has gone: the drain thread is still feeding the
    /// screen while the child restores the terminal, and a stream read before
    /// the join is a stream missing exactly the sequences that say the terminal
    /// was given back.
    pub fn ended(mut self) -> Vec<u8> {
        self.send("q");
        let status = self.child.wait().expect("the session must end");
        assert!(status.success(), "a reader that quits answers 0: {status}");
        drop(self.writer.take());
        if let Some(drain) = self.drain.take() {
            drain.join().expect("the drain must not panic");
        }
        self.screen.lock().unwrap().raw.clone()
    }
}

impl Drop for Live {
    /// **A session nobody quit is killed, and then reaped** (TASK-69e774ef9d64).
    ///
    /// [`Live::quit`], [`Live::ended`] and [`Live::exited`] each end the child,
    /// and a test that panics before it reaches one of them reaches none:
    /// `std::process::Child` does not kill on drop. The reader then outlived
    /// the suite, and with the master gone it read end of file in a loop.
    /// Measured on 2026-10-28: four of them, orphaned under init from four
    /// worktrees, held a four-core machine at 100% for days. Whatever made a
    /// test panic, it leaves no process behind it.
    ///
    /// The drain is not joined here. It ends on its own once the child is
    /// gone, and a join in a `Drop` that runs during a panic is a second place
    /// for the suite to hang.
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

/// `ank tui` on a terminal whose other side is then closed, as it is when
/// whatever opened the terminal dies (TASK-69e774ef9d64).
///
/// **Closed once the reader has gone quiet, and that is the condition being
/// tested.** A reader still painting when its terminal goes meets the closure
/// as a failed write and ends there; the one that spun for days was idle,
/// waiting on a keystroke. So the frame is read until the session is open and
/// then until nothing has arrived for a second, and only then is the far side
/// closed. A reader that paints again later only makes this easier to pass,
/// never harder: the bound is on the direction that keeps the test honest.
///
/// Separate from [`Live`] because a `Live` cannot close its master: the drain
/// holds a copy until the child is gone, which is the very thing this waits
/// to see. Here the copy is given up by the thread reading it, told to stop by
/// a flag and woken by a byte written into the slave, which reaches the master
/// and nobody else. The child is handed back running; the caller waits for it.
pub fn abandoned(repo: &Repo, columns: u16, rows: u16) -> std::process::Child {
    use std::io::{Read, Write};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    let (master, slave_path) = pty::open();
    let (stdin, stdout, stderr) = (
        pty::slave(&slave_path),
        pty::slave(&slave_path),
        pty::slave(&slave_path),
    );
    pty::resize(&slave_path, columns, rows);
    let mut command = Command::new(ank());
    command
        .arg("tui")
        .current_dir(&repo.0)
        .env("ANK_AGENT", AGENT)
        .env("TERM", TERM)
        .env("NO_COLOR", "1");
    in_this_runs_root(&mut command);
    let mut child = command
        .stdin(pty::stdio(stdin))
        .stdout(pty::stdio(stdout))
        .stderr(pty::stdio(stderr))
        .spawn()
        .expect("the binary must have been built");

    let mut reader = master.try_clone().expect("the master side clones");
    let stop = std::sync::Arc::new(AtomicBool::new(false));
    let stopping = std::sync::Arc::clone(&stop);
    let (chunks, arriving) = std::sync::mpsc::channel::<Vec<u8>>();
    let drain = std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => return,
                // `reader` is dropped on the way out: this is the copy of
                // the master that closes first.
                Ok(_) if stopping.load(Ordering::SeqCst) => return,
                Ok(n) => {
                    if chunks.send(buf[..n].to_vec()).is_err() {
                        return;
                    }
                }
            }
        }
    });

    let mut screen = Screen::new(columns, rows);
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut open = false;
    let quiet = loop {
        if Instant::now() >= deadline {
            break false;
        }
        match arriving.recv_timeout(Duration::from_secs(1)) {
            Ok(chunk) => {
                screen.feed(&chunk);
                open = open || screen.text().contains("2 ENTITIES");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) if open => break true,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break false,
        }
    };
    if !quiet {
        let _ = child.kill();
        let _ = child.wait();
        panic!(
            "the session never opened and went quiet:\n{}",
            screen.text()
        );
    }
    stop.store(true, Ordering::SeqCst);
    drop(master);
    pty::slave(&slave_path)
        .write_all(b"x")
        .expect("the slave takes a byte for the master");
    drain.join().expect("the drain must not panic");
    child
}
