//! Conformance of the bootstrap skill.
//!
//! `skill/SKILL.md` is not documentation: it is loaded into an agent's context
//! permanently, on every session, in every repository that installs it. Two
//! properties are therefore worth a test rather than a habit -- that it carries
//! the whole agent surface, and that it stays small enough to be worth loading.
//!
//! This file exists because the task's declared verifier is `cargo-test`. A
//! criterion nothing executes is a criterion nobody checked, and a proof that
//! covers less than it appears to is worse than a missing one.

use std::fs;
use std::path::PathBuf;

fn skill() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("skill/SKILL.md");
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// The eight of ADR-3859eb46bdc3, and the surface is frozen at exactly these.
const AGENT_VERBS: [&str; 8] = [
    "context", "claim", "show", "log", "done", "new", "find", "release",
];

#[test]
fn the_skill_carries_the_whole_agent_surface() {
    let text = skill();
    for verb in AGENT_VERBS {
        assert!(
            text.contains(&format!("ank {verb}")),
            "SKILL.md never shows `ank {verb}`, and an agent only knows the \
             verbs this file names"
        );
    }
}

/// One page, and the budget is the reason: §9 puts the seven commands and the
/// mental model here, and sends flag detail to `ank help`, loaded on demand.
/// The number is a ceiling to notice drift, not a target to fill.
#[test]
fn the_skill_stays_within_one_page() {
    let text = skill();
    let lines = text.lines().count();
    let words = text.split_whitespace().count();
    assert!(
        lines <= 80,
        "SKILL.md is {lines} lines: it is loaded permanently, so growth costs \
         every session in every repo. Move detail to `ank help`."
    );
    assert!(words <= 700, "SKILL.md is {words} words, over one page");
}

/// The human verbs are not the agent's, and this file is the agent's. Naming
/// them as things to run would grow the surface ADR-3859eb46bdc3 freezes -- by
/// habit rather than by decision, which is how surfaces actually grow.
///
/// `show` is absent from this list because it is no longer on that side:
/// ADR-3859eb46bdc3 supersedes ADR-2f8a61c04b7d and moves it, by decision and
/// with the reason recorded. That is what the list is for -- everything else
/// still costs a succession.
#[test]
fn the_skill_does_not_hand_the_agent_a_human_verb() {
    let text = skill();
    for verb in ["accept", "review", "close", "attest", "check"] {
        assert!(
            !text.contains(&format!("ank {verb}")),
            "SKILL.md shows `ank {verb}`, which is on the human surface"
        );
    }
}
