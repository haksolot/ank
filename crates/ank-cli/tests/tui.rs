//! `ank tui` through the binary (TASK-49746735127f, ADR-8bd76e8d7c4e).
//!
//! CLAUDE.md leaves no choice about where this suite lives: a criterion that
//! talks about the binary is tested through the binary, and twice in this
//! repository green unit tests covered code that was right on a path the binary
//! never reached. A TUI is the extreme case of that trap -- every interesting
//! behaviour sits behind terminal setup a unit test does not perform -- so the
//! session here is driven through a real pseudo-terminal and the frames are
//! read off the master side.
//!
//! It lives in `ank-cli` rather than in `ank-tui` for one mechanical reason:
//! `CARGO_BIN_EXE_ank` is defined only for the package that declares the
//! binary, and a suite that could not name the binary would be back to testing
//! the function instead of the process.
//!
//! **What is covered on which platform.** The refusal with no terminal runs
//! everywhere, which matters most: it is the one an agent meets. The driven
//! session is `#[cfg(unix)]`, because a pseudo-terminal on Windows is ConPTY
//! and reaching it means the console API this workspace does not otherwise
//! call. The reader itself is platform-independent by construction -- it does
//! no FFI, and the escape sequences it writes are the same bytes on all three
//! (`crates/ank-tui/src/frame.rs`).
//!
//! **The writing half is measured here too** (TASK-b50b340c0bb1). What a unit
//! test can say about `claim` from a screen is which `argv` would have been
//! spawned; what it cannot say is that the ref which came out is the ref a
//! shell claim makes, that a refused `done` left the task where it was, or that
//! a screen nobody touched for three seconds renewed nothing. All three are
//! facts about a process and a git repository, so all three are asserted
//! against a real one.

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

// ---------------------------------------------------------------------------
// A corpus of its own
// ---------------------------------------------------------------------------

/// A scratch repository nothing else in this suite uses.
struct Repo(PathBuf);

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch(what: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "ank-tui-it-{what}-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

impl Repo {
    /// A repository with a corpus, one ADR, one task, and a claim held on the
    /// task -- which is the whole of what the criterion asks a frame to show.
    fn seeded(what: &str) -> Repo {
        let root = scratch(what);
        let repo = Repo(root);
        repo.git(&["init", "--initial-branch=main"]);
        repo.git(&["config", "user.email", "suite@example.invalid"]);
        repo.git(&["config", "user.name", "The Suite"]);
        repo.git(&["config", "commit.gpgsign", "false"]);
        // **The signing regime is stated here rather than inherited.** `accept`
        // signs where the repository can sign (ADR-964be4d940b2), and "can" is
        // read out of git's configuration -- which, unset locally, is whatever
        // the developer running this suite happens to have globally. A test
        // whose corpus is signed on one machine and advisory on the next is a
        // test that reports the machine. Set empty, this corpus is squarely in
        // §8's advisory mode, which is the regime `review` then names on the
        // screen and the one both roads through `accept` take.
        repo.git(&["config", "user.signingkey", ""]);
        std::fs::create_dir_all(repo.0.join("src")).unwrap();
        std::fs::write(repo.0.join("src/lib.rs"), "// code\n").unwrap();
        repo.ank(HOLDER, &["init"]);
        // Without this, `accept` cannot tell where ratification is allowed to
        // happen: there is no origin here, so `default_branch` has no second
        // source (§12). A corpus that cannot name its default branch is a
        // corpus in which no ratification is possible at all, which is not the
        // repository this suite is modelling.
        repo.ank(HOLDER, &["config", "default_branch", "main"]);
        repo.ank(
            HOLDER,
            &[
                "new",
                "adr",
                "--title",
                ADR_TITLE,
                "--scope",
                "src/**",
                "--constraint",
                "Every byte shown is a byte the CLI printed.",
            ],
        );
        repo.ank(
            HOLDER,
            &[
                "new",
                "task",
                "--title",
                TASK_TITLE,
                "--scope",
                "src/**",
                "--criteria",
                CRITERION,
            ],
        );
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-m", "seed"]);
        let id = repo.only(&["--type", "task"]);
        repo.ank(HOLDER, &["claim", &id]);
        repo
    }

    fn git(&self, args: &[&str]) -> Output {
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

    fn ank(&self, agent: &str, args: &[&str]) -> Output {
        let out = Command::new(ANK)
            .args(args)
            .current_dir(&self.0)
            .env("ANK_AGENT", agent)
            .output()
            .expect("the binary must have been built");
        assert!(
            out.status.success(),
            "ank {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }

    fn stdout(&self, agent: &str, args: &[&str]) -> String {
        String::from_utf8_lossy(&self.ank(agent, args).stdout).to_string()
    }

    /// A second task, unclaimed, and its identifier.
    ///
    /// The seeded one is held by [`HOLDER`], and a task the suite means to claim
    /// from the screen has to be free. Named by its title rather than found by
    /// elimination: two tasks in a corpus and a filter that leaves one is a
    /// filter that will leave two the day a third arrives.
    fn spare(&self, title: &str, criteria: &str) -> String {
        self.ank(
            OTHER,
            &[
                "new",
                "task",
                "--title",
                title,
                "--scope",
                "src/**",
                "--criteria",
                criteria,
            ],
        );
        self.only(&["--type", "task", "--status", "open"])
    }

    /// Every read the reader makes on its first frame, made once beforehand.
    ///
    /// `.ank/index.db` is the CLI's own cache and it is written the first time a
    /// corpus is searched. Warming it before a snapshot is what separates "the
    /// session wrote something" from "the first read built a cache".
    fn warm(&self, agent: &str) {
        let _ = self.stdout(agent, &["find", "--json"]);
        let _ = self.stdout(agent, &["status", "--json"]);
        let _ = self.stdout(agent, &["scope", "src/**", "--json"]);
    }

    /// The one identifier a `find` filter leaves, read out of the `--json`
    /// document rather than off the human page.
    fn only(&self, filter: &[&str]) -> String {
        let mut args = vec!["find"];
        args.extend_from_slice(filter);
        args.push("--json");
        let doc = self.stdout(HOLDER, &args);
        let ids = ids_of(&doc);
        assert_eq!(ids.len(), 1, "{filter:?} left {ids:?} in\n{doc}");
        ids[0].clone()
    }
}

const HOLDER: &str = "claude-code/opus-5+tui-suite";
const OTHER: &str = "claude-code/opus-5+someone-else";
/// A third identity, holding nothing, so that a claim taken from the screen is
/// taken by an agent free to take one (§3: one live claim per identity).
const READER: &str = "claude-code/opus-5+at-the-keyboard";
const ADR_TITLE: &str = "The reader draws what the CLI printed";
const TASK_TITLE: &str = "A task the reader opens";
/// Deliberately wider than the window the suite opens, and ending on a marker
/// nothing else carries: a reader that cut a body line at the right edge would
/// lose [`TAIL`], and losing it is exactly what "whole" forbids.
const CRITERION: &str = "The frame names this entity, and the body arrives whole: this sentence is longer than the window this suite opens, so a reader that cut it at the right edge would lose TAIL-9f31 off its end.";
/// The last word of [`CRITERION`], which only a whole body carries.
const TAIL: &str = "TAIL-9f31";

/// Every `"id":"..."` of a document, in the order it carries them.
///
/// A five-line reader rather than a parser: the suite needs the identifiers a
/// document states, the escaper on the other side never puts a backslash inside
/// one, and a JSON dependency for this would be a dependency the tree does not
/// otherwise have.
fn ids_of(doc: &str) -> Vec<String> {
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
fn short_of(id: &str) -> String {
    let (kind, rest) = id.split_once('-').expect("an identifier has a kind");
    format!("{kind}-{}", &rest[..4])
}

// ---------------------------------------------------------------------------
// The refusal, on every platform
// ---------------------------------------------------------------------------

/// The one an agent meets, and the reason it exists (ADR-8bd76e8d7c4e): `ank`
/// is run by agents far more often than by people, and one that typed `ank tui`
/// by accident must get a refusal it can read rather than a process that hangs
/// holding a terminal it does not have.
#[test]
fn with_no_terminal_it_refuses_with_the_environment_code_and_names_what_to_run() {
    let repo = Repo::seeded("no-terminal");
    let out = Command::new(ANK)
        .arg("tui")
        .current_dir(&repo.0)
        .env("ANK_AGENT", HOLDER)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary must have been built")
        .wait_with_output()
        .expect("it must not hang: a refusal is the whole point");
    assert_eq!(
        out.status.code(),
        Some(9),
        "stdout {:?} stderr {:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let said = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(said.contains("error[9]:"), "{said}");
    assert!(said.contains("terminal"), "{said}");
    assert!(
        said.contains("ank context"),
        "a refusal names the command to run next: {said}"
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).is_empty(),
        "nothing was drawn into the pipe"
    );
}

/// `--json` does not buy a way past it. §4 makes `--json` available on every
/// verb without exception, and a `tui` that answered a document into a pipe
/// while refusing a screen there would make the sentence above a sentence with
/// a footnote.
#[test]
fn json_does_not_exempt_a_caller_from_the_terminal() {
    let repo = Repo::seeded("no-terminal-json");
    let out = Command::new(ANK)
        .args(["tui", "--json"])
        .current_dir(&repo.0)
        .env("ANK_AGENT", HOLDER)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(9));
}

// ---------------------------------------------------------------------------
// The driven session
// ---------------------------------------------------------------------------

/// A pseudo-terminal, opened with the four calls POSIX names for it.
///
/// Declared here rather than taken from a crate. `libc` is in the lockfile and
/// would have been the tidier road, but it is not compiled for this target
/// today and §13 spends a dependency only on necessity: what is needed is four
/// symbols and one flag, `O_RDWR`, whose value POSIX fixes at 2 on every system
/// this suite runs on. Rust links the platform's C library already, so nothing
/// is added to the link line either.
#[cfg(unix)]
mod pty {
    use std::ffi::CStr;
    use std::fs::File;
    use std::os::fd::{FromRawFd, IntoRawFd};
    use std::os::raw::{c_char, c_int};
    use std::path::PathBuf;

    extern "C" {
        fn posix_openpt(flags: c_int) -> c_int;
        fn grantpt(fd: c_int) -> c_int;
        fn unlockpt(fd: c_int) -> c_int;
        fn ptsname(fd: c_int) -> *mut c_char;
    }

    const O_RDWR: c_int = 2;

    /// The master side as a `File`, and the path of the slave to hand the
    /// child.
    pub fn open() -> (File, PathBuf) {
        // SAFETY: the four calls are the POSIX pseudo-terminal sequence, in
        // order, and every return is checked before the next is made. The
        // pointer `ptsname` answers with is owned by the C library and is
        // copied out before anything else can invalidate it.
        unsafe {
            let master = posix_openpt(O_RDWR);
            assert!(master >= 0, "posix_openpt failed");
            assert_eq!(grantpt(master), 0, "grantpt failed");
            assert_eq!(unlockpt(master), 0, "unlockpt failed");
            let name = ptsname(master);
            assert!(!name.is_null(), "ptsname answered nothing");
            let path = PathBuf::from(
                CStr::from_ptr(name)
                    .to_str()
                    .expect("a device path is UTF-8")
                    .to_string(),
            );
            (File::from_raw_fd(master), path)
        }
    }

    /// The slave, opened once per standard stream the child is given.
    pub fn slave(path: &PathBuf) -> File {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .expect("the slave side of a pseudo-terminal must open")
    }

    /// A `File` handed to a child as one of its standard streams.
    pub fn stdio(file: File) -> std::process::Stdio {
        // SAFETY: the descriptor is owned by `file`, which gives it up here, so
        // exactly one owner reaches the child.
        unsafe { std::process::Stdio::from_raw_fd(file.into_raw_fd()) }
    }
}

/// Runs `ank tui` on a real terminal and answers everything it drew.
///
/// The output is drained on a thread of its own. A session that painted more
/// than the pseudo-terminal's buffer holds while nobody was reading would
/// deadlock, and a deadlock in a suite is a timeout with no message.
#[cfg(unix)]
fn drive(repo: &Repo, agent: &str, commands: &[&str]) -> String {
    on_a_terminal(repo, agent, &["tui"], commands)
}

/// The same, for a call that takes flags and ends on its own.
#[cfg(unix)]
fn on_a_terminal(repo: &Repo, agent: &str, args: &[&str], commands: &[&str]) -> String {
    use std::io::{Read, Write};

    let (master, slave_path) = pty::open();
    let mut child = Command::new(ANK)
        .args(args)
        .current_dir(&repo.0)
        .env("ANK_AGENT", agent)
        // The window, stated rather than measured: the reader declines the
        // ioctl for the reason its module header gives, and a suite that could
        // not choose the window could not assert what a frame holds.
        .env("COLUMNS", "120")
        .env("LINES", "40")
        .env("NO_COLOR", "1")
        .stdin(pty::stdio(pty::slave(&slave_path)))
        .stdout(pty::stdio(pty::slave(&slave_path)))
        .stderr(pty::stdio(pty::slave(&slave_path)))
        .spawn()
        .expect("the binary must have been built");

    let mut reader = master
        .try_clone()
        .expect("the master side must be clonable for the drain");
    let drain = std::thread::spawn(move || {
        let mut seen = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                // Linux answers EIO once the last slave is closed; macOS
                // answers zero. Both mean the session is over.
                Ok(0) | Err(_) => break,
                Ok(n) => seen.extend_from_slice(&buf[..n]),
            }
        }
        seen
    });

    let mut writer = master;
    for command in commands {
        writeln!(writer, "{command}").expect("the terminal must accept a command");
        writer.flush().unwrap();
    }
    let status = child.wait().expect("the session must end");
    assert!(
        status.success(),
        "the session ended with {status}, and a reader that quits answers 0"
    );
    // The child's slave descriptors went with it; dropping the master ends the
    // drain, which is otherwise blocked reading a terminal nobody will write
    // to again.
    drop(writer);
    let seen = drain.join().expect("the drain must not panic");
    String::from_utf8_lossy(&seen).to_string()
}

#[cfg(unix)]
#[test]
fn a_driven_session_names_the_entities_the_corpus_carries() {
    let repo = Repo::seeded("frames");
    let task = repo.only(&["--type", "task"]);
    let adr = repo.only(&["--type", "adr"]);

    // Someone else holds nothing here, so the claim on the screen is the one
    // this suite took: "which claim is held by whom", with a name on it. The
    // task is what is opened, because the criterion is written into its body
    // and that is what "whole" is asserted against.
    let seen = drive(&repo, HOLDER, &["f task", "", "b", "f", "q"]);

    assert!(
        seen.contains("\x1b[?1049h") && seen.contains("\x1b[?1049l"),
        "the session used the alternate screen and gave it back"
    );
    for expected in [
        short_of(&task),
        short_of(&adr),
        TASK_TITLE.to_string(),
        ADR_TITLE.to_string(),
        HOLDER.to_string(),
        "CLAIMS (1)".to_string(),
        "ENTITIES".to_string(),
    ] {
        assert!(
            seen.contains(&expected),
            "the frames never named {expected:?}:\n{seen}"
        );
    }
    // The body of the entity the empty line opened, whole: the criterion is
    // written into it, and the frontmatter around it arrived with it.
    assert!(
        seen.contains("The frame names this entity"),
        "the body was not shown:\n{seen}"
    );
    // And whole in both directions. The criterion is wider than the window, so
    // its end reaches the screen only if the reader wrapped rather than cut.
    assert!(
        seen.contains(TAIL),
        "the body was cut at the right edge, and {TAIL} went with it:\n{seen}"
    );
    assert!(
        seen.contains("done_criteria:"),
        "the frontmatter arrived with the body:\n{seen}"
    );
    assert!(
        seen.contains("claimed by claude-code/opus-5+tui-suite"),
        "the entity view says who holds it:\n{seen}"
    );
    assert!(
        seen.contains(&short_of(&adr)),
        "the constraints binding the scope are on the entity screen:\n{seen}"
    );
}

/// Every row on the screen is a row `find` answers with, and nothing else.
///
/// This is the half of "every byte it shows is obtained by running the CLI"
/// that a test can state: an identifier the frames carry and the corpus does
/// not would be a row the reader invented.
#[cfg(unix)]
#[test]
fn the_frames_carry_no_identifier_the_corpus_does_not() {
    let repo = Repo::seeded("no-invention");
    let real: Vec<String> = ids_of(&repo.stdout(HOLDER, &["find", "--json"]))
        .iter()
        .map(|id| short_of(id))
        .collect();
    let seen = drive(&repo, HOLDER, &["", "b", "j", "", "b", "q"]);

    let mut found = 0;
    let mut rest = seen.as_str();
    while let Some(at) = rest.find('-') {
        // A short identifier is `<KIND>-xxxx`; the kinds are the four this
        // corpus has (ADR-c9f9d1a05b23).
        let start = rest[..at]
            .rfind(|c: char| !c.is_ascii_alphabetic())
            .map_or(0, |i| i + 1);
        let kind = &rest[start..at];
        if ["ADR", "SPEC", "TASK", "LOG"].contains(&kind) && rest.len() >= at + 5 {
            let candidate = &rest[start..at + 5];
            if candidate[kind.len() + 1..]
                .chars()
                .all(|c| c.is_ascii_hexdigit())
            {
                assert!(
                    real.iter().any(|r| r == candidate),
                    "the frames name {candidate}, which the corpus does not carry: {real:?}"
                );
                found += 1;
            }
        }
        rest = &rest[at + 1..];
    }
    assert!(found >= 2, "the frames named no identifier at all");
}

/// Quitting leaves the corpus exactly as it was found (ADR-8bd76e8d7c4e).
///
/// Both halves are compared, because they are two stores and only one of them
/// is a file: `.ank/` is content, `refs/ank/*` is coordination, and a reader
/// that renewed a claim would move the second while leaving the first alone.
///
/// **The index is warmed first, and that is not a cheat.** `.ank/index.db` is
/// the CLI's own cache and it is written the first time a corpus is searched,
/// by `ank find` and not by the reader. Warming it before the snapshot is what
/// separates "the session wrote something" from "the first read built a cache",
/// which is the question this test is asking.
#[cfg(unix)]
#[test]
fn quitting_leaves_no_file_and_no_ref_changed() {
    let repo = Repo::seeded("read-only");
    // Warm the index, and let a claim be taken by somebody else so that the
    // refs under test carry more than one entry.
    let adr = repo.only(&["--type", "adr"]);
    let _ = repo.stdout(HOLDER, &["find", "--json"]);
    let _ = repo.stdout(HOLDER, &["scope", "src/**", "--json"]);
    let _ = repo.stdout(HOLDER, &["show", &adr, "--json"]);

    let before = (corpus_files(&repo), ank_refs(&repo));
    assert!(!before.0.is_empty(), "the corpus has files to compare");
    assert!(!before.1.is_empty(), "a claim is held, so a ref exists");

    // A session that uses every road through the reader: filter, open, page,
    // the constraints pane, back, search, quit.
    let seen = drive(
        &repo,
        HOLDER,
        &["f adr", "", "n", "c", "g", "b", "f", "/task", "j", "k", "q"],
    );
    assert!(seen.contains("ENTITIES"), "the session ran:\n{seen}");
    assert!(seen.contains("CONSTRAINTS"), "it opened an entity:\n{seen}");

    let after = (corpus_files(&repo), ank_refs(&repo));
    assert_eq!(before.0, after.0, "a file under .ank/ changed");
    assert_eq!(before.1, after.1, "a ref under refs/ank/ changed");
}

/// Opening the task you hold takes nothing and creates nothing
/// (ADR-8bd76e8d7c4e).
///
/// The one thing a session can move is the lease, and only because `ank show`
/// renews it when the id is the task the caller holds (§3, ADR-0bb7ea8991bc) --
/// which is what typing that command in a shell does, and there is no second
/// dispatch path here for it to do anything else. What is asserted is what
/// stays true whichever second the session lands in: no file is written, no ref
/// is created or removed, and the claim is still held by the same agent.
///
/// The test above is where the criterion lives, and it is stronger on purpose:
/// a session that never asks about the held task leaves every ref byte for byte
/// where it was. The reader renews **nothing on its own** -- a screen left open
/// all night runs no command at all.
#[cfg(unix)]
#[test]
fn opening_the_task_you_hold_takes_nothing_and_creates_nothing() {
    let repo = Repo::seeded("renewal");
    let task = repo.only(&["--type", "task"]);
    let _ = repo.stdout(HOLDER, &["find", "--json"]);
    let _ = repo.stdout(HOLDER, &["scope", "src/**", "--json"]);
    let _ = repo.stdout(HOLDER, &["show", &task, "--json"]);

    let files = corpus_files(&repo);
    let names = ref_names(&repo);
    let seen = drive(&repo, HOLDER, &["f task", "", "q"]);
    assert!(seen.contains(TAIL), "the held task was opened:\n{seen}");

    assert_eq!(
        files,
        corpus_files(&repo),
        "the reader wrote a file: only the lease may move"
    );
    assert_eq!(names, ref_names(&repo), "a ref was created or removed");
    let status = repo.stdout(HOLDER, &["status", "--json"]);
    assert!(
        status.contains(&format!("\"id\":\"{task}\"")),
        "the claim is still this agent's:\n{status}"
    );
}

/// Every file under `.ank/`, by path and by content, so that a change of bytes
/// is caught and a change of timestamp is not.
#[cfg(unix)]
fn corpus_files(repo: &Repo) -> Vec<(String, Vec<u8>)> {
    let root = repo.0.join(".ank");
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

/// Every ref under `refs/ank/`, with the object it points at.
#[cfg(unix)]
fn ank_refs(repo: &Repo) -> String {
    String::from_utf8_lossy(
        &repo
            .git(&[
                "for-each-ref",
                "--format=%(refname) %(objectname)",
                "refs/ank/",
            ])
            .stdout,
    )
    .to_string()
}

/// The names of the refs under `refs/ank/`, without the objects they point at.
#[cfg(unix)]
fn ref_names(repo: &Repo) -> String {
    String::from_utf8_lossy(
        &repo
            .git(&["for-each-ref", "--format=%(refname)", "refs/ank/"])
            .stdout,
    )
    .to_string()
}

/// A claim held by somebody else is named, with its holder, which is the other
/// half of "which claim is held by whom".
#[cfg(unix)]
#[test]
fn a_claim_held_elsewhere_is_named_with_its_holder() {
    let repo = Repo::seeded("elsewhere");
    // A second task, claimed under a second identity.
    repo.ank(
        OTHER,
        &[
            "new",
            "task",
            "--title",
            "A task somebody else holds",
            "--scope",
            "src/**",
            "--criteria",
            "Held by another agent.",
        ],
    );
    let mine = repo.only(&["--type", "task", "--status", "in_progress"]);
    let theirs = ids_of(&repo.stdout(OTHER, &["find", "--type", "task", "--json"]))
        .into_iter()
        .find(|id| id != &mine)
        .expect("the second task exists");
    repo.ank(OTHER, &["claim", &theirs]);

    let seen = drive(&repo, HOLDER, &["q"]);
    assert!(seen.contains(OTHER), "the other holder is named:\n{seen}");
    assert!(seen.contains(HOLDER), "and so is this one:\n{seen}");
    assert!(seen.contains("CLAIMS (2)"), "{seen}");
    assert!(
        seen.contains(&format!("* {}", short_of(&mine))),
        "the caller's own claim is the marked one:\n{seen}"
    );
}

/// A refusal the CLI gave is what the screen shows, in the CLI's own bytes, and
/// the session survives it (ADR-8bd76e8d7c4e).
#[cfg(unix)]
#[test]
fn a_refusal_on_screen_is_the_one_the_cli_gave() {
    let repo = Repo::seeded("refusal");
    // `LOG-000000000000` is not in this corpus, so `show` refuses with the
    // sentence and the code it always gives.
    let seen = drive(&repo, HOLDER, &["LOG-000000000000", "q"]);
    assert!(
        seen.contains("no entity") || seen.contains("LOG-000000000000"),
        "the refusal reached the screen:\n{seen}"
    );
    assert!(
        seen.contains("ENTITIES"),
        "and the session kept its shape:\n{seen}"
    );
}

/// `--json` on a terminal answers one document and opens no session (§4).
///
/// Full scriptability is an invariant and not an option, and this is what it
/// means for a verb whose ordinary answer is a screen: the reader's own frame,
/// as data, through the one writer and the one escaper every other document in
/// this tool goes through (ADR-6fd69efb629c).
#[cfg(unix)]
#[test]
fn json_on_a_terminal_answers_one_document_and_opens_no_session() {
    let repo = Repo::seeded("json-frame");
    let task = repo.only(&["--type", "task"]);
    let adr = repo.only(&["--type", "adr"]);
    // No commands at all: if a session had opened, the child would still be
    // waiting on the terminal and this would never return.
    let seen = on_a_terminal(&repo, HOLDER, &["tui", "--json"], &[]);

    assert!(
        !seen.contains("\x1b[?1049h"),
        "a screen was opened under --json:\n{seen}"
    );
    let document = seen
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or_else(|| panic!("no document on the stream:\n{seen}"))
        .trim()
        .to_string();
    assert!(
        document.starts_with("{\"contract\":1,"),
        "the contract version comes first:\n{document}"
    );
    let ids = ids_of(&document);
    assert!(
        ids.contains(&task),
        "the task is in the document:\n{document}"
    );
    assert!(ids.contains(&adr), "and so is the ADR:\n{document}");
    assert!(
        document.contains(&format!("\"holder\":\"{HOLDER}\"")),
        "who holds what is in it too:\n{document}"
    );
    assert!(
        document.contains(TASK_TITLE),
        "with the titles the list draws:\n{document}"
    );
}

// ---------------------------------------------------------------------------
// The writing half (TASK-b50b340c0bb1)
// ---------------------------------------------------------------------------

/// A session that is opened, left alone for `quiet`, and then told to quit.
///
/// The one thing [`drive`] cannot express, and the whole of what "a session left
/// idle" means: the commands are written *after* the wait rather than before it,
/// so the reader spends that time blocked on a terminal nobody is typing at --
/// which is where a refresh loop, had this crate one, would be doing its work.
#[cfg(unix)]
fn idle(repo: &Repo, agent: &str, quiet: std::time::Duration) -> String {
    use std::io::{Read, Write};

    let (master, slave_path) = pty::open();
    let mut child = Command::new(ANK)
        .arg("tui")
        .current_dir(&repo.0)
        .env("ANK_AGENT", agent)
        .env("COLUMNS", "120")
        .env("LINES", "40")
        .env("NO_COLOR", "1")
        .stdin(pty::stdio(pty::slave(&slave_path)))
        .stdout(pty::stdio(pty::slave(&slave_path)))
        .stderr(pty::stdio(pty::slave(&slave_path)))
        .spawn()
        .expect("the binary must have been built");

    let mut reader = master
        .try_clone()
        .expect("the master side must be clonable for the drain");
    let drain = std::thread::spawn(move || {
        let mut seen = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => seen.extend_from_slice(&buf[..n]),
            }
        }
        seen
    });

    std::thread::sleep(quiet);
    let mut writer = master;
    writeln!(writer, "q").expect("the terminal must accept a command");
    writer.flush().unwrap();
    let status = child.wait().expect("the session must end");
    assert!(status.success(), "the session ended with {status}");
    drop(writer);
    String::from_utf8_lossy(&drain.join().expect("the drain must not panic")).to_string()
}

/// The claim state of one task: every ref this corpus carries by name, and the
/// record at the task's own address with its two instants masked.
///
/// Both halves matter and they answer different questions. The names say the
/// claim landed at the address a claim lands at and nowhere else; the record
/// says what landed there is the same record, field for field -- the holder, the
/// lease, the hash of the frozen criterion and the hash of the constraints.
///
/// `claimed` and `expires` are masked because they are the two fields that
/// *must* differ between two claims taken a second apart, and a comparison that
/// kept them would be asserting the clock stood still.
#[cfg(unix)]
fn claim_state(repo: &Repo, task: &str) -> String {
    let names = ref_names(repo);
    format!("{names}--\n{}", masked_record(repo, task))
}

/// The record on `refs/ank/claims/<id>`, with its instants replaced.
#[cfg(unix)]
fn masked_record(repo: &Repo, task: &str) -> String {
    let sha = String::from_utf8_lossy(
        &repo
            .git(&["rev-parse", &format!("refs/ank/claims/{task}")])
            .stdout,
    )
    .trim()
    .to_string();
    let body = String::from_utf8_lossy(&repo.git(&["cat-file", "-p", &sha]).stdout).to_string();
    body.lines()
        .map(|line| match line.split_once(':') {
            Some((key, _)) if matches!(key.trim(), "claimed" | "expires" | "completed") => {
                format!("{key}: <instant>")
            }
            _ => line.to_string(),
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// The code the verb table declares for one of a verb's refusals.
///
/// Read out of `ank_contract::COMMANDS` rather than written as a number here:
/// "the code the table declares" is the criterion's phrase, and a test carrying
/// its own copy of the number would agree with a table that moved.
fn declared(verb: &str, about: &str) -> ank_contract::ExitCode {
    ank_contract::spec_of(verb)
        .unwrap_or_else(|| panic!("{verb} is a verb of the surface"))
        .refuses
        .iter()
        .find(|r| r.when.contains(about))
        .unwrap_or_else(|| panic!("{verb} declares no refusal about {about:?}"))
        .code
}

/// A claim taken from the screen is the claim a shell takes (ADR-8bd76e8d7c4e).
///
/// The point of the whole crate, stated as the one comparison that can settle
/// it: the same task is claimed twice by the same agent, once by typing `claim`
/// into the reader and once by running `ank claim` in this suite, and the two
/// records are compared. They are equal because there is no second dispatch
/// path -- the reader spawned the verb that the suite spawned.
#[cfg(unix)]
#[test]
fn a_claim_taken_through_the_reader_is_the_ref_a_shell_claim_makes() {
    let repo = Repo::seeded("claim-ref");
    let task = repo.spare(
        "A task the reader claims",
        "Claimed twice over, once from the screen and once from a shell.",
    );

    let seen = drive(&repo, READER, &[&short_of(&task), "claim", "q"]);
    assert!(
        seen.contains(&format!("ank claim {task}")),
        "the reader ran the verb, and said which one:\n{seen}"
    );
    let from_the_screen = claim_state(&repo, &task);
    assert!(
        from_the_screen.contains(READER),
        "the record names the agent that typed it:\n{from_the_screen}"
    );
    assert!(
        from_the_screen.contains("criteria:"),
        "and the hash of the criterion it froze:\n{from_the_screen}"
    );

    // Hand it back and take it again the way a shell does.
    repo.ank(
        READER,
        &[
            "release",
            &task,
            "--reason",
            "to take it again from a shell",
        ],
    );
    repo.ank(READER, &["claim", &task]);
    let from_a_shell = claim_state(&repo, &task);

    assert_eq!(
        from_the_screen, from_a_shell,
        "the reader's claim and the shell's claim are two different records"
    );
}

/// A `done` with no proof is refused, and the refusal on the screen is the
/// CLI's: its code and the command it named as the way out (TASK-b50b340c0bb1).
///
/// The task is left exactly as it was, which is the half that matters more than
/// the message: a reader that had written the status itself would have moved the
/// file whatever the verb answered.
#[cfg(unix)]
#[test]
fn a_done_refused_for_a_missing_proof_leaves_the_task_untouched() {
    let repo = Repo::seeded("done-no-proof");
    let task = repo.only(&["--type", "task"]);
    repo.warm(HOLDER);
    let before = corpus_files(&repo);
    let names = ref_names(&repo);

    let seen = drive(&repo, HOLDER, &["f task", "", "done", "q"]);

    let code = declared("done", "no proof");
    assert_eq!(code, ank_contract::ExitCode::Proof, "the table moved");
    assert!(
        seen.contains(&format!("error[{code}]:")),
        "the code the table declares is on the screen:\n{seen}"
    );
    assert!(
        seen.contains("--proof"),
        "and the command the CLI named as the way out:\n{seen}"
    );
    assert!(
        seen.contains("ENTITIES") || seen.contains("BODY"),
        "and the session kept its shape:\n{seen}"
    );

    assert_eq!(
        before,
        corpus_files(&repo),
        "a file under .ank/ moved on a refused done"
    );
    assert_eq!(names, ref_names(&repo), "a ref was created or removed");
    // The lease may have moved -- `show` renews on the held task, which is what
    // typing that command in a shell does. What may not is the state of the
    // record: a `done` that landed would have replaced the claim with a
    // completion (ADR-6d8736c04cfa).
    let record = masked_record(&repo, &task);
    assert!(
        record.contains("expires:") && !record.contains("commit:"),
        "the claim ref carries a completion, so the done landed:\n{record}"
    );
    let found = repo.stdout(HOLDER, &["find", "--type", "task", "--json"]);
    assert!(
        found.contains("\"status\":\"in_progress\""),
        "the task moved out of in_progress:\n{found}"
    );
}

/// A screen nobody is typing at runs no command, so it renews nothing
/// (ADR-0bb7ea8991bc).
///
/// The measurement is on the refs, because the lease is the only thing an idle
/// session could move and `refs/ank/claims/<id>` is where it lives. The second
/// half of the test is what makes the first half mean anything: one renewing
/// verb is then run by hand, and the refs are asserted to have moved -- so
/// "nothing changed" is a fact about the session and not about the instrument.
#[cfg(unix)]
#[test]
fn a_session_left_idle_renews_no_claim() {
    let repo = Repo::seeded("idle");
    let task = repo.only(&["--type", "task"]);
    repo.warm(HOLDER);
    let files = corpus_files(&repo);
    let before = ank_refs(&repo);
    assert!(!before.is_empty(), "a claim is held, so there is a ref");

    // Long enough that a renewal would land on a different second, which is the
    // resolution the record is written at.
    let seen = idle(&repo, HOLDER, std::time::Duration::from_secs(3));
    assert!(seen.contains("ENTITIES"), "the session opened:\n{seen}");
    assert!(
        seen.contains(&short_of(&task)),
        "and drew the corpus:\n{seen}"
    );

    assert_eq!(
        before,
        ank_refs(&repo),
        "a screen left alone renewed a claim"
    );
    assert_eq!(files, corpus_files(&repo), "and it wrote a file");

    // The instrument reads a renewal when there is one.
    let _ = repo.stdout(HOLDER, &["show", &task, "--json"]);
    assert_ne!(
        before,
        ank_refs(&repo),
        "three seconds and a renewing verb left the refs identical, so the \
         comparison above proves nothing"
    );
}

/// All five verbs of the writing half, from a selected entity, through the
/// verbs (TASK-b50b340c0bb1).
///
/// One task carried through the loop the way a person would: claim it, log what
/// they learned, amend its scope, hand it back with a reason, take it again and
/// finish it with a proof. What is asserted is not the screen but the corpus
/// afterwards -- the entry is in the log, the glob is in the scope, the reason
/// is recorded, and the task is done with the proof that was typed.
#[cfg(unix)]
#[test]
fn every_verb_of_the_writing_half_is_reachable_from_a_selected_entity() {
    let repo = Repo::seeded("acts");
    let task = repo.spare(
        "A task the reader works",
        "Claimed, logged, amended, released and finished, all from the screen.",
    );
    let head = String::from_utf8_lossy(&repo.git(&["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_string();

    let seen = drive(
        &repo,
        READER,
        &[
            &short_of(&task),
            "claim",
            "log the glob was one directory short",
            "amend --scope src/deeper/**",
            "release the criterion measures the wrong thing",
            "q",
        ],
    );
    assert!(
        seen.contains(&format!("ank claim {task}")),
        "the verbs are named as they run:\n{seen}"
    );

    let entries = repo.stdout(READER, &["log", &task, "--json"]);
    assert!(
        entries.contains("the glob was one directory short"),
        "the entry the reader logged is in the log:\n{entries}"
    );
    assert!(
        entries.contains("the criterion measures the wrong thing"),
        "and so is the reason it was handed back:\n{entries}"
    );
    let shown = repo.stdout(READER, &["show", &task, "--json"]);
    assert!(
        shown.contains("src/deeper/**"),
        "the amended glob is on the entity:\n{shown}"
    );
    let found = repo.stdout(READER, &["find", "--type", "task", "--json"]);
    assert!(
        found.contains("\"status\":\"open\""),
        "the release put it back:\n{found}"
    );

    // And the last one, which needs the claim back.
    let seen = drive(
        &repo,
        READER,
        &[
            &short_of(&task),
            "claim",
            &format!("done commit:{head}"),
            "q",
        ],
    );
    assert!(!seen.contains("error["), "the finish was refused:\n{seen}");
    let shown = repo.stdout(READER, &["show", &task, "--json"]);
    assert!(
        shown.contains("status: done"),
        "the task is finished:\n{shown}"
    );
    assert!(
        shown.contains(&head),
        "with the proof that was typed:\n{shown}"
    );
}

// ---------------------------------------------------------------------------
// The change stream (TASK-2f7777a1fdff)
// ---------------------------------------------------------------------------
//
// The reader is told that the corpus moved instead of asking on a timer, and
// three things have to be true of that. It must reach the same screen the
// reload reaches, or the fast path and the slow path drift until only the one
// the developer runs is correct. It must ask nothing at all while nobody is
// typing and nothing is changing, or it is the refresh loop this crate spent
// TASK-b50b340c0bb1 not having. And it must renew nothing, ever: `ank show`
// renews the lease when the id is the task the caller holds, so an event that
// re-read the open entity would keep a claim alive for somebody who went home,
// which ADR-0bb7ea8991bc forbids in exactly those words.
//
// All three are facts about a running process, a real terminal and a git
// repository, so all three are asserted against one.

/// The reader's own configuration home: where the watcher would put a stream,
/// and where this suite puts one instead.
///
/// **The watcher is not run here, and that is the point.** What `ank tui`
/// consumes is a file of lines, and the lines are built by
/// `ank_contract::events`, which is the encoder `ank-daemon` writes with -- so
/// the two ends are held together by the code they share rather than by two
/// processes agreeing on a Tuesday. That the watcher writes those lines, into
/// this path, is asserted in its own suite, which is where a watcher belongs.
///
/// `XDG_CONFIG_HOME` alone, and never `HOME`: this has to move where the reader
/// looks for its stream without moving where git looks for a user's
/// configuration, and a fixture that changed the second would be testing this
/// machine's git rather than this binary.
#[cfg(unix)]
struct Home(PathBuf);

#[cfg(unix)]
impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
impl Home {
    /// A home whose stream exists and is empty: a watcher has run here, and
    /// nothing has happened yet.
    fn following(what: &str) -> Home {
        let home = Home::empty(what);
        std::fs::write(home.stream(), "").unwrap();
        home
    }

    /// A home with no stream in it at all: no watcher has ever run for this
    /// reader, which is the mode every checkout without one is in.
    fn empty(what: &str) -> Home {
        let root = scratch(what);
        std::fs::create_dir_all(root.join("ank")).unwrap();
        Home(root)
    }

    fn stream(&self) -> PathBuf {
        self.0.join("ank").join(ank_contract::events::STREAM_FILE)
    }

    /// One line of news, exactly as the watcher writes one.
    fn says(&self, corpus: &str, change: ank_contract::events::Change) {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.stream())
            .unwrap();
        file.write_all(
            ank_contract::events::Event::new(corpus, change)
                .line()
                .as_bytes(),
        )
        .unwrap();
    }
}

/// A session that can be watched while it is still running.
///
/// [`drive`] writes every command before reading anything, which is all a
/// keystroke-driven screen ever needed. A screen that repaints on its own has to
/// be observed *between* keystrokes, and often with no keystroke at all, so the
/// drain here writes into a buffer the test can read at any moment.
#[cfg(unix)]
struct Live {
    child: std::process::Child,
    writer: std::fs::File,
    seen: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
}

#[cfg(unix)]
impl Live {
    /// Opens `ank tui` on a real terminal, with whatever environment the test
    /// needs on top of the usual one.
    fn open(repo: &Repo, agent: &str, env: &[(&str, String)]) -> Live {
        use std::io::Read;

        let (master, slave_path) = pty::open();
        let mut command = Command::new(ANK);
        command
            .arg("tui")
            .current_dir(&repo.0)
            .env("ANK_AGENT", agent)
            .env("COLUMNS", "120")
            .env("LINES", "40")
            .env("NO_COLOR", "1");
        for (key, value) in env {
            command.env(key, value);
        }
        let child = command
            .stdin(pty::stdio(pty::slave(&slave_path)))
            .stdout(pty::stdio(pty::slave(&slave_path)))
            .stderr(pty::stdio(pty::slave(&slave_path)))
            .spawn()
            .expect("the binary must have been built");

        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let into = std::sync::Arc::clone(&seen);
        let mut reader = master
            .try_clone()
            .expect("the master side must be clonable for the drain");
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => return,
                    Ok(n) => into.lock().unwrap().extend_from_slice(&buf[..n]),
                }
            }
        });
        Live {
            child,
            writer: master,
            seen,
        }
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.seen.lock().unwrap()).to_string()
    }

    /// Waits for the screen to say something.
    ///
    /// Bounded and generous, on the rule the watcher's suite states: this is
    /// asserting that something happens at all, not how fast, so the wall is
    /// high enough that a loaded runner never reports the runner instead of the
    /// code.
    fn until(&self, what: &str, done: impl Fn(&str) -> bool) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while std::time::Instant::now() < deadline {
            if done(&self.text()) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        panic!("timed out waiting for {what}:\n{}", self.text());
    }

    /// The last frame drawn, once the screen has stopped moving.
    ///
    /// Settled first, because a frame read the instant a needle appeared is
    /// half a frame: the reader writes a screen in one call but a terminal
    /// hands it over in whatever pieces it likes.
    fn frame(&self) -> String {
        let mut last = String::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while std::time::Instant::now() < deadline {
            let now = self.text();
            if !now.is_empty() && now == last {
                return last_frame(&now);
            }
            last = now;
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        panic!("the screen never stopped moving:\n{last}");
    }

    fn send(&mut self, line: &str) {
        use std::io::Write;
        writeln!(self.writer, "{line}").expect("the terminal must accept a command");
        self.writer.flush().unwrap();
    }

    fn quit(mut self) {
        self.send("q");
        let status = self.child.wait().expect("the session must end");
        assert!(status.success(), "the session ended with {status}");
    }
}

/// The last whole frame of a byte stream a session wrote.
///
/// Frames are separated by the sequence that homes the cursor and clears the
/// screen, and the session's last act is to leave the alternate buffer, which is
/// chrome rather than a frame.
#[cfg(unix)]
fn last_frame(seen: &str) -> String {
    const HOME: &str = "\x1b[H\x1b[2J";
    const LEAVE: &str = "\x1b[?1049l";
    let at = seen
        .rfind(HOME)
        .expect("a session draws at least one frame");
    seen[at + HOME.len()..].replace(LEAVE, "")
}

/// A frame with the one line that names the route taken out of it.
///
/// The two routes must reach the same displayed state, and the one thing that
/// must *not* be the same is the line saying which route the screen is on: a
/// comparison that demanded byte equality there would be demanding the reader
/// lie about how it is being kept current. Everything else -- the claims, every
/// row, the counts, the note -- is compared byte for byte.
#[cfg(unix)]
fn without_the_route(frame: &str) -> String {
    frame
        .lines()
        .map(
            |line| match (line.starts_with("identity "), line.find("stream ")) {
                (true, Some(at)) => line[..at].to_string(),
                _ => line.to_string(),
            },
        )
        .collect::<Vec<String>>()
        .join("\n")
}

/// The repository identity the stream keys on, as the CLI states it.
#[cfg(unix)]
fn corpus_of(repo: &Repo) -> String {
    let doc = repo.stdout(READER, &["status", "--json"]);
    let at = doc.find("\"corpus\":\"").expect("status names the corpus");
    let rest = &doc[at + 10..];
    rest[..rest.find('"').expect("a closed string")].to_string()
}

/// Both routes, on one corpus, reaching one screen.
///
/// **Two sessions open at once, and one change.** The fast path and the slow
/// path have to be compared against the same corpus at the same state, and a
/// test that ran them one after the other would be comparing two states. So
/// both screens are opened before anything moves: one with a stream to follow
/// and one without, the corpus gains a task, and each screen catches up the way
/// it can -- the first because it was told, the second because somebody typed
/// `r`.
///
/// What is asserted is the frame, byte for byte, minus the one line that says
/// which of the two it was. That line is asserted to differ, because two routes
/// that turned out to be one route would otherwise pass this test perfectly.
#[cfg(unix)]
#[test]
fn the_event_and_the_reload_reach_the_same_displayed_state() {
    let repo = Repo::seeded("routes");
    repo.warm(READER);
    let corpus = corpus_of(&repo);
    let following = Home::following("routes-stream");
    let alone = Home::empty("routes-none");

    let told = Live::open(
        &repo,
        READER,
        &[("XDG_CONFIG_HOME".into(), following.0.display().to_string())],
    );
    told.until("the told screen to open", |t| t.contains("ENTITIES"));
    let mut asking = Live::open(
        &repo,
        READER,
        &[("XDG_CONFIG_HOME".into(), alone.0.display().to_string())],
    );
    asking.until("the asking screen to open", |t| t.contains("ENTITIES"));
    assert!(
        told.text().contains("stream following"),
        "the first screen has a stream:\n{}",
        told.text()
    );
    assert!(
        asking.text().contains("stream none"),
        "and the second has none:\n{}",
        asking.text()
    );

    let arrived = repo.spare(
        "A task that arrives while two screens are open",
        "both screens name it, and neither polled for it",
    );
    let needle = short_of(&arrived);

    // Nobody types into this one.
    following.says(&corpus, ank_contract::events::Change::Entities);
    told.until("the event to reach the screen", |t| t.contains(&needle));
    let by_event = told.frame();

    asking.send("r");
    asking.until("the reload to reach the screen", |t| t.contains(&needle));
    let by_reload = asking.frame();

    assert_eq!(
        without_the_route(&by_event),
        without_the_route(&by_reload),
        "the two routes drew two different screens"
    );
    assert_ne!(
        by_event, by_reload,
        "the two frames are identical, so the route line says nothing and this \
         test compared one route with itself"
    );

    told.quit();
    asking.quit();
}

/// A screen with a stream connected, and nobody typing, asks nothing.
///
/// **The instrument is git.** Every read this reader makes is `ank <verb>
/// --json` spawned as a child, and every one of those verbs asks git something
/// (ADR-9307e5d214a7 requires it per verb). So a shim on `PATH` that records
/// each invocation and hands the call to the real binary counts every query the
/// reader makes, whatever route it took to make one -- which is stronger than
/// counting the spawns this crate knows about, because it would also catch a
/// query made some other way.
///
/// The corpus is changed from the test process, which does not carry the shim,
/// so what the log holds is the reader's own asking and nothing else.
#[cfg(unix)]
#[test]
fn a_screen_with_the_stream_connected_asks_nothing_while_it_is_idle() {
    let repo = Repo::seeded("idle-stream");
    repo.warm(READER);
    let corpus = corpus_of(&repo);
    let home = Home::following("idle-stream-home");
    let shim = Shim::new("idle-stream-shim");

    let live = Live::open(
        &repo,
        READER,
        &[
            ("XDG_CONFIG_HOME".into(), home.0.display().to_string()),
            ("PATH".into(), shim.path()),
            ("ANK_GIT_LOG".into(), shim.log.display().to_string()),
        ],
    );
    live.until("the screen to open", |t| t.contains("ENTITIES"));
    assert!(
        live.text().contains("stream following"),
        "the stream is connected:\n{}",
        live.text()
    );
    let opened = shim.settled();
    assert!(
        opened > 0,
        "the instrument counted nothing, so it counts nothing"
    );

    // Three seconds, the length TASK-b50b340c0bb1 chose for the same reason: a
    // renewal writes at second resolution, and anything that happens here would
    // have to be visible at that scale.
    std::thread::sleep(std::time::Duration::from_secs(3));
    assert_eq!(
        shim.count(),
        opened,
        "a screen with a stream connected asked again while nobody typed"
    );

    // And the instrument reads a query when there is one: an event arrives, the
    // reader answers it by reading the corpus, and the count moves.
    let arrived = repo.spare(
        "A task that arrives while the screen is idle",
        "the screen names it without anybody typing",
    );
    home.says(&corpus, ank_contract::events::Change::Entities);
    live.until("the event to reach the screen", |t| {
        t.contains(&short_of(&arrived))
    });
    assert!(
        shim.count() > opened,
        "the reader repainted without asking the corpus anything, so the \
         comparison above proves nothing"
    );
    live.quit();
}

/// An event repaints, and renews nothing (ADR-0bb7ea8991bc).
///
/// **This is the trap the previous wave laid bare.** `ank show <id>` renews the
/// lease when the id is the task the caller holds, so a reader that answered an
/// event by re-reading the entity on screen would have made a watcher's news
/// renew somebody's claim -- a claim renewed by reporting rather than by
/// working, which is the thing that decision exists to refuse and which
/// TASK-b50b340c0bb1 already forbade an idle session to do.
///
/// So the session is put where the damage would be: the entity view, on the very
/// task this identity holds. Then events arrive for three seconds. Afterwards
/// `b` goes back to the list, which runs nothing at all -- and the list names a
/// task that did not exist when the entity was opened, which is only possible if
/// every one of those events did repaint. The refs are byte for byte where they
/// were.
#[cfg(unix)]
#[test]
fn an_event_repaints_the_list_and_renews_no_claim() {
    let repo = Repo::seeded("event-claim");
    let held = repo.only(&["--type", "task"]);
    repo.warm(HOLDER);
    let corpus = corpus_of(&repo);
    let home = Home::following("event-claim-home");

    let mut live = Live::open(
        &repo,
        HOLDER,
        &[("XDG_CONFIG_HOME".into(), home.0.display().to_string())],
    );
    live.until("the screen to open", |t| t.contains("ENTITIES"));
    // Opening the task you hold renews the lease, and it is supposed to: it is
    // `ank show`, run because a person typed an identifier (TASK-49746735127f).
    // What follows is about what happens with nobody typing.
    live.send(&short_of(&held));
    live.until("the held task to open", |t| t.contains(TAIL));
    let _ = live.frame();

    let before = ank_refs(&repo);
    assert!(!before.is_empty(), "a claim is held, so there is a ref");

    let arrived = repo.spare(
        "A task that arrives while the held one is open",
        "the list names it, and the lease did not move",
    );
    for _ in 0..6 {
        home.says(&corpus, ank_contract::events::Change::Entities);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // `b` draws the list out of what is already in hand: it reads nothing.
    live.send("b");
    live.until("the list to name the task that arrived", |t| {
        t.contains(&short_of(&arrived))
    });
    assert_eq!(
        before,
        ank_refs(&repo),
        "an event renewed a lease, which is a claim renewed by reporting"
    );

    // The instrument reads a renewal when there is one.
    let _ = repo.stdout(HOLDER, &["show", &held, "--json"]);
    assert_ne!(
        before,
        ank_refs(&repo),
        "three seconds of events and a renewing verb left the refs identical, \
         so the comparison above proves nothing"
    );
    live.quit();
}

/// A shim `git` on `PATH` that records every call and hands it to the real one.
///
/// Four symbols' worth of shell rather than a crate: what is needed is a count
/// of invocations, and the honest place to count them is where they happen.
/// The real binary is resolved once, here, so the shim cannot find itself.
#[cfg(unix)]
struct Shim {
    dir: PathBuf,
    log: PathBuf,
}

#[cfg(unix)]
impl Drop for Shim {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[cfg(unix)]
impl Shim {
    fn new(what: &str) -> Shim {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch(what);
        let real = String::from_utf8_lossy(
            &Command::new("sh")
                .args(["-c", "command -v git"])
                .output()
                .expect("a shell is a hard dependency of this suite")
                .stdout,
        )
        .trim()
        .to_string();
        assert!(!real.is_empty(), "git must be on PATH for this suite");
        let script = dir.join("git");
        std::fs::write(
            &script,
            format!("#!/bin/sh\necho call >> \"$ANK_GIT_LOG\"\nexec {real} \"$@\"\n"),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        Shim {
            log: dir.join("calls"),
            dir,
        }
    }

    fn path(&self) -> String {
        format!(
            "{}:{}",
            self.dir.display(),
            std::env::var("PATH").unwrap_or_default()
        )
    }

    fn count(&self) -> usize {
        std::fs::read_to_string(&self.log)
            .unwrap_or_default()
            .lines()
            .count()
    }

    /// The count once the reader has stopped making calls: two identical
    /// readings a moment apart.
    fn settled(&self) -> usize {
        let mut last = usize::MAX;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while std::time::Instant::now() < deadline {
            let now = self.count();
            if now > 0 && now == last {
                return now;
            }
            last = now;
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        panic!("the reader never stopped asking");
    }
}

// ---------------------------------------------------------------------------
// Ratification (TASK-d90e94afca08)
// ---------------------------------------------------------------------------
//
// The act this project guards hardest, driven from a screen. What has to be
// true of that is four things, and every one of them is a fact about a process
// and a git repository rather than about a function.
//
// The queue has to be the CLI's queue. A document ratified through the screen
// has to be verifiable exactly as one ratified in a shell is -- same entity,
// same anchor, same commit, judged the same by `check`. With the word withheld,
// nothing may move: a screen that has a proposal open and is never typed at
// leaves the queue where it found it. And where the CLI refuses -- the wrong
// branch above all -- what reaches the screen has to be the CLI's own refusal,
// with the command it named as the way out.

/// A proposed ADR of this corpus, and its identifier.
///
/// `new adr` lands `proposed` (§3), which is the state the queue is made of.
///
/// The identifier is read as the one that was not there a moment ago, rather
/// than by looking for the title. Two proposals in this suite deliberately
/// carry the *same* title -- it is what makes their two ratifications
/// comparable -- so a title is not a name here and finding one by it would find
/// whichever came first.
#[cfg(unix)]
fn proposal(repo: &Repo, title: &str) -> String {
    let before = ids_of(&repo.stdout(HOLDER, &["find", "--type", "adr", "--json"]));
    repo.ank(
        OTHER,
        &[
            "new",
            "adr",
            "--title",
            title,
            "--scope",
            "src/**",
            "--constraint",
            "A rule this suite ratifies from a screen.",
        ],
    );
    ids_of(&repo.stdout(HOLDER, &["find", "--type", "adr", "--json"]))
        .into_iter()
        .find(|id| !before.contains(id))
        .expect("the proposal exists")
}

/// The queue names what is waiting and who may sign it, and both are `review`'s
/// own answer.
///
/// The seeded corpus declares no signing key, so the second half is §8's
/// advisory sentence rather than a section with no rows -- which is the
/// distinction `review` itself insists on, carried onto the screen.
#[cfg(unix)]
#[test]
fn the_queue_names_what_is_waiting_and_the_regime_the_corpus_is_in() {
    let repo = Repo::seeded("queue");
    let waiting = proposal(&repo, "A decision waiting for a person");
    let task = repo.only(&["--type", "task"]);

    let seen = drive(&repo, READER, &["v", "q"]);
    assert!(seen.contains("QUEUE"), "the queue was drawn:\n{seen}");
    assert!(
        seen.contains(&short_of(&waiting)),
        "and it names the proposal:\n{seen}"
    );
    assert!(
        seen.contains("A decision waiting for a person"),
        "with its title:\n{seen}"
    );
    assert!(
        seen.contains("permissions are advisory"),
        "a corpus declaring no key says which regime it is in:\n{seen}"
    );
    // A task is not waiting for a signature and has no business in this queue.
    let queue = seen
        .rsplit("QUEUE")
        .next()
        .expect("the queue heading was drawn");
    assert!(
        !queue.contains(&short_of(&task)),
        "a task is in the ratification queue:\n{queue}"
    );
}

/// A document ratified through the reader is what a shell `accept` makes
/// (ADR-8bd76e8d7c4e).
///
/// The point of the whole crate, on the one act it matters most for. Two
/// proposals identical but for their identifiers: one is ratified by opening it
/// on the screen and typing the word, the other by running `ank accept` in this
/// suite. What is compared is everything a later reader would verify -- the
/// entity as `show` prints it, the ratification commit's message, and what
/// `check` says about each -- with the identifiers and the instants masked,
/// because those are what two documents must differ in.
///
/// They are equal because there is no second dispatch path: the reader spawned
/// the verb this suite spawned.
#[cfg(unix)]
#[test]
fn a_document_ratified_through_the_reader_is_what_a_shell_accept_makes() {
    let repo = Repo::seeded("ratify");
    // One title for both, so the two documents differ in nothing a
    // ratification could legitimately depend on: same slug, same scope, same
    // constraint, same author. What is left to differ is the identifier and the
    // instants, and those are masked below.
    const TITLE: &str = "A decision ratified twice over";
    let by_screen = proposal(&repo, TITLE);
    let by_shell = proposal(&repo, TITLE);

    let seen = drive(&repo, READER, &[&by_screen, "accept", "q"]);
    assert!(
        seen.contains(&format!("ank accept {by_screen}")),
        "the reader ran the verb, and said which one:\n{seen}"
    );
    assert!(
        !seen.contains("error["),
        "the ratification was refused:\n{seen}"
    );

    repo.ank(READER, &["accept", &by_shell]);

    assert_eq!(
        ratification(&repo, &by_screen),
        ratification(&repo, &by_shell),
        "the screen's ratification and the shell's are two different acts"
    );
    // And both are verifiable, which is what "ratified" is worth: `check`
    // reports no fault about either.
    let report = repo.stdout(READER, &["check", "--json"]);
    for id in [&by_screen, &by_shell] {
        assert!(
            !faulted(&report, id),
            "{id} is a fault after ratification:\n{report}"
        );
    }
}

/// Everything a later reader verifies about one ratification, with what must
/// differ between two of them masked.
///
/// The entity as `show` prints it -- which carries `status`, the anchor under
/// `ratified` and the reading recorded by the act -- and the message of the
/// commit that made it binding. Two things are replaced and no more: the
/// identifier, and every instant. Two documents are two documents and two acts
/// happen at two moments, so a comparison that kept either would be asserting
/// something false.
///
/// **The anchor is compared and never masked**, and that is the assertion doing
/// the most work here. These two proposals carry the same constraint over the
/// same scope, so the text `accept` hashes is the same text -- which means the
/// two ratifications must arrive at the same anchor, byte for byte, or one of
/// the two roads hashed something the other did not.
#[cfg(unix)]
fn ratification(repo: &Repo, id: &str) -> String {
    let shown = repo.stdout(READER, &["show", id, "--json"]);
    let message = String::from_utf8_lossy(
        &repo
            .git(&[
                "log",
                "-1",
                "--format=%B",
                "--grep",
                &format!("ratify {id}"),
            ])
            .stdout,
    )
    .to_string();
    assert!(
        message.contains("ratify"),
        "no ratification commit for {id}:\n{message}"
    );
    masked_instants(&format!("{shown}\n--\n{message}").replace(id, "<id>"))
}

/// Every RFC 3339 instant of a text, replaced.
///
/// Two acts happen at two moments, and a comparison that kept them would be
/// asserting the clock stood still -- the same reason
/// [`masked_record`] masks `claimed` and `expires`. Matched on the shape rather
/// than on the field name, because these instants are inside a JSON string
/// where there are no fields to name.
#[cfg(unix)]
fn masked_instants(text: &str) -> String {
    const SHAPE: &str = "dddd-dd-ddTdd:dd:ddZ";
    let chars: Vec<char> = text.chars().collect();
    let shape: Vec<char> = SHAPE.chars().collect();
    let mut out = String::new();
    let mut at = 0;
    while at < chars.len() {
        let fits = at + shape.len() <= chars.len()
            && shape.iter().enumerate().all(|(i, want)| {
                let got = chars[at + i];
                if *want == 'd' {
                    got.is_ascii_digit()
                } else {
                    got == *want
                }
            });
        if fits {
            out.push_str("<instant>");
            at += shape.len();
        } else {
            out.push(chars[at]);
            at += 1;
        }
    }
    out
}

/// Whether `check` reports a fault about this entity.
#[cfg(unix)]
fn faulted(report: &str, id: &str) -> bool {
    report
        .split("{\"subject\"")
        .any(|f| f.contains(id) && f.contains("\"severity\":\"fault\""))
}

/// With the word withheld, nothing in the queue changes state
/// (TASK-d90e94afca08).
///
/// The negative that matters more than the positive: a session that opens the
/// queue, opens the proposal, reads its body to the end and goes back has done
/// everything a ratification does except the one thing that is a ratification.
/// Afterwards the document is still `proposed`, the corpus is byte for byte
/// where it was, and history has not moved -- which is what "the reader may
/// drive one and never perform one" means when nobody is at the keyboard.
#[cfg(unix)]
#[test]
fn with_the_word_withheld_nothing_in_the_queue_changes_state() {
    let repo = Repo::seeded("withheld");
    let waiting = proposal(&repo, "A decision nobody ratifies");
    repo.warm(READER);
    let before = corpus_files(&repo);
    let head = String::from_utf8_lossy(&repo.git(&["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_string();

    let seen = drive(
        &repo,
        READER,
        &["v", &short_of(&waiting), "n", "n", "c", "b", "v", "q"],
    );
    assert!(
        seen.contains(&short_of(&waiting)),
        "the proposal was on the screen:\n{seen}"
    );
    assert!(
        seen.contains("accept   (this document"),
        "and the reader offered the word it never typed:\n{seen}"
    );

    let found = repo.stdout(READER, &["find", "--type", "adr", "--json"]);
    assert!(
        found.contains("\"status\":\"proposed\""),
        "the proposal left the queue with nobody typing:\n{found}"
    );
    assert_eq!(
        before,
        corpus_files(&repo),
        "a file under .ank/ moved with no word typed"
    );
    assert_eq!(
        head,
        String::from_utf8_lossy(&repo.git(&["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string(),
        "a ratification commit was made with no word typed"
    );
}

/// Off the default branch the CLI refuses, and the screen shows that refusal
/// with the command that resolves it (§12).
///
/// The code is read out of the verb table rather than written here, on the rule
/// `a_done_refused_for_a_missing_proof_leaves_the_task_untouched` already
/// follows: "the code the table declares" is the criterion's phrase, and a
/// number typed into this file would agree with a table that moved.
#[cfg(unix)]
#[test]
fn a_ratification_off_the_default_branch_shows_the_clis_refusal_and_the_way_out() {
    let repo = Repo::seeded("wrong-branch");
    let waiting = proposal(&repo, "A decision on the wrong branch");
    repo.git(&["switch", "-c", "wave7/not-the-default"]);
    repo.warm(READER);
    let before = corpus_files(&repo);

    let seen = drive(&repo, READER, &[&short_of(&waiting), "accept", "q"]);

    let code = declared("accept", "not on the default branch");
    assert_eq!(
        code,
        ank_contract::ExitCode::Prerequisite,
        "the table moved"
    );
    assert!(
        seen.contains(&format!("error[{code}]:")),
        "the code the table declares is on the screen:\n{seen}"
    );
    assert!(
        seen.contains("git switch main"),
        "and the command the CLI named as the way out:\n{seen}"
    );
    assert!(
        seen.contains("BODY") || seen.contains("ENTITIES"),
        "and the session kept its shape:\n{seen}"
    );
    assert_eq!(
        before,
        corpus_files(&repo),
        "a refused ratification moved a file"
    );
}

/// The word is refused off the document, and refused with a tail, and neither
/// refusal spawns anything (TASK-d90e94afca08).
///
/// Both are the reader's own and both are about the line that was typed, which
/// is the line this crate draws: a refusal on the state of the corpus is always
/// the CLI's, and a refusal about what somebody wrote is always this one's.
#[cfg(unix)]
#[test]
fn accept_is_refused_off_the_document_and_refused_with_a_tail() {
    let repo = Repo::seeded("accept-grammar");
    let waiting = proposal(&repo, "A decision typed at wrongly");
    repo.warm(READER);
    let before = corpus_files(&repo);

    // From the queue, where the row is under the cursor and the body is not on
    // the screen; then on the document, with something after the word.
    let seen = drive(
        &repo,
        READER,
        &["v", "accept", &short_of(&waiting), "accept ADR-0000", "q"],
    );
    assert!(
        seen.contains("open it first"),
        "a ratification off a row named the way in:\n{seen}"
    );
    assert!(
        seen.contains("takes nothing after it"),
        "a ratification carrying a tail was refused:\n{seen}"
    );
    assert!(
        !seen.contains("ank accept"),
        "one of the two reached the verb:\n{seen}"
    );
    let found = repo.stdout(READER, &["find", "--type", "adr", "--json"]);
    assert!(
        found.contains("\"status\":\"proposed\""),
        "something was ratified:\n{found}"
    );
    assert_eq!(before, corpus_files(&repo), "a file under .ank/ moved");
}
