//! The one integration test target of `ank-tui`: every file beside this one is
//! a module of it, so the dependency tree is linked once rather than once per
//! file (TASK-22af5b0918a4). `Cargo.toml` sets `autotests = false` and names
//! this file, so a new suite is a new `mod` line here and never a new binary.
//!
//! `terminal` is declared here, once, and only on Unix, where the suites that
//! drive a pseudo-terminal run; each reaches it as `crate::terminal`, and the
//! ones that open a session take its `ONE_AT_A_TIME` lock.

#[cfg(unix)]
mod terminal;

// The Windows half of what `terminal` measures about a terminal that goes
// away (TASK-69e774ef9d64): ConPTY, where the console is records and not bytes.
#[cfg(windows)]
mod conpty;

mod address;
mod bindings;
mod chrome;
mod claims;
mod colour;
mod commands;
mod config;
mod confirmation;
mod dashboard;
mod dependencies;
mod edit;
mod entity;
mod graph;
mod keyline;
mod kind;
mod log;
mod note;
mod opening;
mod ordering;
mod overlay;
mod phone;
mod press;
mod refusal;
mod region;
mod search;
mod verbs;
mod wheel;
