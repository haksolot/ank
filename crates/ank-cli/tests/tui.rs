//! `ank tui` through the binary (TASK-49746735127f, ADR-8bd76e8d7c4e,
//! ADR-c07e2694f0e1).
//!
//! CLAUDE.md leaves no choice about where this suite lives: a criterion that
//! talks about the binary is tested through the binary, and twice in this
//! repository green unit tests covered code that was right on a path the binary
//! never reached. A TUI is the extreme case of that trap -- every interesting
//! behaviour sits behind terminal setup a unit test does not perform -- so the
//! session here is driven through a real pseudo-terminal and the screen is read
//! off the master side.
//!
//! It lives in `ank-cli` rather than in `ank-tui` for one mechanical reason:
//! `CARGO_BIN_EXE_ank` is defined only for the package that declares the
//! binary, and a suite that could not name the binary would be back to testing
//! the function instead of the process.
//!
//! # What changed when the reader moved onto ratatui, and what did not
//!
//! **What is asserted did not move.** Quitting leaves no file and no ref
//! changed; a session left idle renews no claim; an event repaints and renews
//! nothing; the event route and the reload route reach the same displayed
//! state; the frames carry no identifier the corpus does not. Those are facts
//! about the reader's behaviour and not about how it draws, so they survived
//! the engine whole.
//!
//! **How the session is driven did move, twice.** A command is a key now
//! (ADR-c07e2694f0e1), so a list of lines became a list of keystrokes, with the
//! four verbs that carry a message, a reason, a proof or a flag spelled into
//! the one line `/` opens -- see [`on_a_terminal`] for what an entry of one
//! of those lists is.
//!
//! And the screen is no longer the byte stream. ratatui draws by diffing: it
//! writes the cells that changed and moves the cursor over the ones that did
//! not, so a space that stayed a space is never sent and `CLAIMS (1)` reaches
//! the wire as `CLAIMS`, a cursor move and `(1)`. A substring search over those
//! bytes would have been asserting something no longer true of them, so the
//! bytes are applied to a grid ([`Screen`]) and every assertion here is made
//! against what a person would have been looking at. That is stricter than
//! what it replaced rather than looser: a frame is compared as a frame, and
//! [`Seen::raw`] keeps the stream for the one assertion that is genuinely about
//! it.
//!
//! **What is covered on which platform.** The refusal with no terminal runs
//! everywhere, which matters most: it is the one an agent meets. The driven
//! session is `#[cfg(unix)]`, because a pseudo-terminal on Windows is ConPTY
//! and reaching it means the console API this workspace does not otherwise
//! call. What CLAUDE.md's three-platform rule asks of raw mode is answered a
//! rung down: this crate declares no foreign symbol at all
//! (`crates/ank-tui/tests/dependencies.rs`), the two implementations of it are
//! crossterm's and are exercised far beyond what this workspace could give
//! them, and what runs on all three here is everything above them -- the
//! keystroke mapping, the layout, the render, and the refusal an agent meets.
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

/// One entry of a key list that opens an entity by its identifier.
///
/// An identifier is a line by nature -- it is fourteen characters and no key
/// spells it -- so it goes through the prompt, which is where the grammar reads
/// one and jumps to it.
#[cfg(unix)]
fn open(id: &str) -> String {
    format!(":{}", short_of(id))
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

/// A pseudo-terminal, opened with the four calls POSIX names for it, and sized.
///
/// Declared here rather than taken from a crate. `libc` is in the lockfile and
/// would have been the tidier road, but it is not compiled for this target
/// today and §13 spends a dependency only on necessity: what is needed is six
/// symbols and two flags, and Rust links the platform's C library already, so
/// nothing is added to the link line either.
///
/// **The window is set here and it has to be.** The reader asks the terminal
/// how big it is (ADR-c07e2694f0e1), and a pseudo-terminal nobody sized is nought
/// by nought -- so a suite that skipped this would be asserting what a reader
/// draws into no window at all. It is also what makes the resize measurable:
/// setting it again while a session is running is exactly what a person
/// dragging the corner of a window does.
///
/// A test may declare what this crate may not, which is the rule
/// `crates/ank-tui/tests/dependencies.rs` states from the other side: the
/// reader reaches raw mode, the window and a keystroke through crossterm and
/// declares no foreign symbol at all. The `extern` below is the instrument, not
/// the subject.
#[cfg(unix)]
mod pty {
    use std::ffi::CStr;
    use std::fs::File;
    use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};
    use std::os::raw::{c_char, c_int, c_ulong};
    use std::path::PathBuf;

    extern "C" {
        fn posix_openpt(flags: c_int) -> c_int;
        fn grantpt(fd: c_int) -> c_int;
        fn unlockpt(fd: c_int) -> c_int;
        fn ptsname(fd: c_int) -> *mut c_char;
        fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
        fn kill(pid: c_int, signal: c_int) -> c_int;
    }

    const O_RDWR: c_int = 2;

    /// The two constants that are not POSIX, spelled per platform because they
    /// are per platform: an `ioctl` number encodes the direction and the size
    /// of its argument, and the two kernels encode them differently.
    #[cfg(target_os = "linux")]
    const TIOCSWINSZ: c_ulong = 0x5414;
    #[cfg(not(target_os = "linux"))]
    const TIOCSWINSZ: c_ulong = 0x8008_7467;

    /// `SIGWINCH`, which is 28 on every platform this suite runs on.
    const SIGWINCH: c_int = 28;

    #[repr(C)]
    struct WinSize {
        rows: u16,
        columns: u16,
        x_pixels: u16,
        y_pixels: u16,
    }

    /// The master side as a `File`, and the path of the slave to hand the
    /// child.
    ///
    /// The window is not set here: it is set through a slave, and the caller is
    /// what holds one open. See [`resize`].
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

    /// The window, stated on the slave side.
    ///
    /// **On the slave and never on the master, and macOS is why.** Linux takes
    /// `TIOCSWINSZ` on `/dev/ptmx` and this suite did that first; macOS does
    /// not, because there the master is a cloning device answering only the
    /// three ioctls `grantpt`, `unlockpt` and `ptsname` are built out of, and a
    /// general tty ioctl on it answers `-1`. The slave is a tty on both, which
    /// is where a window size belongs anyway: it is a property of the terminal
    /// the program is looking at.
    pub fn resize(slave_path: &PathBuf, columns: u16, rows: u16) {
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
            "the window could not be set on the pseudo-terminal: {}",
            std::io::Error::last_os_error()
        );
    }

    /// The signal a terminal sends when its window changed.
    ///
    /// **Sent by name rather than left to the kernel**, and the reason is worth
    /// stating: the kernel sends `SIGWINCH` to the foreground process group of
    /// the terminal's *session*, and this child was never given the slave as a
    /// controlling terminal -- no `setsid`, no `TIOCSCTTY`, because neither is
    /// needed for anything else here. So the delivery is done explicitly, which
    /// is the same signal arriving by a shorter road: what is under test is what
    /// the reader does when it is told the window moved, and not which of the
    /// two roads told it.
    pub fn winch(child: &std::process::Child) {
        // SAFETY: the child is alive -- it has not been waited on -- and
        // SIGWINCH is a valid signal number.
        let sent = unsafe { kill(child.id() as c_int, SIGWINCH) };
        assert_eq!(sent, 0, "SIGWINCH could not be delivered to the session");
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

/// A terminal, as far as this suite needs one.
///
/// **The reader draws by diffing now, and that is what forced this.** ratatui
/// writes the cells that changed and moves the cursor over the ones that did
/// not, so a byte stream carrying `CLAIMS (1)` may well carry `CLAIMS`, a
/// cursor move and `(1)` -- the screen is right and a substring search over the
/// bytes is wrong. So the bytes are applied to a grid, exactly as a terminal
/// applies them, and every assertion in this file is made against what a person
/// would have been looking at.
///
/// It is deliberately the smallest emulator that is honest about this stream:
/// cursor position, the two erases, and text. Everything else a CSI can say --
/// colour, attributes, the alternate buffer, the cursor's visibility -- moves
/// no character on the grid, so it is consumed and dropped rather than half
/// understood. [`Screen::raw`] keeps the bytes for the one assertion that is
/// genuinely about them.
#[cfg(unix)]
struct Screen {
    grid: Vec<Vec<char>>,
    columns: usize,
    rows: usize,
    x: usize,
    y: usize,
    /// Bytes of an escape sequence whose end has not arrived yet. A terminal
    /// hands over a frame in whatever pieces it likes, and half a `MoveTo` is
    /// not a character.
    pending: Vec<u8>,
    raw: Vec<u8>,
    /// Every screen that was displayed, in order, without the repeats.
    ///
    /// The current grid answers "what is on the screen"; this answers "was this
    /// ever on it", which is what a session driven by a list of keys is asked --
    /// a refusal a person reads and then presses past is on one frame and gone
    /// from the next.
    shown: Vec<String>,
}

#[cfg(unix)]
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
            shown: Vec::new(),
        }
    }

    /// The window changed under the session, so the grid does too.
    ///
    /// Cleared rather than kept: a terminal being resized redraws from
    /// whatever the application sends next, and a grid holding rows of the old
    /// width would let an assertion pass on a line nobody can still see.
    fn resized(&mut self, columns: u16, rows: u16) {
        self.grid = vec![vec![' '; columns as usize]; rows as usize];
        self.columns = columns as usize;
        self.rows = rows as usize;
        self.x = 0;
        self.y = 0;
    }

    fn feed(&mut self, bytes: &[u8]) {
        self.raw.extend_from_slice(bytes);
        self.pending.extend_from_slice(bytes);
        let taken = self.apply();
        self.pending.drain(..taken);
        self.snapshot();
    }

    /// The screen as it stands, kept if it is not the one already kept.
    fn snapshot(&mut self) {
        let now = self.text();
        if self.shown.last() != Some(&now) {
            self.shown.push(now);
        }
    }

    /// Applies whole sequences and characters, answering how many bytes it
    /// consumed. What is left is the tail of something unfinished.
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
                            // **A frame is kept where the frame ended**, and
                            // not where a read happened to stop. ratatui closes
                            // every `draw` by hiding the cursor or by showing
                            // it and placing it, so `?25l` and `?25h` are where
                            // one screen becomes the next -- and a suite that
                            // sampled on read boundaries instead would lose a
                            // frame whenever two of them arrived together,
                            // which under load is most of them.
                            if self.csi(&params, final_byte, private) {
                                self.snapshot();
                            }
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
                _ => {
                    // UTF-8, and a character split across two reads waits like
                    // an escape sequence does.
                    let width = utf8_width(bytes[at]);
                    if at + width > bytes.len() {
                        break;
                    }
                    match std::str::from_utf8(&bytes[at..at + width]) {
                        Ok(s) => {
                            for c in s.chars() {
                                self.put(c);
                            }
                        }
                        // Not a character: dropped rather than guessed at.
                        Err(_) => {}
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

    /// Applies one CSI, answering whether it ended a frame.
    fn csi(&mut self, params: &[usize], final_byte: u8, private: bool) -> bool {
        if private {
            // `?1049h`, `?25l` and their kind move no character. The cursor's
            // visibility is the one that says something all the same: it is
            // the last thing a `draw` writes.
            return matches!(final_byte, b'h' | b'l') && params.first() == Some(&25);
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
                // To the end of the screen, from the cursor.
                0 => {
                    for x in self.x..self.columns {
                        self.grid[self.y][x] = ' ';
                    }
                    for y in self.y + 1..self.rows {
                        self.grid[y] = vec![' '; self.columns];
                    }
                }
                // To the start of it.
                1 => {
                    for y in 0..self.y {
                        self.grid[y] = vec![' '; self.columns];
                    }
                    for x in 0..=self.x.min(self.columns - 1) {
                        self.grid[self.y][x] = ' ';
                    }
                }
                // The whole of it.
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
        false
    }

    /// What is on the screen now, one row per line, trailing space cut.
    fn text(&self) -> String {
        self.grid
            .iter()
            .map(|row| row.iter().collect::<String>().trim_end().to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Every screen that was displayed, for a session driven by a list of keys.
    fn everything(&self) -> String {
        self.shown.join("\n")
    }

    fn raw(&self) -> String {
        String::from_utf8_lossy(&self.raw).to_string()
    }
}

/// One escape sequence at the head of `bytes`: how long it is, and the CSI it
/// was if it was one.
///
/// `None` means it is not all here yet.
#[cfg(unix)]
#[allow(clippy::type_complexity)]
fn escape(bytes: &[u8]) -> Option<(usize, Option<(Vec<usize>, u8, bool)>)> {
    if bytes.len() < 2 {
        return None;
    }
    match bytes[1] {
        b'[' => {
            let mut at = 2;
            let private = bytes.get(2).is_some_and(|b| b"<=>?".contains(b));
            if private {
                at += 1;
            }
            let start = at;
            while at < bytes.len() && (bytes[at].is_ascii_digit() || bytes[at] == b';') {
                at += 1;
            }
            let final_byte = *bytes.get(at)?;
            let params = String::from_utf8_lossy(&bytes[start..at])
                .split(';')
                .map(|p| p.parse::<usize>().unwrap_or(0))
                .collect();
            Some((at + 1, Some((params, final_byte, private))))
        }
        // An OSC runs to a BEL or an ST, and says nothing about the grid.
        b']' => {
            let end = bytes.iter().position(|b| *b == 0x07).or_else(|| {
                bytes
                    .windows(2)
                    .position(|w| w == [0x1b, b'\\'])
                    .map(|i| i + 1)
            })?;
            Some((end + 1, None))
        }
        // Two-byte escapes: consumed and dropped.
        _ => Some((2, None)),
    }
}

#[cfg(unix)]
fn utf8_width(first: u8) -> usize {
    match first {
        0x00..=0x7f => 1,
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf7 => 4,
        // A continuation byte with no lead: one byte, dropped as invalid.
        _ => 1,
    }
}

/// What a driven session was shown, with the bytes kept for the one assertion
/// that is about them.
///
/// It derefs to the screens, so `seen.contains(...)` asks what this suite has
/// always asked: was this ever on the screen.
#[cfg(unix)]
struct Seen {
    screens: String,
    raw: String,
}

#[cfg(unix)]
impl std::ops::Deref for Seen {
    type Target = str;
    fn deref(&self) -> &str {
        &self.screens
    }
}

#[cfg(unix)]
impl std::fmt::Display for Seen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.screens)
    }
}

/// The window every session in this suite opens in.
#[cfg(unix)]
const WINDOW: (u16, u16) = (120, 40);

/// Runs `ank tui` on a real terminal and answers every screen it drew.
#[cfg(unix)]
fn drive(repo: &Repo, agent: &str, keys: &[&str]) -> Seen {
    on_a_terminal(repo, agent, &["tui"], keys)
}

/// The same, for a call that takes flags and ends on its own.
///
/// **What an entry of `keys` is.** Anything beginning with `:` is a line typed
/// into the prompt: the key that opens it, Control-U to take the search seed
/// off, the rest of the entry, and Enter. Anything else is the keys themselves,
/// byte for byte -- `"q"`, `"j"`, `"\r"` for Enter.
///
/// **No `:` entry names a verb any more** (TASK-1a415107fd56). Every command is
/// one key, the six that write included, so a verb is reached by [`verb`] and
/// what a line is still the shape for is a row number and an identifier. The
/// letter composes the command and shows it rather than running it, so an entry
/// that presses one is followed by [`confirm`] wherever the verb is meant to
/// land (TASK-d4a882345837).
#[cfg(unix)]
fn on_a_terminal(repo: &Repo, agent: &str, args: &[&str], keys: &[&str]) -> Seen {
    let mut live = Live::open_with(repo, agent, args, &[]);
    // **The first frame is waited for, and it is not politeness.** Bytes
    // written before the reader has taken the terminal sit in the line
    // discipline, which echoes them and holds them until a newline -- so a
    // suite that wrote straight after spawning would be measuring the terminal
    // it opened rather than the program it started.
    if !args.contains(&"--json") {
        live.until("the session to open", |t| t.contains("ank tui"));
    }
    for entry in keys {
        live.press(entry);
    }
    live.finished("a reader that quits answers 0")
}

/// A session that is opened, left alone for `quiet`, and then told to quit.
///
/// The one thing a list of keys cannot express, and the whole of what "a
/// session left idle" means: the quit is pressed *after* the wait rather than
/// before it, so the reader spends that time blocked on a terminal nobody is
/// typing at -- which is where a refresh loop, had this crate one, would be
/// doing its work.
#[cfg(unix)]
fn idle(repo: &Repo, agent: &str, quiet: std::time::Duration) -> Seen {
    let mut live = Live::open(repo, agent, &[]);
    live.until("the session to open", |t| t.contains("ank tui"));
    std::thread::sleep(quiet);
    live.press("q");
    live.finished("a reader that quits answers 0")
}

/// A session that can be watched while it is still running.
///
/// The drain feeds a [`Screen`] the test can read at any moment: what is on it
/// now, and everything that was ever on it.
#[cfg(unix)]
struct Live {
    child: std::process::Child,
    writer: std::fs::File,
    /// The slave's path, kept because the window is set through it: see
    /// [`pty::resize`] for why it cannot be set through the master.
    slave_path: PathBuf,
    screen: std::sync::Arc<std::sync::Mutex<Screen>>,
    /// The thread draining the master side. Held so that a session read after
    /// it ended is read *after* the last frame arrived: a child that has
    /// exited is not a terminal that has been read to the end, and the frame
    /// carrying whatever the last command answered is the one most likely to
    /// still be in flight.
    drain: Option<std::thread::JoinHandle<()>>,
}

#[cfg(unix)]
impl Live {
    /// Opens `ank tui` on a real terminal, with whatever environment the test
    /// needs on top of the usual one.
    fn open(repo: &Repo, agent: &str, env: &[(&str, String)]) -> Live {
        Live::open_with(repo, agent, &["tui"], env)
    }

    fn open_with(repo: &Repo, agent: &str, args: &[&str], env: &[(&str, String)]) -> Live {
        use std::io::Read;

        let (master, slave_path) = pty::open();
        // **The child's streams are opened before the window is set, and that
        // ordering is deliberate.** A pseudo-terminal whose last slave
        // descriptor closes is a terminal that hung up, and how permanently
        // depends on the platform. Holding these three from here means one is
        // always open from before the size is stated until the session ends.
        let (stdin, stdout, stderr) = (
            pty::slave(&slave_path),
            pty::slave(&slave_path),
            pty::slave(&slave_path),
        );
        pty::resize(&slave_path, WINDOW.0, WINDOW.1);
        let mut command = Command::new(ANK);
        command
            .args(args)
            .current_dir(&repo.0)
            .env("ANK_AGENT", agent)
            .env("NO_COLOR", "1");
        for (key, value) in env {
            command.env(key, value);
        }
        let child = command
            .stdin(pty::stdio(stdin))
            .stdout(pty::stdio(stdout))
            .stderr(pty::stdio(stderr))
            .spawn()
            .expect("the binary must have been built");

        let screen = std::sync::Arc::new(std::sync::Mutex::new(Screen::new(WINDOW.0, WINDOW.1)));
        let into = std::sync::Arc::clone(&screen);
        let mut reader = master
            .try_clone()
            .expect("the master side must be clonable for the drain");
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
                    Ok(n) => into.lock().unwrap().feed(&buf[..n]),
                }
            }
        });
        Live {
            child,
            writer: master,
            slave_path,
            screen,
            drain: Some(drain),
        }
    }

    /// Waits for the session to end and for the last of it to be read.
    ///
    /// The order is the whole of it: wait for the child, then drop the master
    /// so the drain stops being blocked on a terminal nobody will write to
    /// again, then join it. Reading the screen before that join is reading it
    /// while a frame is still on its way.
    fn finished(mut self, why: &str) -> Seen {
        let status = self.child.wait().expect("the session must end");
        assert!(
            status.success(),
            "the session ended with {status}, and {why}"
        );
        drop(self.writer);
        if let Some(drain) = self.drain.take() {
            drain.join().expect("the drain must not panic");
        }
        let screen = self.screen.lock().unwrap();
        Seen {
            screens: screen.everything(),
            raw: screen.raw(),
        }
    }

    /// Everything that was ever on the screen.
    fn text(&self) -> String {
        self.screen.lock().unwrap().everything()
    }

    /// Waits for the screen as it is now to say something.
    ///
    /// Different from [`Live::until`] in the one way that matters to a resize:
    /// that one asks whether this was *ever* on the screen, and a reflow is a
    /// question about what is on it now.
    fn until_screen(&self, what: &str, done: impl Fn(&str) -> bool) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while std::time::Instant::now() < deadline {
            if done(&self.screen.lock().unwrap().text()) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        panic!(
            "timed out waiting for {what}:\n{}",
            self.screen.lock().unwrap().text()
        );
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

    /// The screen as it is, once it has stopped moving.
    ///
    /// Settled first, because a screen read the instant a needle appeared is
    /// half a screen: the reader writes a frame in one call but a terminal
    /// hands it over in whatever pieces it likes.
    fn frame(&self) -> String {
        let mut last = String::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while std::time::Instant::now() < deadline {
            let now = self.screen.lock().unwrap().text();
            if !now.trim().is_empty() && now == last {
                return now;
            }
            last = now;
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        panic!("the screen never stopped moving:\n{last}");
    }

    /// One entry of a key list: a line typed into the prompt, or the keys
    /// themselves.
    fn press(&mut self, entry: &str) {
        match entry.strip_prefix(':') {
            Some(line) => {
                self.send(&FIND.to_string());
                self.send(CLEAR);
                self.send(line);
                self.send("\r");
            }
            None => self.send(entry),
        }
    }

    fn send(&mut self, bytes: &str) {
        use std::io::Write;
        self.writer
            .write_all(bytes.as_bytes())
            .expect("the terminal must accept a keystroke");
        self.writer.flush().unwrap();
    }

    /// The window moved, and the session was told.
    fn resize(&mut self, columns: u16, rows: u16) {
        pty::resize(&self.slave_path, columns, rows);
        self.screen.lock().unwrap().resized(columns, rows);
        pty::winch(&self.child);
    }

    fn quit(mut self) {
        self.press("q");
        let _ = self.finished("a reader that quits answers 0");
    }
}

/// The key that opens the one line this reader still takes, read out of the
/// reader rather than typed as a letter here: a suite carrying its own copy of
/// it would agree with a mapping that moved.
///
/// It opens a *search*, seeded with a slash (TASK-1a415107fd56). The prompt a
/// verb used to be spelled into is gone, and the six verbs are letters now, so
/// what a `:` entry of a key list means is "clear the seed and type the line",
/// which is what [`Live::press`] does.
#[cfg(unix)]
const FIND: char = ank_tui::keys::FIND;

/// The byte a terminal sends for Control-U, which clears the open line and
/// leaves the prompt open (`ank_tui::keys::edit`).
///
/// How a line that is not a search is reached: the seed is a slash, and this
/// takes it off. Not Backspace, which closes the prompt on the keystroke after
/// the line empties -- one key with one meaning is what a suite should send.
#[cfg(unix)]
const CLEAR: &str = "\u{15}";

/// The letter one verb of the writing half is bound to, out of the reader's own
/// table (TASK-1a415107fd56).
///
/// Never spelled here. The point of the wave is that a key *is* the verb, and a
/// suite typing `c` because that is what claim happens to be bound to today
/// would go on passing against a table that moved the letter.
#[cfg(unix)]
fn verb(name: &str) -> String {
    let binding = ank_tui::bindings::of_verb(name)
        .unwrap_or_else(|| panic!("'{name}' is a verb of the writing half"));
    // The reader's own spelling of the key, which is the character itself
    // where there is one: a suite must send a keystroke and not a name.
    let letter = ank_tui::bindings::named(binding.key);
    assert_eq!(
        letter.chars().count(),
        1,
        "the key is named '{letter}', which is not one keystroke to send"
    );
    letter
}

/// The key that answers the confirmation every write now passes through
/// (TASK-d4a882345837), read out of the reader for the reason [`FIND`] is.
///
/// **Every entry of a key list that presses one of the six is followed by this
/// one**, and that is the shape of the reader rather than a wrinkle of the
/// suite: a letter composes the `argv` and shows it, and nothing is spawned
/// until a person says yes to what they were shown. A drive that stopped at the
/// letter would now be driving a reader that had been asked for a write and
/// given none -- which `crates/ank-tui/tests/confirmation.rs` asserts on
/// purpose, and which every test here that expects a verb to have landed must
/// not do by accident.
#[cfg(unix)]
fn confirm() -> String {
    ank_tui::keys::CONFIRM.to_string()
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
    let seen = drive(&repo, HOLDER, &[":filter task", "\r", "b", ":filter", "q"]);

    // The one assertion here that is genuinely about the bytes rather than
    // about the screen: what a full-screen reader owes the shell it was
    // launched from is the scrollback it was covering.
    assert!(
        seen.raw.contains("\x1b[?1049h") && seen.raw.contains("\x1b[?1049l"),
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

/// Every short identifier a frame carries, in the order they are drawn.
///
/// A short identifier is `<KIND>-xxxx`, and the kinds are the four this corpus
/// has (ADR-c9f9d1a05b23).
///
/// **Both ends of the window are taken in characters and never in bytes**
/// (ADR-c07e2694f0e1). The frames carry box-drawing glyphs now and a glyph is
/// three bytes, so the two byte offsets this used to take are both a slice
/// through a code point -- which is a panic and not a failure. `rfind`
/// answers the byte index a character *starts* at, so the old `i + 1` landed
/// inside a border drawn hard against a kind; `at + 5` landed inside one drawn
/// hard against an identifier the reader had cut. The left boundary therefore
/// steps over the whole character it found, and the right one counts the four
/// characters after the hyphen rather than four bytes.
fn identifiers_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find('-') {
        let start = rest[..at]
            .char_indices()
            .rev()
            .find(|(_, c)| !c.is_ascii_alphabetic())
            .map_or(0, |(i, c)| i + c.len_utf8());
        let kind = &rest[start..at];
        let tail: String = rest[at + 1..].chars().take(4).collect();
        if ["ADR", "SPEC", "TASK", "LOG"].contains(&kind)
            && tail.chars().count() == 4
            && tail.chars().all(|c| c.is_ascii_hexdigit())
        {
            out.push(format!("{kind}-{tail}"));
        }
        rest = &rest[at + 1..];
    }
    out
}

/// The scan reads a frame carrying a box-drawing glyph beside a truncated
/// identifier, at either end of its slice (TASK-e900637aeac4).
///
/// Stated on the shapes rather than left to whichever ones a driven session
/// happened to draw: a border pressed against a kind, a border pressed against
/// an identifier the reader cut, and a hyphen with fewer than four characters
/// left before the border. Each of the three panicked before the repair, and a
/// panic in a suite is a crash with no verdict rather than a failure with one.
///
/// Not `#[cfg(unix)]`, unlike the session below it: this is arithmetic on a
/// string and it is worth running on all three platforms.
#[test]
fn the_scan_reads_an_identifier_pressed_against_a_box_drawing_glyph() {
    // The left boundary: the character before the kind is three bytes wide.
    assert_eq!(
        identifiers_in("\u{2503}TASK-4974\u{2503}"),
        ["TASK-4974"],
        "a border drawn hard against a kind was not read past"
    );
    // The right: a cut identifier, then the border that follows it.
    assert_eq!(
        identifiers_in("\u{2502}  ADR-8bd7~\u{2502}  LOG-e053\u{2502}"),
        ["ADR-8bd7", "LOG-e053"]
    );
    // And a hyphen with less than a short identifier left after it, which is
    // where `at + 5` used to land inside the glyph rather than past it.
    assert_eq!(identifiers_in("SPEC-fe\u{2503}"), Vec::<String>::new());
    assert_eq!(
        identifiers_in("\u{256d}\u{2500}TASK-49\u{2501}"),
        Vec::<String>::new()
    );
    // Nothing the corpus names is invented out of ordinary prose.
    assert_eq!(
        identifiers_in("claude-code/opus-5+glyphs   until 2026-08-25T04:36:32Z"),
        Vec::<String>::new()
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
    let seen = drive(&repo, HOLDER, &["\r", "b", "j", "\r", "b", "q"]);

    let named = identifiers_in(&seen.to_string());
    for candidate in &named {
        assert!(
            real.iter().any(|r| r == candidate),
            "the frames name {candidate}, which the corpus does not carry: {real:?}"
        );
    }
    assert!(named.len() >= 2, "the frames named no identifier at all");
}

/// A terminal made narrower, then wider, redraws to its new size with nobody
/// typing (TASK-4fa385c1772d, ADR-c07e2694f0e1).
///
/// **The whole of the assertion is that no key was pressed.** The window is
/// changed on the master side, exactly as a terminal emulator changes it, and
/// the session is told the way a terminal tells one. What has to follow is a
/// frame in the new shape: a row whose title no longer fits is cut and says so,
/// no line runs past the new right edge, and the trailer sits on the last row
/// of the new height rather than off the bottom of it. Widened again, the title
/// comes back whole -- so what happened was a fit and never a loss.
///
/// Measured through the binary, on a real pseudo-terminal, because the window
/// is an `ioctl` and a signal: the reader asks crossterm for the size and
/// crossterm asks the kernel, and no unit test stands anywhere near that.
#[cfg(unix)]
#[test]
fn a_terminal_resized_redraws_to_its_new_size_with_nothing_typed() {
    // Wide enough that sixty columns cannot hold it once a row has spent
    // thirty-five on its number, its identifier and its status, and narrow
    // enough that a hundred and twenty can.
    const LONG: &str = "A title a narrow window must cut and a wide one need not";
    let repo = Repo::seeded("resize");
    repo.spare(
        LONG,
        "The row carrying it is fitted to whatever window there is.",
    );
    repo.warm(READER);
    let before = corpus_files(&repo);

    let mut live = Live::open(&repo, READER, &[]);
    live.until("the session to open", |t| t.contains("ank tui"));
    live.until_screen("the wide frame to carry the title whole", |s| {
        s.contains(LONG)
    });
    let wide = live.frame();
    assert_eq!(wide.lines().count(), WINDOW.1 as usize, "{wide}");

    live.resize(60, 20);
    live.until_screen("the narrow frame", |s| {
        s.contains("ank tui") && !s.contains(LONG)
    });
    let narrow = live.frame();
    for line in narrow.lines() {
        assert!(
            line.chars().count() <= 60,
            "{} columns in a 60 column window: {line}\n{narrow}",
            line.chars().count()
        );
    }
    assert!(
        narrow.contains('~'),
        "nothing was cut, so nothing was fitted to the narrower window:\n{narrow}"
    );
    // The trailer is on the last row there is, which is what says the layout
    // used the new height rather than the old one.
    let rows: Vec<&str> = narrow.lines().collect();
    assert_eq!(rows.len(), 20, "{narrow}");
    // The trailer's own first entry, read out of the reader rather than spelled
    // here: it was `a then` for as long as a verb was spelled into a prompt,
    // and TASK-1a415107fd56 made it the first verb's letter.
    let trailer = ank_tui::bindings::write_line();
    let first = trailer
        .split("  ")
        .next()
        .expect("the trailer names a verb");
    assert!(
        rows[19].starts_with(first),
        "the key line is not on the last row of the new window:\n{narrow}"
    );

    // Wider again, and nothing was lost: the title is whole.
    live.resize(140, 44);
    live.until_screen("the wide frame again", |s| s.contains(LONG));
    let again = live.frame();
    assert_eq!(again.lines().count(), 44, "{again}");
    assert!(again.contains("ENTITIES"), "{again}");

    // And a window being dragged about reads nothing and writes nothing: a
    // resize is a fact about the terminal, never about the corpus.
    assert_eq!(
        before,
        corpus_files(&repo),
        "a resize moved a file under .ank/"
    );
    live.quit();
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
        &[
            // Space pages and `s` opens the constraints: `n` and `c` went to
            // the verbs' side of the ledger (TASK-1a415107fd56).
            ":filter adr",
            "\r",
            " ",
            "s",
            "g",
            "b",
            ":filter",
            "/task\r",
            "j",
            "k",
            "q",
        ],
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
    let seen = drive(&repo, HOLDER, &[":filter task", "\r", "q"]);
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
    let seen = drive(&repo, HOLDER, &[":LOG-000000000000", "q"]);
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
        !seen.raw.contains("\x1b[?1049h"),
        "a screen was opened under --json:\n{}",
        seen.raw
    );
    // The bytes and not the screen: `--json` opens no session, so nothing draws
    // and there is no screen to read. A document written to a terminal is a
    // line as long as it is, and reading it off a 120 column grid would be
    // reading it wrapped.
    let document = seen
        .raw
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or_else(|| panic!("no document on the stream:\n{}", seen.raw))
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

    let seen = drive(
        &repo,
        READER,
        &[&open(&task), &verb("claim"), &confirm(), "q"],
    );
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

    let seen = drive(
        &repo,
        HOLDER,
        &[":filter task", "\r", &verb("done"), &confirm(), "q"],
    );

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

/// All five verbs of the writing half, from a selected entity, each by its own
/// letter (TASK-b50b340c0bb1, TASK-1a415107fd56).
///
/// **What this test measures moved when the letters arrived, and it moved onto
/// firmer ground.** It used to carry one task around the loop by spelling a
/// tail after each word -- a message, a glob, a reason, a proof -- and to
/// assert the corpus afterwards. There is no line to spell a tail on now
/// (ADR-c07e2694f0e1: input is a keystroke), so a press composes the verb and
/// the identifier and nothing else, and three of the five reach a verb that
/// wants something the reader cannot yet give it.
///
/// So what is asserted is what the reader is answerable for either way: each
/// letter reaches the CLI with the entity the panel names, and the answer on
/// the screen is the binary's own. `claim` lands, `log` reads the log the way
/// `ank log <id>` does at a shell, and `amend`, `release` and `done` come back
/// with the refusal §4 declares and the command that resolves it -- which is
/// exactly what a person typing the bare verb in a shell would meet.
/// TASK-e8da6a00564a is where the tails come back, as a form.
#[cfg(unix)]
#[test]
fn every_verb_of_the_writing_half_is_reachable_from_a_selected_entity() {
    let repo = Repo::seeded("acts");
    let task = repo.spare(
        "A task the reader works",
        "Claimed, logged, amended, released and finished, all from the screen.",
    );

    let seen = drive(
        &repo,
        READER,
        &[
            &open(&task),
            &verb("claim"),
            &confirm(),
            &verb("log"),
            &confirm(),
            &verb("amend"),
            &confirm(),
            &verb("release"),
            &confirm(),
            &verb("done"),
            &confirm(),
            "q",
        ],
    );
    // Every one of the five was spelled against the entity the panel named,
    // and spelled whole: the verb, the identifier, and `--json`.
    for name in ["claim", "log", "amend", "release", "done"] {
        assert!(
            seen.contains(&format!("ank {name} {task} --json")),
            "'{name}' was not composed against the open entity:\n{seen}"
        );
    }
    // `claim` landed, which is what keeps the rest of this from being a test
    // of a reader that reaches nothing.
    let record = masked_record(&repo, &task);
    assert!(
        record.contains(READER),
        "the claim the screen took is not on the ref:\n{record}"
    );

    // And the three that want a tail came back with the binary's own refusal
    // and the way out, rather than with anything this crate wrote.
    for (name, refusal) in [
        ("amend", "nothing to amend"),
        ("release", "--reason is required"),
        ("done", "--proof"),
    ] {
        assert!(
            seen.contains(refusal),
            "'{name}' did not answer with the CLI's own refusal:\n{seen}"
        );
    }
    let found = repo.stdout(READER, &["find", "--type", "task", "--json"]);
    assert!(
        found.contains("\"status\":\"in_progress\""),
        "a verb the CLI refused moved the task anyway:\n{found}"
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

    // The reload key, out of the reader's own table: it was `r` until `release`
    // took the letter (TASK-1a415107fd56).
    asking.press(&ank_tui::bindings::spelling_of(
        &ank_tui::input::Command::Reload,
    ));
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
    live.press(&open(&held));
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
    live.press("b");
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

    let seen = drive(
        &repo,
        READER,
        &[&format!(":{by_screen}"), &verb("accept"), &confirm(), "q"],
    );
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
        &["v", &open(&waiting), "n", "n", "c", "b", "v", "q"],
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

    let seen = drive(
        &repo,
        READER,
        &[&open(&waiting), &verb("accept"), &confirm(), "q"],
    );

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

/// The letter is refused off the document, and on the document it carries the
/// document and nothing else (TASK-d90e94afca08, TASK-1a415107fd56).
///
/// **The second half is stronger than the refusal it replaces.** `accept` used
/// to be refused when a tail was typed after it, which was the grammar saying
/// no to something a person could write. There is no line to write it on now,
/// so "nothing beyond the single document" is held by there being no shape in
/// which a second argument could travel: what the key composes is the verb, the
/// identifier the body panel names, and `--json`. That is asserted here as the
/// whole of the composed line rather than as a sentence about what was
/// refused.
///
/// The first half is unchanged and is still the reader's own: a refusal on the
/// state of the corpus is always the CLI's, and a proposal binds nobody until
/// somebody reads it, so a ratification driven off a row that merely names the
/// document names the way in instead.
#[cfg(unix)]
#[test]
fn accept_is_refused_off_the_document_and_carries_nothing_but_it() {
    let repo = Repo::seeded("accept-grammar");
    let waiting = proposal(&repo, "A decision typed at wrongly");
    repo.warm(READER);
    let before = corpus_files(&repo);

    // From the queue, where the row is under the cursor and the body is not on
    // the screen; then on the document, and dismissed rather than answered.
    //
    // Dismissed with a letter and not with Escape, and that is about this
    // terminal rather than about this reader: an escape byte with another key
    // straight behind it is an *Alt chord* to any terminal decoder, and a suite
    // that writes a key list in one breath produces exactly that. `b` reaches
    // the same place, because over a command waiting every key but the one
    // dismisses (TASK-d4a882345837).
    let seen = drive(
        &repo,
        READER,
        &[
            "v",
            &verb("accept"),
            &open(&waiting),
            &verb("accept"),
            "b",
            "q",
        ],
    );
    assert!(
        seen.contains("open it into the body"),
        "a ratification off a row named the way in:\n{seen}"
    );
    assert!(
        seen.contains(&format!("ank accept {waiting} --json")),
        "the document was not what the letter composed:\n{seen}"
    );
    let found = repo.stdout(READER, &["find", "--type", "adr", "--json"]);
    assert!(
        found.contains("\"status\":\"proposed\""),
        "something was ratified:\n{found}"
    );
    assert_eq!(before, corpus_files(&repo), "a file under .ank/ moved");
}
