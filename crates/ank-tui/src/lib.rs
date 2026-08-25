//! The `tui` verb: a full-screen reader over one corpus (ADR-8bd76e8d7c4e, §4).
//!
//! **It reaches the corpus by running the CLI, and by nothing else.** Every row
//! of corpus data on the screen arrives as a `--json` document written by
//! `ank find`, `ank status`, `ank show` or `ank scope`, spawned as a shell would
//! spawn them. Nothing here links `ank-core`, opens `.ank/`, or touches
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
//! (ADR-8bd76e8d7c4e). Repainting runs the four verbs that only read, so a
//! screen left open all night runs no command at all and renews no claim; the
//! five that write -- `claim`, `log`, `release`, `done` and `amend` -- run once
//! each, when their word is typed whole. Nothing here is on a timer, and there
//! is no timer to put anything on. The assertion is in the suite, not in this
//! sentence.
//!
//! **And a write is the verb, run as a shell would run it.** Nothing here
//! composes a claim record, moves a ref or edits an entity: `ank claim <id>`
//! does that, and this crate spawns it. So ADR-052accd6e3b2 names an
//! intersecting claim exactly as it would in a terminal, a criterion frozen by
//! hash is frozen by the same code that freezes it everywhere else, and every
//! refusal on the screen is the one the binary gave -- with its exit code and
//! the command it named as the way out, unaltered.
//!
//! # Why the input is a line and not a keystroke
//!
//! Raw mode is two implementations of one behaviour: `tcsetattr` on Unix and
//! `SetConsoleMode` on Windows, each behind an `extern` block this workspace
//! does not otherwise have. CLAUDE.md's rule is that OS-dependent behaviour is
//! not verified until it has run on all three platforms, and only one of those
//! two can be run from where this was written. So the reader uses the terminal's
//! own line discipline as its input layer -- the same reflex that sends git
//! plumbing through the git binary rather than reimplementing it -- and a
//! command is a short word followed by Enter. The screen is full-screen all the
//! same: it lives on the alternate buffer and is repainted whole on every
//! command.
//!
//! It buys something back, too. A line can carry an argument, so `f adr`,
//! `/claim` and `TASK-4974` are commands rather than modes, which a
//! keystroke reader would have had to build a prompt for -- and so are
//! `log <what you learned>` and `done commit:<sha>`, which a keystroke reader
//! could not have taken at all without one.
//!
//! # The layout
//!
//! Two views, one screen each, because a criterion that asks for a body *whole*
//! and a list of every kind on one screen asks for two screens.
//!
//! * [`view::View::List`] -- who holds what, then every entity of every kind
//!   with its status, windowed and filterable.
//! * [`view::View::Entity`] -- one entity: what holds it, the constraints
//!   binding its declared scope, and its body, paged rather than cut.

use std::io::IsTerminal;
use std::path::PathBuf;

pub mod ank;
pub mod frame;
pub mod input;
pub mod model;
pub mod view;

pub use ank::Ank;
pub use ank_contract::ExitCode;

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
/// renders an error -- that is one rendering and it lives there (ADR-0c8a) --
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
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    session(&ank, &mut input, out, terminal_size())
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

/// A refusal the child gave, carried out whole.
fn refused_by_the_cli(failed: ank::Failed) -> Refused {
    Refused {
        code: failed.code(),
        message: failed.to_string(),
        hint: INSTEAD,
    }
}

/// The session, with its three edges injected so the suite can drive it.
///
/// `size` is passed in rather than measured here: measuring the window is an
/// `ioctl` on Unix and a console call on Windows, which is the FFI this crate
/// declines (see the module header), and a test that could not choose the
/// window could not assert what a frame holds.
pub fn session(
    ank: &Ank,
    input: &mut dyn std::io::BufRead,
    out: &mut dyn std::io::Write,
    size: (usize, usize),
) -> Result<ExitCode, Refused> {
    let mut app = view::App::new(size);
    app.reload(ank);
    let _ = write!(out, "{}", frame::ENTER);
    let code = loop {
        let _ = write!(out, "{}{}", frame::HOME, app.frame());
        let _ = out.flush();
        let mut line = String::new();
        match input.read_line(&mut line) {
            // End of input is a quit and never an error: a session whose
            // terminal went away has nothing left to draw to.
            Ok(0) => break ExitCode::Ok,
            Ok(_) => {}
            Err(e) => {
                app.note(format!("cannot read from the terminal: {e}"));
                break ExitCode::Environment;
            }
        }
        if app.act(
            input::parse(line.trim_end_matches(['\n', '\r']), app.view()),
            ank,
        ) {
            break ExitCode::Ok;
        }
    };
    let _ = write!(out, "{}", frame::LEAVE);
    let _ = out.flush();
    Ok(code)
}

/// The window, as the environment states it, or the classic default.
///
/// `COLUMNS` and `LINES` are what a shell exports when it knows, and 80x24 is
/// what every terminal is at least. A wrong guess costs a narrower frame and
/// never a wrong row: the renderer clamps rather than assuming.
pub fn terminal_size() -> (usize, usize) {
    sized(std::env::var("COLUMNS").ok(), std::env::var("LINES").ok())
}

/// The measurement itself, given the two values rather than reading them.
///
/// Split out so the suite can state a window without writing to the process
/// environment, which is shared by every test in a binary and is therefore the
/// one input a test must never set.
fn sized(columns: Option<String>, lines: Option<String>) -> (usize, usize) {
    // Below twenty columns or ten rows there is no frame to draw, only a
    // vertical smear, so a value that small is treated as absent rather than
    // honoured: `COLUMNS=0` is what a shell leaves behind, not a window.
    let read = |value: Option<String>, floor: usize, fallback: usize| {
        value
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|n| *n >= floor)
            .unwrap_or(fallback)
    };
    (read(columns, 20, 80), read(lines, 10, 24))
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

    #[test]
    fn a_window_too_small_to_draw_in_is_not_believed() {
        let s = |c: &str, l: &str| sized(Some(c.to_string()), Some(l.to_string()));
        assert_eq!(
            s("0", "4"),
            (80, 24),
            "a shell's leftover zero is not a window"
        );
        assert_eq!(s("not a number", "x"), (80, 24));
        assert_eq!(s("120", "48"), (120, 48), "a stated window is honoured");
        assert_eq!(sized(None, None), (80, 24));
    }
}
