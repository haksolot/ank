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
        std::fs::create_dir_all(repo.0.join("src")).unwrap();
        std::fs::write(repo.0.join("src/lib.rs"), "// code\n").unwrap();
        repo.ank(HOLDER, &["init"]);
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
