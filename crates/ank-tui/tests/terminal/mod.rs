//! A terminal, a corpus and a driven `ank tui`, shared by the suites that need
//! all three.
//!
//! **Why it is here rather than in `src/`.** A Rust integration test is its own
//! crate, so two of them share nothing that is not in a library -- and a
//! pseudo-terminal in `ank-tui`'s own `src/` is not an option:
//! `tests/dependencies.rs` forbids this crate a foreign symbol and forbids it
//! `unsafe`, which is the whole of what ADR-0b55983421dd bought by taking
//! crossterm. A test may name what the crate may not, and that file's
//! `sources()` reads only `src/`, which is exactly the exemption it is written
//! to give. So the smallest terminal that can answer these questions lives in a
//! module both suites declare, and nothing above it is duplicated at all.
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

// A shared test module is compiled into every suite that declares it, and each
// of them uses the part it needs. What one suite does not reach is not dead
// code; it is code the other suite is using.
#![allow(dead_code)]

use std::path::PathBuf;
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
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub const AGENT: &str = "claude-code/opus-5+panel-suite";

/// The terminal every session but [`Live::dumb`] is opened on.
///
/// Stated rather than inherited: this suite runs under whatever the developer
/// has exported, and a reader that reads `TERM` -- which this one does, for
/// its palette and for its border set alike -- would otherwise draw one frame
/// on one machine and another on the next.
pub const TERM: &str = "xterm-256color";

/// Deliberately wider than the entities panel at either window, so that a
/// frame which overflowed rather than fitted would be visible as a row past
/// the right edge.
const TASK_TITLE: &str =
    "A task whose title is wider than any panel this reader draws at forty columns";

impl Repo {
    pub fn seeded() -> Repo {
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
// A driven session
// ---------------------------------------------------------------------------

/// `ank tui` on a real terminal of a stated size, drivable while it runs.
pub struct Live {
    child: std::process::Child,
    writer: std::fs::File,
    screen: std::sync::Arc<std::sync::Mutex<Screen>>,
    drain: Option<std::thread::JoinHandle<()>>,
}

impl Live {
    /// A session that may not paint, which is what every suite but the one
    /// about painting wants: `NO_COLOR` makes the frames the characters they
    /// are and nothing else.
    pub fn open(repo: &Repo, columns: u16, rows: u16) -> Live {
        Live::opened(repo, columns, rows, false, TERM)
    }

    /// A session that may paint (TASK-6cd41d23b7d1).
    ///
    /// `NO_COLOR` is *removed* from the child's environment rather than left
    /// unset here, and `TERM` is stated: this suite inherits whatever the
    /// developer running it has exported, and a test whose subject is colour
    /// must not be a test that reports the machine.
    pub fn painting(repo: &Repo, columns: u16, rows: u16) -> Live {
        Live::opened(repo, columns, rows, true, TERM)
    }

    /// A session on a terminal that has declared it can render nothing rich
    /// (ADR-c07e2694f0e1, proposed).
    ///
    /// `TERM=dumb` is the whole of the declaration, and it is what puts the
    /// reader on its ASCII border set and its plain palette at once -- one
    /// probe, two answers. `NO_COLOR` is left set as [`Live::open`] sets it,
    /// because it is beside the point here: a terminal this poor was getting
    /// the plain palette either way.
    pub fn dumb(repo: &Repo, columns: u16, rows: u16) -> Live {
        Live::opened(repo, columns, rows, false, "dumb")
    }

    fn opened(repo: &Repo, columns: u16, rows: u16, colour: bool, term: &str) -> Live {
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
            .current_dir(&repo.0)
            .env("ANK_AGENT", AGENT)
            .env("TERM", term);
        match colour {
            true => command.env_remove("NO_COLOR"),
            false => command.env("NO_COLOR", "1"),
        };
        let child = command
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

    pub fn until(&self, what: &str, done: impl Fn(&str) -> bool) {
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
    pub fn frame(&self) -> String {
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

    pub fn send(&mut self, bytes: &str) {
        use std::io::Write;
        self.writer
            .write_all(bytes.as_bytes())
            .expect("the terminal must accept a keystroke");
        self.writer.flush().unwrap();
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
        drop(self.writer);
        if let Some(drain) = self.drain.take() {
            drain.join().expect("the drain must not panic");
        }
        self.screen.lock().unwrap().raw.clone()
    }
}
