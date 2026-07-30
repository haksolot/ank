//! The `ank` binary.
//!
//! Deliberately thin: it resolves the current directory, delegates to
//! [`cli::run`] and propagates the exit code. Everything worth testing lives
//! in the modules, where it is testable without spawning a process.

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
mod repo;
mod store;

// Verb modules, which dispatch routes to. Each is filled in by its own task —
// see .ank/tasks/. They exist already so that the command table in cli.rs is
// complete and tested, and so that no verb task has to touch dispatch.
mod claim;
mod commands;
mod context;
mod done;
mod human;
mod index;
mod verify;

fn main() {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut out = std::io::stdout();
    std::process::exit(cli::run(&argv, &cwd, &mut out));
}
