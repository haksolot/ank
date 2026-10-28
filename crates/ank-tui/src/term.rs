//! The engine: raw mode, the alternate screen, and the things that wake a
//! drawn screen (ADR-ac6be1ebe9aa).
//!
//! **No `extern` block enters this workspace for any of it, and that is the
//! whole of why this file may exist at all.** Raw mode is `tcsetattr` on Unix
//! and `SetConsoleMode` on Windows; the window is an `ioctl` on one and a
//! console call on the other; a keystroke is a byte on one and a console record
//! on the other. Each of those is two implementations of one behaviour, and
//! CLAUDE.md's rule is that OS-dependent behaviour is not verified until it has
//! run on all three platforms. crossterm *is* that cross-platform
//! implementation, maintained and exercised far beyond what this workspace
//! could give it, so what this crate spends is a dependency and not an
//! `extern`. `tests/dependencies.rs` reads that back out of the sources.
//!
//! **The mouse is captured, and that is a decision rather than a nicety**
//! (TASK-dd9747e5e305). A terminal does not report a press until it is asked
//! to, and the reader that asked is the reader a person holding a phone can
//! use at all: a tap is a mouse press on that wire and there is no other way it
//! arrives. What it costs is the terminal's own selection -- with capture on, a
//! drag is reported here instead of highlighting text -- and that is the trade
//! every full-screen reader with a pointer makes. It is given back on every
//! road out, beside raw mode and the alternate buffer.
//!
//! **Every road out of a session gives the terminal back.** A process that
//! exits in raw mode leaves a shell nobody can type into, and a process that
//! exits on the alternate buffer leaves a screen nobody asked for. So the
//! restoration is a [`Drop`] and never a line at the end of the loop: a `?`, a
//! refusal, a quit and a panic all take it. The panic case gets a hook of its
//! own on top, because a panic message painted onto the alternate screen is a
//! message the teardown then hides.
//!
//! **What the session is woken by is four things and one clock that paints.**
//! A key, a tap, a resize, and the change stream (`stream`); and, while a task
//! in progress stands on the frame, the pulse's tick ([`ticker`],
//! TASK-54bb5cddc671). That clock spawns no verb, so it reads nothing and
//! renews nothing: what a tick does is move a cell. And it stops when no
//! `in_progress` row is on the screen, parked until a frame holds one again,
//! so with nobody typing, nothing changing, the window still and nothing in
//! progress on the screen, this reader is a process asleep on a channel. That
//! is what keeps ADR-0bb7ea8991bc's "a claim is renewed by working" true of a
//! screen somebody left open and went home.
//!
//! The double tap's clock is here too ([`Moment`]): when a press landed, read
//! at the terminal's edge so the session and the frame read no clock at all.

use crate::view::App;
use crate::Wake;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io::Stdout;
use std::sync::mpsc::Sender;

/// What a session draws on.
///
/// A trait and not the concrete terminal, for the reason [`crate::session`]
/// takes its wakes as an iterator: the loop is the one piece of this crate that
/// cannot be driven from a unit test while it owns a real terminal, and a loop
/// nothing can drive is a loop that is only ever tested through a
/// pseudo-terminal. The suite in `ank-cli` still drives one, because that is
/// where the criterion lives; this is what lets the loop's own edges -- a
/// resize, a broken terminal, an exhausted stream -- be stated here too.
pub trait Painter {
    /// The window, as the terminal states it now.
    fn size(&mut self) -> std::io::Result<(u16, u16)>;
    /// One frame.
    fn draw(&mut self, app: &App) -> std::io::Result<()>;
}

/// A terminal in raw mode, on the alternate buffer, with ratatui over it.
pub struct Screen {
    terminal: ratatui::Terminal<CrosstermBackend<Stdout>>,
}

impl Screen {
    /// Takes the terminal, or gives back whatever half it had taken.
    ///
    /// The order is raw mode, then the alternate buffer, then ratatui, and each
    /// failure undoes exactly what succeeded before it. A reader that returned
    /// an error while leaving the terminal raw would be handing its caller a
    /// shell it could not type into, which is worse than the failure it was
    /// reporting.
    pub fn open() -> std::io::Result<Screen> {
        hook();
        enable_raw_mode()?;
        let mut out = std::io::stdout();
        if let Err(e) = execute!(out, EnterAlternateScreen, EnableMouseCapture) {
            let _ = restore();
            return Err(e);
        }
        match ratatui::Terminal::new(CrosstermBackend::new(out)) {
            Ok(terminal) => Ok(Screen { terminal }),
            Err(e) => {
                let _ = restore();
                Err(e)
            }
        }
    }
}

impl Painter for Screen {
    fn size(&mut self) -> std::io::Result<(u16, u16)> {
        let size = self.terminal.size()?;
        Ok((size.width, size.height))
    }

    fn draw(&mut self, app: &App) -> std::io::Result<()> {
        self.terminal.draw(|frame| app.draw(frame))?;
        Ok(())
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = restore();
    }
}

/// The terminal, given back.
///
/// Both halves attempted whatever the first one answers: a failure to leave the
/// alternate buffer is no reason to leave the terminal raw as well, and there
/// is nobody left to report either failure to.
fn restore() -> std::io::Result<()> {
    let left = execute!(std::io::stdout(), DisableMouseCapture, LeaveAlternateScreen);
    let cooked = disable_raw_mode();
    left.and(cooked)
}

/// The panic hook, installed once.
///
/// A panic unwinds through [`Screen`]'s `Drop`, which is what actually restores
/// the terminal; this is about the *message*. The default hook prints to stderr,
/// which at that moment is the alternate buffer -- so the sentence explaining
/// what went wrong is painted onto a screen the unwind then tears down. Leaving
/// first means the panic lands in the scrollback, where somebody can read it.
fn hook() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panicked| {
            let _ = restore();
            previous(panicked);
        }));
    });
}

/// How often a task in progress pulses: four phases a second
/// (TASK-54bb5cddc671).
pub const PULSE: std::time::Duration = std::time::Duration::from_millis(250);

/// The pulse's clock: a thread that sends [`Wake::Tick`] on the session's
/// channel, at most once every [`PULSE`], and only while the session says a
/// pulse is due (TASK-54bb5cddc671).
///
/// **It paints, and it is allowed nothing else.** A tick reads nothing,
/// spawns no verb and so renews no claim (ADR-0bb7ea8991bc): what the session
/// does with one is move a counter and draw. **And it stops when there is
/// nothing to pulse.** While no task in progress stands on the frame last
/// drawn the thread is parked, so a reader with no pulse on its screen is
/// still a process asleep on a channel.
///
/// One tick in flight at most: the thread lowers the flag as it sends, and
/// only the session's next frame raises it again. A session busy with a slow
/// read therefore finds one tick waiting when it is done, never a queue of
/// them to draw back to back. The precedent is [`crate::stream::follow`], a
/// thread on a quarter-second clock that touches no corpus.
pub struct Ticker {
    due: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: std::thread::Thread,
}

impl Ticker {
    /// Whether the frame just drawn holds a task in progress. Raising it
    /// wakes the thread; lowering it means no tick is sent until it is
    /// raised again.
    pub fn due(&self, due: bool) {
        let was = self.due.swap(due, std::sync::atomic::Ordering::SeqCst);
        if due && !was {
            self.thread.unpark();
        }
    }
}

/// Starts the pulse's clock on `wake`, parked until [`Ticker::due`] says
/// otherwise.
pub fn ticker(wake: Sender<Wake>) -> Ticker {
    use std::sync::atomic::Ordering::SeqCst;
    let due = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let theirs = std::sync::Arc::clone(&due);
    let thread = std::thread::spawn(move || loop {
        if !theirs.load(SeqCst) {
            // A spurious return from `park` is a pass round the loop.
            std::thread::park();
            continue;
        }
        std::thread::sleep(PULSE);
        if theirs.swap(false, SeqCst) && wake.send(Wake::Tick).is_err() {
            return;
        }
    });
    Ticker {
        due,
        thread: thread.thread().clone(),
    }
}

/// When a press landed (TASK-42de0df951a4), read off the machine's clock
/// here, at the terminal's edge, and nowhere in the session or the frame
/// (TASK-54bb5cddc671).
///
/// **What it decides is whether a press is the second of a pair**, and only
/// that: it starts nothing, wakes nothing, and no frame is drawn because time
/// passed. It lives in this file rather than in `view.rs` because the frame's
/// one clock is the pulse's counter, and `crate::tests` reads the sources of
/// the session and the frame to hold them to that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Moment(std::time::Instant);

impl Moment {
    /// The machine's clock, now.
    pub fn now() -> Moment {
        Moment(std::time::Instant::now())
    }

    /// How long after `earlier` this is, and zero where it is not after it.
    pub fn duration_since(self, earlier: Moment) -> std::time::Duration {
        self.0.saturating_duration_since(earlier.0)
    }
}

impl std::ops::Add<std::time::Duration> for Moment {
    type Output = Moment;

    /// A moment `by` later: how a test states a second press without waiting
    /// for one.
    fn add(self, by: std::time::Duration) -> Moment {
        Moment(self.0 + by)
    }
}

/// Reads the terminal on a thread, so the drawn screen can be woken by
/// something other than a keystroke.
///
/// **A release is not a press, and Windows is where that matters.** A terminal
/// on Unix sends bytes and crossterm reports one event per key; a Windows
/// console sends key records for the way down *and* the way up, and crossterm
/// reports both. A reader that took either would run every command twice there
/// and nowhere else -- the exact shape of defect CLAUDE.md's three-platform rule
/// exists to catch, and one no Linux run would ever show. A repeat is taken,
/// because holding `j` is somebody moving down a list on purpose.
///
/// **A mouse crossing the window is not a wake** (TASK-dd9747e5e305). Capture
/// is on, so a terminal reports movement and drags as well as presses, and a
/// reader woken by a pointer travelling over it would be a reader drawing a
/// frame per pixel for nobody. What is sent on is what a person did: a press, a
/// release and a scroll. The release is sent rather than dropped here because
/// which of them means something is the screen's question and not this thread's
/// -- the same line [`crate::view::App`] draws for a key.
///
/// Focus and paste are still read and dropped: neither is enabled, so neither
/// arrives.
pub fn typing(wake: Sender<Wake>) {
    std::thread::spawn(move || loop {
        let said = match event::read() {
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release => Wake::Key(key),
            Ok(Event::Mouse(mouse))
                if !matches!(mouse.kind, MouseEventKind::Moved | MouseEventKind::Drag(_)) =>
            {
                Wake::Mouse(mouse)
            }
            Ok(Event::Resize(columns, rows)) => Wake::Resize(columns, rows),
            Ok(_) => continue,
            Err(e) => Wake::Broken(e.to_string()),
        };
        let ending = matches!(said, Wake::Broken(_));
        if wake.send(said).is_err() || ending {
            return;
        }
    });
}

/// Watches for the terminal going away, on a thread of its own, and says so
/// as [`Wake::Broken`] (TASK-69e774ef9d64).
///
/// **[`typing`] cannot be the one to notice.** crossterm 0.29 reads the
/// terminal in a loop that leaves only on `WouldBlock`: a hung-up terminal
/// answers end of file on one platform and `EIO` on another, neither is
/// `WouldBlock`, and the loop reads again at once without returning to its
/// `poll`. Once the reading thread is inside `event::read` it never comes out,
/// so no check placed before the call could help. Measured on 2026-10-28: four
/// readers orphaned by a pseudo-terminal suite whose master had closed made
/// about 1.3 million `read` calls a second each, for days, and held a
/// four-core machine at 100%.
///
/// **A hang-up is otherwise a signal, and this reader is not always sent
/// one.** `SIGHUP` goes to the controlling process of a terminal, and a reader
/// handed a terminal it does not control -- by a script, a pane manager or
/// this crate's own suite -- is told nothing when the other side closes.
///
/// So this asks the descriptor the reader reads, and asks for nothing: a
/// `poll` that requests no event still reports a hang-up and an error, and
/// sleeps through every keystroke, so this thread is woken once, by the one
/// thing it watches for. A descriptor `poll` cannot watch is left alone rather
/// than taken for a hang-up, because a reader that quit on it would be a
/// reader that never opened.
///
/// On Windows this is nothing: the console is read through crossterm's own
/// records, not this loop.
#[cfg(unix)]
pub fn hangup(wake: Sender<Wake>) {
    use rustix::event::{poll, PollFd, PollFlags};
    use std::io::IsTerminal;
    use std::os::fd::{AsFd, OwnedFd};

    // The descriptor crossterm reads: standard input where it is a terminal,
    // and the process's terminal otherwise.
    let watched: OwnedFd = if std::io::stdin().is_terminal() {
        match std::io::stdin().as_fd().try_clone_to_owned() {
            Ok(fd) => fd,
            Err(_) => return,
        }
    } else {
        match std::fs::File::open("/dev/tty") {
            Ok(file) => file.into(),
            Err(_) => return,
        }
    };
    std::thread::spawn(move || loop {
        let mut fds = [PollFd::new(&watched, PollFlags::empty())];
        match poll(&mut fds, None) {
            Ok(_) => {}
            Err(rustix::io::Errno::INTR) => continue,
            Err(_) => return,
        }
        let said = fds[0].revents();
        if said.contains(PollFlags::NVAL) {
            return;
        }
        if said.intersects(PollFlags::HUP | PollFlags::ERR) {
            let _ = wake.send(Wake::Broken("the terminal hung up".to_string()));
            return;
        }
    });
}

/// Nothing to watch: see the Unix one.
#[cfg(not(unix))]
pub fn hangup(_wake: Sender<Wake>) {}
