//! The engine: raw mode, the alternate screen, and the three things that wake a
//! drawn screen (ADR-c07e2694f0e1).
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
//! **What the session is woken by is four things and no clock.** A key, a tap,
//! a resize, and the change stream (`stream`). Nothing here is on a timer and
//! there is no timer to put anything on: with nobody typing, nothing changing
//! and the window still, this reader is a process asleep on a channel. That is
//! what keeps ADR-0bb7ea8991bc's "a claim is renewed by working" true of a
//! screen somebody left open and went home.

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
