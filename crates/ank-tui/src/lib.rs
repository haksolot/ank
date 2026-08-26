//! The `tui` verb: a full-screen reader over one corpus (ADR-8bd76e8d7c4e, §4),
//! drawn with ratatui over crossterm (ADR-0b55983421dd).
//!
//! **It reaches the corpus by running the CLI, and by nothing else.** Every row
//! of corpus data on the screen arrives as a `--json` document written by
//! `ank find`, `ank status`, `ank show`, `ank scope` or `ank review`, spawned as
//! a shell would spawn them. Nothing here links `ank-core`, opens `.ank/`, or touches
//! `refs/ank/*`. That is slower than linking the core and the trade is taken
//! deliberately: every refusal in this project is a refusal on state, a reader
//! that reproduced them would get one subtly wrong, and the first symptom would
//! be a screen showing a task as claimable that `claim` then refuses. Spawning
//! the binary makes that class of bug unreachable rather than unlikely.
//!
//! The chrome -- headings, the gutter, the key line -- is this crate's own; the
//! data is not, and `tests/tui.rs` in `ank-cli` asserts that the frames name
//! entities the corpus actually carries.
//!
//! **It writes nothing the person at the keyboard did not ask for**
//! (ADR-8bd76e8d7c4e). Repainting runs the verbs that only read, so a screen
//! left open all night runs no command at all and renews no claim; the six that
//! write -- `claim`, `log`, `release`, `done`, `amend` and `accept` -- run once
//! each, when their word is typed whole into the prompt. Nothing here is on a
//! timer, and there is no timer to put anything on. The assertion is in the
//! suite, not in this sentence.
//!
//! **A change reaches the screen as an event, never as a poll**
//! (TASK-2f7777a1fdff). `ank-daemon` appends a line when a corpus it watches
//! moves, and [`stream`] follows that file: an event wakes the session and the
//! screen is drawn again from the corpus. Where there is no stream -- no
//! watcher has ever run here, or this reader has no home to find one in -- the
//! screen is drawn again when its person presses `r`, which is what it always
//! did. Both routes reach the same displayed state, and the suite compares
//! them.
//!
//! An event is a repaint and never a write. It runs `status` and `find` and
//! deliberately not `show`, because `show` renews the lease when the id is the
//! task the caller holds (ADR-0bb7ea8991bc): a reader that re-read the open
//! entity on an event would have made a watcher's news renew a claim, which is
//! the one thing this reader has been forbidden twice over. The reasoning sits
//! on [`view::App::repaint`], next to the code that keeps it true.
//!
//! **And a write is the verb, run as a shell would run it.** Nothing here
//! composes a claim record, moves a ref or edits an entity: `ank claim <id>`
//! does that, and this crate spawns it. So ADR-052accd6e3b2 names an
//! intersecting claim exactly as it would in a terminal, a criterion frozen by
//! hash is frozen by the same code that freezes it everywhere else, and every
//! refusal on the screen is the one the binary gave -- with its exit code and
//! the command it named as the way out, unaltered.
//!
//! # The engine
//!
//! Raw mode, on the alternate buffer, drawn by ratatui and woken by three
//! things: a key, a resize, and the change stream. That is [`term`], and the
//! reason it may exist is stated there -- raw mode and the window are two
//! implementations of one behaviour on the platforms this project ships to,
//! crossterm is that implementation, and taking it puts no `extern` block in
//! this workspace at all.
//!
//! **The window is the terminal's answer and no longer an environment
//! variable.** A resize is an event the loop takes like any other, so a
//! terminal made narrower redraws to its new size with nobody typing -- which
//! is not a nicety: a reader that only reflowed when a command arrived would be
//! a reader whose screen is wrong for as long as its person is reading rather
//! than acting.
//!
//! # Input is a key, and the one line that is left
//!
//! Every command that only moves the screen is one key ([`keys`]), focus
//! included: `Tab` walks the panels, `1` to `4` name one, and Left and Right
//! cross the columns. Four of the six verbs that write carry something a key
//! has no room for -- a message, a reason, a proof, a flag -- so `a` opens a
//! one-line prompt and what is typed there goes through [`input`], which is the
//! grammar that spells the six whole. `/` opens the same prompt on a search.
//!
//! **What the line discipline used to guarantee is not yet replaced, and this
//! is the honest state of it.** Under a line reader a slipped finger typed
//! nothing, because a command was a word and Enter and no word was one letter.
//! Under a keystroke reader the guarantee is the confirmation that shows the
//! argv before anything is spawned, which ADR-0b55983421dd requires and
//! TASK-d4a882345837 builds. Until it lands, what stands between a slip and a
//! write is that no key writes at all: the road to a spawned verb runs through
//! a prompt somebody opened and a word somebody spelled.
//!
//! # The layout
//!
//! Four panels on one screen, two columns of two, one of them focused
//! (TASK-bb43cfe2192b). What used to be three views one at a time is now four
//! places at once, and the reason is that a corpus reader is read across rather
//! than down: what binds this, who holds it, what is waiting, and what does it
//! say are four questions a person carries together.
//!
//! * [`view::Focus::Claims`] -- who holds what, the caller's own marked.
//! * [`view::Focus::Entities`] -- every entity of every kind with its status,
//!   windowed and filterable. Where a session opens.
//! * [`view::Focus::Queue`] -- what is proposed and waiting for a signature,
//!   and which regime the corpus is in: `ank review`, run when its person
//!   focuses the panel and never on the reader's own initiative.
//! * [`view::Focus::Body`] -- one entity: what holds it, the constraints
//!   binding its declared scope, and its body, paged rather than cut. This is
//!   the only panel `accept` can be typed in.
//!
//! Focus moves by key and is drawn in characters -- a heavy border and the
//! `> ` marker -- so the screen says where a reader is with no colour at all.
//! It also decides the width: the focused column takes four fifths of it, which
//! is what lets a list of sentences and a body of prose share eighty columns
//! without both being too narrow. [`view::App::arrange`] is where all of that
//! is decided, and it is the one function TASK-dd9747e5e305 has to extend for a
//! phone.
//!
//! # Ratification, and the two words that are not the same
//!
//! ADR-8bd76e8d7c4e lets this reader *drive* `ank accept` and forbids it
//! *performing* one unattended, and TASK-d90e94afca08 is where that sentence
//! became code. Driving is spawning the verb, exactly as `claim` is spawned:
//! one identifier, no flags, because a person typed the word whole on a
//! document they had opened. Performing would be holding a key, answering a
//! prompt, or moving more than one document at a time -- and none of the three
//! is reachable from here. No key is `accept` and no key is any of the six, so
//! a held key repeats a movement and nothing else; the queue is never accepted
//! in bulk because the grammar has no shape for it and because the word is
//! refused anywhere but the body panel; no secret can reach git
//! through this process because the child is given no stdin; and a screen left
//! open all night still runs nothing at all, because `accept` is on the acting
//! list and a repaint only ever reads.

use ratatui::crossterm::event::KeyEvent;
use std::io::IsTerminal;
use std::path::PathBuf;

pub mod ank;
pub mod input;
pub mod keys;
pub mod model;
pub mod stream;
pub mod term;
pub mod text;
pub mod view;

pub use ank::Ank;
pub use ank_contract::ExitCode;
pub use term::Painter;

/// Where the reader is told to look when it cannot open a screen.
///
/// One command, in the shape every refusal in this tool ends on. `context` and
/// not `find`: the caller who typed `tui` into a pipe is far more often an agent
/// that meant to orient itself than a human who meant to browse, and orienting
/// is what `context` is (ADR-8bd76e8d7c4e).
pub const INSTEAD: &str = "ank context";

/// A refusal, in the two halves the CLI renders: the sentence and the command
/// that resolves it.
///
/// Returned rather than printed. This crate knows nothing about how `ank-cli`
/// renders an error -- that is one rendering and it lives there
/// (ADR-1f70ce2c3eac) --
/// so what crosses the boundary is the code, the message and the next command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    pub code: ExitCode,
    pub message: String,
    pub hint: &'static str,
}

/// How the reader addresses the corpus, and where it finds the CLI.
///
/// The address is the caller's own `--repo` and `--worktree`, passed through
/// untouched, and the working directory is the caller's. So the child resolves
/// the corpus exactly as the parent did, rather than being handed a path the
/// parent resolved -- one walk, one answer, and no second resolution to
/// disagree with the first.
#[derive(Debug, Clone)]
pub struct Address {
    pub exe: PathBuf,
    pub cwd: PathBuf,
    pub repo: Option<String>,
    pub worktree: Option<String>,
}

impl Address {
    /// The flags that address the corpus, in the form the child is given them.
    pub fn flags(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(repo) = &self.repo {
            out.push("--repo".to_string());
            out.push(repo.clone());
        }
        if let Some(worktree) = &self.worktree {
            out.push("--worktree".to_string());
            out.push(worktree.clone());
        }
        out
    }
}

/// The verb.
///
/// **The terminal check is first, and it is first for both forms.** `--json`
/// does not exempt a caller from it: §4 makes `--json` available on every verb
/// without exception, and a `tui` that answered a document into a pipe while
/// refusing a screen there would make "with no terminal, `ank tui` refuses"
/// a sentence with a footnote. With a terminal, `--json` answers the opening
/// frame as data and opens no session -- the reader's own answer, one document,
/// nothing else on the stream.
pub fn run(
    address: &Address,
    json: bool,
    out: &mut dyn std::io::Write,
) -> Result<ExitCode, Refused> {
    if !attached() {
        return Err(no_terminal());
    }
    let ank = Ank::new(address.clone());
    if json {
        let snapshot = model::Snapshot::load(&ank).map_err(refused_by_the_cli)?;
        let _ = writeln!(out, "{}", snapshot.document());
        return Ok(ExitCode::Ok);
    }
    // The corpus this reader is on, asked once and never again. The stream
    // names corpora by the repository identity of ADR-621a7fd96ce1, so a
    // follower has to know which lines are its own before the first one
    // arrives, and an identity does not change under a session. A refusal here
    // is not one: the reader simply has no stream to follow and falls back to
    // reading when its person asks, which is what the next line already does.
    let corpus = ank
        .json("status", &[])
        .map(|v| ank::text(&v, "corpus"))
        .unwrap_or_default();
    let (wake, waking) = std::sync::mpsc::channel::<Wake>();
    let stream = stream::follow(&corpus, wake.clone());
    // The terminal is taken last, and given back by [`term::Screen`]'s `Drop`
    // on every road out of what follows.
    let mut screen = term::Screen::open().map_err(no_screen)?;
    term::typing(wake);
    // Blocking, and there is nothing else in it: with nobody typing, nothing
    // changing and the window still, this reader is a process asleep on a
    // channel. No verb runs, no clock ticks and no claim moves.
    let mut wakes = std::iter::from_fn(move || waking.recv().ok());
    session(&ank, &mut wakes, &mut screen, stream)
}

/// What can wake a drawn screen. There are exactly three things, and a fourth
/// that ends the session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wake {
    /// A key the person pressed, which is a press or a repeat and never a
    /// release (see [`term::typing`]).
    Key(KeyEvent),
    /// The terminal is a different size now. It carries the new one, and what
    /// the screen does about it is draw again -- nothing is read, and no verb
    /// runs, because a window is a fact about the terminal and never about the
    /// corpus.
    Resize(u16, u16),
    /// The watcher said this corpus moved (TASK-2f7777a1fdff). It carries what
    /// happened and not what to do about it, which is why there is nothing in
    /// this variant: the answer is to read the corpus again.
    Changed,
    /// The terminal could not be read from. An environment to repair, and the
    /// one way out of a session that is not a zero.
    Broken(String),
}

/// Whether there is a screen to draw on.
///
/// Both streams are asked about, and either one being redirected is enough: a
/// screen painted into a file is a file full of escape sequences, and a screen
/// waiting on a closed stdin is a process that hangs. An agent that typed
/// `ank tui` by accident must get a refusal it can read rather than either.
pub fn attached() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// The refusal §4 owes a caller with no terminal, for the dispatch to render.
pub fn no_terminal() -> Refused {
    Refused {
        code: ExitCode::Environment,
        message: "'tui' needs a terminal on stdin and stdout, and this process has neither"
            .to_string(),
        hint: INSTEAD,
    }
}

/// The terminal could not be taken: raw mode was refused, or the alternate
/// buffer was.
///
/// An environment to repair rather than a defect of the corpus, so it leaves by
/// the same road the missing terminal does and names the same command. Nothing
/// has been drawn at this point and nothing needs giving back: `Screen::open`
/// undoes whatever half it had taken before it answers.
fn no_screen(error: std::io::Error) -> Refused {
    Refused {
        code: ExitCode::Environment,
        message: format!("'tui' could not take this terminal: {error}"),
        hint: INSTEAD,
    }
}

/// A refusal the child gave, carried out whole.
fn refused_by_the_cli(failed: ank::Failed) -> Refused {
    Refused {
        code: failed.code(),
        message: failed.to_string(),
        hint: INSTEAD,
    }
}

/// The session, with its edges injected so it can be driven.
///
/// `wakes` is an iterator and not a reader, because a screen has three things
/// that can move it and a `read` can only ever be one. It blocks in `next`,
/// which is where an idle session sits: no verb, no clock, nothing.
///
/// `screen` is a [`Painter`] and not the terminal, so the loop's own edges can
/// be stated without owning one. What a session does on a real terminal is the
/// suite's question and it drives a real one; what is asserted here is that a
/// resize redraws, a broken terminal ends the session with an environment code,
/// and an exhausted stream of wakes is a quit.
///
/// `stream` is what the screen says about how it is being kept current, and
/// `None` where there is nothing to follow.
///
/// **The window is read from the terminal before every frame**, and the resize
/// event is answered as well. Two roads to one fact, deliberately: the event is
/// what wakes a screen nobody is typing at, and the reading before the paint is
/// what keeps the size the page arithmetic used and the size ratatui draws into
/// from ever being two different numbers.
pub fn session(
    ank: &Ank,
    wakes: &mut dyn Iterator<Item = Wake>,
    screen: &mut dyn Painter,
    stream: Option<stream::Stream>,
) -> Result<ExitCode, Refused> {
    let opened = screen.size().unwrap_or((80, 24));
    let mut app = view::App::new((opened.0 as usize, opened.1 as usize), stream);
    app.reload(ank);
    let code = loop {
        if let Ok((columns, rows)) = screen.size() {
            app.resize(columns, rows);
        }
        if let Err(e) = screen.draw(&app) {
            // Nothing to draw the reason on, so it leaves by the exit code and
            // by stderr once the terminal has been given back.
            app.note(format!("cannot draw to the terminal: {e}"));
            break ExitCode::Environment;
        }
        // Exhausted means every sender is gone, which is the same end of input
        // by another road.
        let Some(wake) = wakes.next() else {
            break ExitCode::Ok;
        };
        match wake {
            Wake::Broken(e) => {
                app.note(format!("cannot read from the terminal: {e}"));
                break ExitCode::Environment;
            }
            Wake::Resize(columns, rows) => app.resize(columns, rows),
            // News, and never an instruction: what the screen does about it is
            // read the corpus again, and `repaint` is where the one read it may
            // not run is refused.
            Wake::Changed => app.repaint(ank),
            Wake::Key(key) => {
                if app.press(key, ank) {
                    break ExitCode::Ok;
                }
            }
        }
    };
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_address_passes_the_callers_own_flags_through() {
        let a = Address {
            exe: PathBuf::from("ank"),
            cwd: PathBuf::from("."),
            repo: Some("/x".to_string()),
            worktree: Some("/y".to_string()),
        };
        assert_eq!(a.flags(), ["--repo", "/x", "--worktree", "/y"]);
        let bare = Address {
            repo: None,
            worktree: None,
            ..a
        };
        assert!(bare.flags().is_empty(), "nothing is invented for the child");
    }

    #[test]
    fn the_refusal_names_the_command_to_run_instead() {
        let r = no_terminal();
        assert_eq!(r.code, ExitCode::Environment);
        assert_eq!(r.hint, "ank context");
        assert!(r.message.contains("terminal"), "{}", r.message);
    }

    /// A screen a test can hold, so the loop's own edges can be stated without
    /// a terminal.
    ///
    /// It keeps every frame it was given, which is what makes "the screen was
    /// drawn again, and at the new size" an assertion rather than an inference.
    struct Paper {
        size: std::rc::Rc<std::cell::Cell<(u16, u16)>>,
        frames: Vec<String>,
        broken: Option<&'static str>,
    }

    impl Paper {
        fn new(size: std::rc::Rc<std::cell::Cell<(u16, u16)>>) -> Paper {
            Paper {
                size,
                frames: Vec::new(),
                broken: None,
            }
        }
    }

    impl Painter for Paper {
        fn size(&mut self) -> std::io::Result<(u16, u16)> {
            Ok(self.size.get())
        }

        fn draw(&mut self, app: &view::App) -> std::io::Result<()> {
            match self.broken {
                Some(e) => Err(std::io::Error::other(e)),
                None => {
                    self.frames.push(app.frame());
                    Ok(())
                }
            }
        }
    }

    /// The CLI, addressed where there is none: every read fails to spawn, the
    /// session survives it and draws the refusal, which is what lets the loop
    /// be driven with no corpus at all.
    fn nowhere() -> Ank {
        Ank::new(Address {
            exe: "/nonexistent/ank".into(),
            cwd: ".".into(),
            repo: None,
            worktree: None,
        })
    }

    /// A terminal whose window moves with the events it sends, which is what a
    /// terminal is: the `ioctl` and the `SIGWINCH` agree.
    fn driven(
        first: (u16, u16),
        said: Vec<Wake>,
    ) -> (
        Paper,
        impl Iterator<Item = Wake>,
        std::rc::Rc<std::cell::Cell<(u16, u16)>>,
    ) {
        let size = std::rc::Rc::new(std::cell::Cell::new(first));
        let moving = std::rc::Rc::clone(&size);
        let mut said = said.into_iter();
        let wakes = std::iter::from_fn(move || {
            let next = said.next()?;
            if let Wake::Resize(columns, rows) = next {
                moving.set((columns, rows));
            }
            Some(next)
        });
        (Paper::new(std::rc::Rc::clone(&size)), wakes, size)
    }

    fn key(code: ratatui::crossterm::event::KeyCode) -> Wake {
        Wake::Key(KeyEvent::new(
            code,
            ratatui::crossterm::event::KeyModifiers::NONE,
        ))
    }

    /// A terminal made narrower redraws to its new size, with nobody typing
    /// (TASK-4fa385c1772d, ADR-0b55983421dd).
    ///
    /// The only wake in the session is the resize, so a second frame existing at
    /// all is the reader answering the window rather than a command. What is
    /// asserted about that frame is the shape of the new window in both
    /// directions: as many rows as the terminal now has, and no line wider than
    /// it is.
    #[test]
    fn a_resize_draws_the_screen_again_at_the_new_size_with_nothing_typed() {
        let (mut paper, mut wakes, _size) = driven((120, 40), vec![Wake::Resize(60, 20)]);
        let code = session(&nowhere(), &mut wakes, &mut paper, None).expect("a session");
        assert_eq!(code, ExitCode::Ok);
        assert_eq!(
            paper.frames.len(),
            2,
            "the screen was not drawn again for the resize"
        );
        let (before, after) = (&paper.frames[0], &paper.frames[1]);
        assert_eq!(before.lines().count(), 40);
        assert_eq!(after.lines().count(), 20, "the new height:\n{after}");
        for line in after.lines() {
            assert!(
                line.chars().count() <= 60,
                "{} columns in a 60 column window: {line}",
                line.chars().count()
            );
        }
        // And the frame is still this reader's: a resize reads nothing, so what
        // was on the screen is what is on it.
        assert!(after.contains("ank tui"), "{after}");
    }

    /// A key that quits ends the session, and the exit is a zero.
    #[test]
    fn a_key_that_quits_ends_the_session() {
        let (mut paper, mut wakes, _size) = driven(
            (100, 30),
            vec![
                key(ratatui::crossterm::event::KeyCode::Char('j')),
                key(ratatui::crossterm::event::KeyCode::Char('q')),
            ],
        );
        assert_eq!(
            session(&nowhere(), &mut wakes, &mut paper, None).expect("a session"),
            ExitCode::Ok
        );
        // One for the opening frame, one after the `j`, and none after the quit:
        // a session that drew again on its way out would be painting a screen
        // it is about to take away.
        assert_eq!(paper.frames.len(), 2);
    }

    /// Nothing left to wake it is a quit, not a fault: every sender is gone.
    #[test]
    fn a_session_with_nothing_left_to_wake_it_is_a_quit() {
        let (mut paper, mut wakes, _size) = driven((100, 30), Vec::new());
        assert_eq!(
            session(&nowhere(), &mut wakes, &mut paper, None).expect("a session"),
            ExitCode::Ok
        );
        assert_eq!(paper.frames.len(), 1, "the opening frame was drawn");
    }

    /// A terminal that cannot be read from is an environment to repair, and it
    /// is the one way out of a session that is not a zero.
    #[test]
    fn a_terminal_that_broke_ends_the_session_with_the_environment_code() {
        let (mut paper, mut wakes, _size) = driven(
            (100, 30),
            vec![Wake::Broken("input/output error".to_string())],
        );
        assert_eq!(
            session(&nowhere(), &mut wakes, &mut paper, None).expect("a session"),
            ExitCode::Environment
        );
    }

    /// And a terminal that cannot be drawn to is the same fact from the other
    /// side.
    #[test]
    fn a_terminal_that_cannot_be_drawn_to_ends_the_session_the_same_way() {
        let (mut paper, mut wakes, _size) = driven((100, 30), Vec::new());
        paper.broken = Some("no space left on device");
        assert_eq!(
            session(&nowhere(), &mut wakes, &mut paper, None).expect("a session"),
            ExitCode::Environment
        );
        assert!(paper.frames.is_empty());
    }
}
