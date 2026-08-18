//! The `ank` binary.
//!
//! Deliberately thin: it resolves the current directory, delegates to
//! [`cli::run`] and propagates the exit code. Nearly everything worth testing
//! lives in the modules, where it is testable without spawning a process — the
//! exception being what only exists once the process does, and the exit code
//! carrying the semantics of §4 is exactly that. `tests/cli.rs` spawns the real
//! binary for it.

// The foundation deliberately exposes more than dispatch consumes today:
// identity, config and store are written and tested here, but their callers
// are the verbs, which arrive task by task. The allow goes away with the last
// of them; keeping it at the root rather than scattered in annotations makes
// it visible and removable in one line.
#![allow(dead_code)]

mod cli;
mod config;
mod git;
mod identity;
mod init;
// The one writer and the one escaper, which now live in `ank-contract` so the
// protocol surface shares them rather than growing a second pair
// (TASK-e819448560e7). Re-exported under the path every call site already
// names: `crate::json::Obj` is what twenty-six documents are built with.
pub use ank_contract::json;
mod paint;
mod repo;
mod store;
mod style;

// Verb modules. Each is filled in by its own task — see .ank/tasks/ — and each
// adds its own arm to cli.rs::dispatch at that point; a verb whose module is
// still a stub answers not_implemented and names the task that owns it. They
// exist already so that the command table in cli.rs is complete and tested.
// `index` and `verify` are not verbs: nothing dispatches to them, the other
// modules call them.
mod claim;
mod commands;
mod context;
mod done;
mod edit;
mod editor;
mod entries;
mod graph;
mod human;
mod index;
mod migrate;
mod status;
mod verify;

fn main() {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut out = std::io::stdout();
    // Detected here and nowhere else: this is the only place that knows what
    // the process was actually attached to, and answering the question once
    // keeps every verb downstream from asking it differently (§4).
    let style = style::detect();
    // The one bare integer in the tool, and it is where the type has to end:
    // `exit` takes an `i32` (§4, ADR-6fd69efb629c).
    std::process::exit(cli::run(&argv, &cwd, &mut out, style).code());
}
