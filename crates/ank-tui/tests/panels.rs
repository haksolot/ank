//! The panels, through the binary, at the two widths the criterion names
//! (TASK-bb43cfe2192b).
//!
//! CLAUDE.md leaves no choice about where the measurement happens: a criterion
//! that talks about the binary is tested through the binary, and "a frame never
//! overflows the window at eighty columns and at forty" is a claim about a
//! process reading a real terminal's size. `src/view.rs` asserts the same
//! property of the layout function, at more sizes and on every platform; this
//! asserts it of `ank tui`, on a pseudo-terminal, with the window stated the
//! way a terminal emulator states one.
//!
//! **Why the harness is here and not shared.** `crates/ank-cli/tests/tui.rs`
//! drives a session the same way, and this file cannot call into it: a Rust
//! integration test is its own crate, and two of them share nothing that is not
//! in a library. Putting a pseudo-terminal in `ank-tui`'s own `src/` is not an
//! option either -- `tests/dependencies.rs` forbids this crate a foreign symbol
//! and forbids it `unsafe`, which is the whole of what ADR-0b55983421dd bought
//! by taking crossterm. A test may name what the crate may not, and that is
//! exactly the exemption `sources()` there is written to give. So what is
//! duplicated is the smallest terminal that can answer this question, and
//! nothing above it.
//!
//! **Why the binary is found rather than named.** `CARGO_BIN_EXE_ank` is
//! defined only for the package that declares the binary, and that is
//! `ank-cli`. So it is looked for beside the test executable, which is where
//! cargo puts it, and the assertion when it is missing names the command that
//! builds it rather than passing on nothing.
//!
//! The driven session is `#[cfg(unix)]`, for the reason `ank-cli`'s suite
//! gives: a pseudo-terminal on Windows is ConPTY, and reaching it means the
//! console API this workspace does not otherwise call. What runs on all three
//! platforms is the layout, the keystroke mapping and the render, in
//! `src/view.rs`.

#![cfg(unix)]

use std::path::PathBuf;
use std::process::{Command, Output};

// ---------------------------------------------------------------------------
// The binary, and a corpus for it to read
// ---------------------------------------------------------------------------

/// The `ank` this workspace just built.
///
/// Beside the test executable's own directory: cargo puts an integration test
/// in `<target>/<profile>/deps/` and a binary in `<target>/<profile>/`.
fn ank() -> PathBuf {
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
struct Repo(PathBuf);

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const AGENT: &str = "claude-code/opus-5+panel-suite";
/// Deliberately wider than the entities panel at either window, so that a
/// frame which overflowed rather than fitted would be visible as a row past
/// the right edge.
const TASK_TITLE: &str =
    "A task whose title is wider than any panel this reader draws at forty columns";

impl Repo {
    fn seeded() -> Repo {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "ank-tui-panels-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let repo = Repo(root);
        repo.git(&["init", "--initial-branch=main"]);
        repo.git(&["config", "user.email", "suite@example.invalid"]);
        repo.git(&["config", "user.name", "The Suite"]);
        // Signing off: a throwaway corpus inherits whatever the machine has
        // globally otherwise, and a suite whose commits are signed on one
        // machine and not the next is a suite that reports the machine.
        repo.git(&["config", "commit.gpgsign", "false"]);
        std::fs::create_dir_all(repo.0.join("src")).unwrap();
        std::fs::write(repo.0.join("src/lib.rs"), "// code\n").unwrap();
        repo.ank(&["init"]);
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

    /// The short form of the task this corpus carries, which is what a listing
    /// prints and what the prompt takes.
    fn task(&self) -> String {
        let doc = String::from_utf8_lossy(&self.ank(&["find", "--type", "task", "--json"]).stdout)
            .to_string();
        let at = doc.find("\"id\":\"").expect("the corpus carries a task") + 6;
        let rest = &doc[at..];
        let id = &rest[..rest.find('"').expect("an id is a closed string")];
        let (kind, hex) = id.split_once('-').expect("an identifier has a kind");
        format!("{kind}-{}", &hex[..4])
    }

    fn ank(&self, args: &[&str]) -> Output {
        let out = Command::new(ank())
            .args(args)
            .current_dir(&self.0)
            .env("ANK_AGENT", AGENT)
            .env("NO_COLOR", "1")
            .output()
            .expect("the binary must have been built");
        assert!(
            out.status.success(),
            "ank {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }
}

// ---------------------------------------------------------------------------
// The terminal
// ---------------------------------------------------------------------------

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
        fn ptsname(fd: c_int) -> *mut c_char;
        fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    }

    const O_RDWR: c_int = 2;

    /// Not POSIX, and spelled per platform because it is per platform: an
    /// `ioctl` number encodes the direction and the size of its argument, and
    /// the two kernels encode them differently.
    #[cfg(target_os = "linux")]
    const TIOCSWINSZ: c_ulong = 0x5414;
    #[cfg(not(target_os = "linux"))]
    const TIOCSWINSZ: c_ulong = 0x8008_7467;

    #[repr(C)]
    struct WinSize {
        rows: u16,
        columns: u16,
        x_pixels: u16,
        y_pixels: u16,
    }

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
        }
    }

    fn feed(&mut self, bytes: &[u8]) {
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
// A driven session
// ---------------------------------------------------------------------------

/// `ank tui` on a real terminal of a stated size, drivable while it runs.
struct Live {
    child: std::process::Child,
    writer: std::fs::File,
    screen: std::sync::Arc<std::sync::Mutex<Screen>>,
    drain: Option<std::thread::JoinHandle<()>>,
}

impl Live {
    fn open(repo: &Repo, columns: u16, rows: u16) -> Live {
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
        let child = Command::new(ank())
            .arg("tui")
            .current_dir(&repo.0)
            .env("ANK_AGENT", AGENT)
            .env("NO_COLOR", "1")
            .stdin(pty::stdio(stdin))
            .stdout(pty::stdio(stdout))
            .stderr(pty::stdio(stderr))
            .spawn()
            .expect("the binary must have been built");

        let screen = std::sync::Arc::new(std::sync::Mutex::new(Screen::new(columns, rows)));
        let into = std::sync::Arc::clone(&screen);
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
                    Ok(n) => into.lock().unwrap().feed(&buf[..n]),
                }
            }
        });
        Live {
            child,
            writer: master,
            screen,
            drain: Some(drain),
        }
    }

    fn until(&self, what: &str, done: impl Fn(&str) -> bool) {
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

    fn send(&mut self, bytes: &str) {
        use std::io::Write;
        self.writer
            .write_all(bytes.as_bytes())
            .expect("the terminal must accept a keystroke");
        self.writer.flush().unwrap();
    }

    fn quit(mut self) {
        self.send("q");
        let status = self.child.wait().expect("the session must end");
        assert!(status.success(), "a reader that quits answers 0: {status}");
        drop(self.writer);
        if let Some(drain) = self.drain.take() {
            drain.join().expect("the drain must not panic");
        }
    }
}

// ---------------------------------------------------------------------------
// The criterion
// ---------------------------------------------------------------------------

/// The two windows the criterion names.
const WINDOWS: [(u16, u16); 2] = [(80, 24), (40, 24)];

/// The key that opens the prompt, read out of the reader rather than typed as a
/// letter here: it is the one key this suite has to know, and a suite carrying
/// its own copy of it would agree with a mapping that moved.
const ACT: char = ank_tui::keys::ACT;

/// A frame never overflows the window the terminal reported, at eighty columns
/// and at forty (TASK-bb43cfe2192b).
///
/// Both directions, because a frame overflows in two ways and only one of them
/// is visible in a screenshot: a row wider than the window, and more rows than
/// the window has. The trailer being on the last row is the third half of it --
/// it is what says the layout used the size the terminal gave rather than a
/// default it was born with.
#[test]
fn no_frame_overflows_the_window_at_eighty_columns_or_at_forty() {
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let live = Live::open(&repo, columns, rows);
        live.until("the session to open", |t| t.contains("ank tui"));
        let frame = live.frame();
        let lines: Vec<&str> = frame.lines().collect();
        assert_eq!(
            lines.len(),
            rows as usize,
            "the frame is {} rows in a {columns}x{rows} window:\n{frame}",
            lines.len()
        );
        for line in &lines {
            assert!(
                line.chars().count() <= columns as usize,
                "{} columns in a {columns} column window: {line}\n{frame}",
                line.chars().count()
            );
        }
        assert!(
            lines[rows as usize - 1].starts_with("a then"),
            "the key line is not on the last row of a {columns}x{rows} window:\n{frame}"
        );
        live.quit();
    }
}

/// The screen is panels drawn side by side, one of them focused, and the
/// focused one is told apart with no colour (TASK-bb43cfe2192b).
///
/// `NO_COLOR` is set on the child, so what reaches the terminal carries no
/// palette at all and every difference this test can see is a character. That
/// is the criterion's "without colour", measured where it is true or not: on
/// the wire.
#[test]
fn the_panels_are_side_by_side_and_the_focused_one_is_marked_in_characters() {
    let repo = Repo::seeded();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows);
        live.until("the session to open", |t| t.contains("2 ENTITIES"));
        let frame = live.frame();
        for panel in ["1 CLAIMS", "2 ENTITIES", "3 BODY", "4 QUEUE"] {
            assert!(
                frame.contains(panel),
                "{panel} is not on a {columns}x{rows} frame:\n{frame}"
            );
        }
        // A row carrying two vertical borders is a row two panels share.
        let shared = frame
            .lines()
            .filter(|l| l.chars().filter(|c| *c == '|').count() >= 4)
            .count();
        assert!(
            shared >= 4,
            "no row of a {columns}x{rows} frame carries two panels:\n{frame}"
        );
        // The session opens on the entities, and the mark is there and nowhere
        // else.
        assert!(
            frame.contains("> 2 ENTITIES"),
            "the focused panel is not marked:\n{frame}"
        );
        for other in ["> 1 CLAIMS", "> 3 BODY", "> 4 QUEUE"] {
            assert!(
                !frame.contains(other),
                "two panels are marked at once:\n{frame}"
            );
        }
        // And the doubled rule, which is the second signal and also a
        // character. Both border sets are on the frame, so this is a
        // difference and not a style everything shares.
        assert!(
            frame.contains("=========="),
            "no panel is drawn with the doubled border:\n{frame}"
        );
        assert!(
            frame.contains("----------"),
            "every panel is drawn as the focused one:\n{frame}"
        );

        // Focus moves by key, and the mark moves with it.
        live.send("\t");
        live.until("the focus to move to the body", |t| t.contains("> 3 BODY"));
        let moved = live.frame();
        assert!(
            !moved.contains("> 2 ENTITIES"),
            "the mark stayed where it was:\n{moved}"
        );
        // A digit reaches one directly, which is what the number in a title is
        // for.
        live.send("1");
        live.until("the focus to reach the claims", |t| {
            t.contains("> 1 CLAIMS")
        });
        live.quit();
    }
}

/// The body of a selected entity is served whole rather than cut, at either
/// window (TASK-bb43cfe2192b).
///
/// Opened by pressing Enter on the row a session opens on, which is what a
/// person does. What is asserted is the end of a criterion wider than the
/// panel: it reaches the screen only if the reader wrapped rather than cut.
#[test]
fn the_body_of_a_selected_entity_is_served_whole() {
    let repo = Repo::seeded();
    let task = repo.task();
    for (columns, rows) in WINDOWS {
        let mut live = Live::open(&repo, columns, rows);
        live.until("the session to open", |t| t.contains("2 ENTITIES"));
        // Opened by identifier rather than by counting rows: an identifier is a
        // line by nature and the prompt is where the grammar reads one, so this
        // says which entity is meant instead of depending on where `find`
        // happens to have put it.
        live.send(&format!("{ACT}{task}\r"));
        live.until("the document to open in the body panel", |t| {
            t.contains("> 3 BODY")
        });
        // Paged until the criterion's end arrives, which is what "whole" means
        // on a panel shorter than the document.
        let mut found = false;
        for _ in 0..12 {
            if live.frame().contains("arrives whole") {
                found = true;
                break;
            }
            live.send("n");
        }
        let frame = live.frame();
        assert!(found, "the body was cut at {columns}x{rows}:\n{frame}");
        // And it is still a frame that fits the window it was given.
        for line in frame.lines() {
            assert!(
                line.chars().count() <= columns as usize,
                "{} columns in a {columns} column window: {line}\n{frame}",
                line.chars().count()
            );
        }
        live.quit();
    }
}
