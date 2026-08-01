//! Conformance of the bootstrap skill.
//!
//! `skill/SKILL.md` is not documentation: it is loaded into an agent's context
//! permanently, on every session, in every repository that installs it. That is
//! what makes its content the thing actually frozen (ADR-c656cbcc33a9) -- the
//! dispatch table refuses nobody, and the loop exists because this file teaches
//! it and teaches nothing else. Two properties are therefore worth a test
//! rather than a habit: that it carries the whole loop, and that it stays small
//! enough to be worth loading.
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

/// The loop, and the content of SKILL.md is frozen at exactly these
/// (ADR-c656cbcc33a9).
const LOOP_VERBS: [&str; 8] = [
    "context", "claim", "show", "log", "done", "new", "find", "release",
];

#[test]
fn the_skill_carries_the_whole_loop() {
    let text = skill();
    for verb in LOOP_VERBS {
        assert!(
            text.contains(&format!("ank {verb}")),
            "SKILL.md never shows `ank {verb}`, and an agent only knows the \
             verbs this file names"
        );
    }
}

/// One page, and the budget is the reason: §9 puts the loop and the mental
/// model here, and sends flag detail to `ank help`, loaded on demand.
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

/// These verbs run for whoever types them -- nothing here is refused to an
/// agent (ADR-c656cbcc33a9). What this asserts is narrower and is the whole of
/// the freeze: SKILL.md does not *teach* them. Naming one as a thing to run
/// would grow what every session pays for, by habit rather than by decision,
/// which is how a permanently loaded file actually grows.
///
/// `show` is absent from this list because it was moved into the loop by
/// decision and with the reason recorded, back when the loop was still called a
/// surface. That is what the list is for -- everything else still costs a
/// succession.
#[test]
fn the_skill_teaches_nothing_beyond_the_loop() {
    let text = skill();
    for verb in ["accept", "review", "close", "attest", "check"] {
        assert!(
            !text.contains(&format!("ank {verb}")),
            "SKILL.md shows `ank {verb}`, which is outside the loop it is \
             frozen at"
        );
    }
}
