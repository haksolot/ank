//! What `ank help tui` says this reader spells, held to the reader's own key
//! table, through the binary (TASK-a836fdb2fca2).
//!
//! **The drift this exists to make impossible has already happened once.** The
//! note read *claim, log, release, done, amend and accept act on the entity the
//! focused panel names* while the reader had grown `new`, `edit` and `config`,
//! and nothing failed: a sentence transcribed beside a table agrees with that
//! table only for as long as somebody remembers to retype it. LOG-f455c09157bc
//! and LOG-1afc1b09f95b are two agents finding it and each declaring it out of
//! its own perimeter. `crates/ank-tui` depends on `ank-contract`, so the
//! comparison has somewhere to live; `ank-contract` cannot see this crate, so
//! it cannot live on the other side.
//!
//! **Both directions, because either one alone is half a check.** A verb bound
//! here and missing from the note is a help page short of the reader, which is
//! the failure §9 exists to prevent -- a page narrower than the binary is read
//! as the binary refusing. A verb named on the note and bound nowhere is the
//! opposite failure and the worse one: a person typing what the page taught
//! them at a reader that answers nothing.
//!
//! **Through the binary, which is CLAUDE.md's rule and the criterion's own
//! words.** "ank help tui names every verb the reader spells" is a claim about
//! what a process prints. A unit test could compare two constants this crate
//! links -- and the pair being one comparison is exactly why they cannot
//! disagree -- but between `ank_contract::verbs` and a person reading a help
//! page there is a renderer, and this repository has twice shipped green unit
//! tests over code the binary never reached. `tests/config.rs` reads
//! `ank help config`'s note the same way and for the same reason.
//!
//! **No `#![cfg(unix)]`, and no `terminal/mod.rs`.** Every other suite here
//! declares that module because it opens a pseudo-terminal, which on Windows is
//! ConPTY and a console API this workspace does not call. Nothing here needs a
//! terminal: what is measured is a page written to a pipe and a static table,
//! and neither has a platform. So the binary is found with the ten lines below
//! rather than by taking a unix-only module along with it, and this suite runs
//! on all three -- which is what CLAUDE.md asks of anything that is not
//! OS-dependent.

use ank_tui::bindings::{BINDINGS, FURTHER};
use std::path::PathBuf;
use std::process::Command;

/// What the note this suite reads opens with.
///
/// The names are the front of the sentence and the prose is behind a `;`, which
/// is a shape rather than a decoration: the note has to be readable by this
/// suite, and a list that has to be picked out of the middle of a sentence is a
/// list picked out by a guess. `crates/ank-contract/src/verbs.rs` says the same
/// thing beside the note itself.
const NAMES: &str = "the verbs it spells are ";
/// Where the names stop and the sentence about them starts.
const UNTIL: char = ';';

/// The `ank` this workspace just built.
///
/// Beside the test executable's own directory: cargo puts an integration test
/// in `<target>/<profile>/deps/` and a binary in `<target>/<profile>/`.
/// `CARGO_BIN_EXE_ank` is defined only for the package that declares the
/// binary, and that is `ank-cli`.
fn ank() -> PathBuf {
    let mut at = std::env::current_exe().expect("a test executable has a path");
    at.pop();
    if at.file_name().is_some_and(|n| n == "deps") {
        at.pop();
    }
    let binary = at.join(format!("ank{}", std::env::consts::EXE_SUFFIX));
    assert!(
        binary.is_file(),
        "the ank binary is not at {}: this suite drives the process, so build it \
         first (cargo test --workspace, or cargo build -p ank-cli)",
        binary.display()
    );
    binary
}

/// `ank help tui`, as a person at a shell reads it.
///
/// No corpus and no `--repo`: `help` answers about the verb table and reaches
/// no repository at all, so a page that needed one would be a refusal here
/// rather than a page.
fn page() -> String {
    let out = Command::new(ank())
        .args(["help", "tui"])
        .output()
        .expect("the binary must have been built");
    assert!(
        out.status.success(),
        "'ank help tui' refused: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// The verbs the page says this reader spells, in the order it names them.
fn named(page: &str) -> Vec<String> {
    let line = page
        .lines()
        .map(str::trim)
        .find_map(|line| line.split_once(NAMES).map(|(_, rest)| rest.to_string()))
        .unwrap_or_else(|| {
            panic!("no note of 'ank help tui' opens with {NAMES:?}:\n{page}");
        });
    let list = line.split(UNTIL).next().unwrap_or_default();
    list.split(", ")
        .flat_map(|piece| piece.split(" and "))
        .map(|verb| verb.trim().to_string())
        .filter(|verb| !verb.is_empty())
        .collect()
}

/// Every verb this reader spells, out of its own table.
///
/// Two sources and both of them the table's: a row of [`BINDINGS`] that carries
/// a verb spells it on a key, and a name in [`FURTHER`] spells it on a row of
/// the list `x` opens. A verb reachable by either is a verb a person can run
/// from this reader, so a note that named one half would be a note about half
/// the reader.
fn spelled() -> Vec<String> {
    let mut out: Vec<String> = BINDINGS
        .iter()
        .filter_map(|binding| binding.verb)
        .map(|verb| verb.name.to_string())
        .collect();
    out.extend(FURTHER.iter().map(|verb| verb.to_string()));
    out.sort();
    out.dedup();
    out
}

/// **The note names every verb the reader spells, and no verb it does not**
/// (TASK-a836fdb2fca2).
///
/// The assertion is on the sets and the message carries the difference, because
/// "these two lists differ" sends the next reader back to count words: what
/// fails a wave from now is one verb, and naming it is the difference between a
/// diagnosis and a diff.
#[test]
fn the_help_page_names_exactly_the_verbs_the_reader_spells() {
    let page = page();
    let mut named = named(&page);
    named.sort();
    let mut deduped = named.clone();
    deduped.dedup();
    assert_eq!(
        named, deduped,
        "'ank help tui' names a verb twice: {named:?}"
    );
    let spelled = spelled();

    let missing: Vec<&String> = spelled.iter().filter(|v| !named.contains(v)).collect();
    assert!(
        missing.is_empty(),
        "the reader spells {missing:?} and 'ank help tui' does not name {}: a \
         help page narrower than the binary is read as the binary refusing. The \
         note is `ank_contract::verbs`' own, on the `tui` command.\n{page}",
        match missing.len() {
            1 => "it",
            _ => "them",
        }
    );
    let invented: Vec<&String> = named.iter().filter(|v| !spelled.contains(v)).collect();
    assert!(
        invented.is_empty(),
        "'ank help tui' names {invented:?} and no row of \
         `ank_tui::bindings::BINDINGS` spells it, nor does `FURTHER`: a page \
         teaching a key this reader does not answer to is worse than a page \
         that is short one.\n{page}"
    );
    assert_eq!(named, spelled, "the note and the table disagree:\n{page}");
}

/// **The sentence the note ends on is stated over the whole of them**
/// (ADR-559eebf5c6f5).
///
/// This is the half a list alone cannot carry, and it is the half that made
/// this task: the note used to say the verbs *act on the entity the focused
/// panel names*, which is true of the writing six and false of `ank new`, whose
/// first positional is a kind, and of `ank config`, which names no entity at
/// all. So what is asserted is that the two phrasings the drift produced are
/// gone -- and the note is not allowed to count, because a number is a fact
/// about the table that has to be retyped every time the table grows.
#[test]
fn the_note_counts_nothing_and_claims_nothing_of_a_subset() {
    let page = page();
    let note = page
        .lines()
        .map(str::trim)
        .find(|line| line.contains(NAMES))
        .expect("the note this suite reads is on the page")
        .to_string();
    for said in [
        "act on the entity the focused panel names",
        "six",
        "seven",
        "eight",
    ] {
        assert!(
            !note.contains(said),
            "'ank help tui' says {said:?}, which is a claim about some of the \
             verbs it names rather than about all of them: {note}"
        );
    }
}
