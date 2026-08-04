//! Conformance of the bootstrap skill.
//!
//! `skill/SKILL.md` is not documentation: it is loaded into an agent's context
//! permanently, on every session, in every repository that installs it. That is
//! what makes its content the thing actually frozen (ADR-c656cbcc33a9) -- the
//! dispatch table refuses nobody, and the loop exists because this file teaches
//! it and teaches nothing else. Three properties are therefore worth a test
//! rather than a habit: that it carries the whole loop, that it stays small
//! enough to be worth loading, and that a copy in the wild says which revision
//! it is.
//!
//! This file exists because the task's declared verifier is `cargo-test`. A
//! criterion nothing executes is a criterion nobody checked, and a proof that
//! covers less than it appears to is worse than a missing one.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_file(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn skill() -> String {
    repo_file("skill/SKILL.md")
}

// ---------------------------------------------------------------------------
// §4's order, and the listing that has to match it (TASK-973fc0173b98)
// ---------------------------------------------------------------------------

/// The leading verb of a usage line, `ank <verb> ...`.
///
/// `ank --version` yields nothing and is meant to: it is not a verb, and §4
/// says so in as many words.
fn leading_verb(line: &str) -> Option<String> {
    let rest = line.strip_prefix("ank ")?;
    let verb: String = rest.chars().take_while(char::is_ascii_lowercase).collect();
    (!verb.is_empty()).then_some(verb)
}

fn push_once(verbs: &mut Vec<String>, verb: String) {
    if !verbs.contains(&verb) {
        verbs.push(verb);
    }
}

/// The verbs of §4's `Commands` block, in the order the block lists them.
///
/// Read from the specification rather than restated here: a second
/// hand-maintained copy of the order is the very drift this is checking for.
fn section_4_order() -> Vec<String> {
    let spec = repo_file("docs/ank-spec-v1.1.md");
    let mut verbs = Vec::new();
    let mut seen_heading = false;
    let mut inside = false;
    for line in spec.lines() {
        if line.trim() == "### Commands" {
            seen_heading = true;
            continue;
        }
        if !seen_heading {
            continue;
        }
        if line.trim_start().starts_with("```") {
            // The opening fence, then the closing one: the block is over.
            if inside {
                break;
            }
            inside = true;
            continue;
        }
        if inside {
            if let Some(v) = leading_verb(line) {
                push_once(&mut verbs, v);
            }
        }
    }
    assert!(
        !verbs.is_empty(),
        "the Commands block of §4 was not found: this test reads the \
         specification, so a renamed heading must fail loudly rather than pass \
         on an empty list"
    );
    verbs
}

/// The verbs `ank help` prints, in the order it prints them.
///
/// Through the binary, because ADR-c656cbcc33a9 is a statement about what the
/// process prints, not about the table it is derived from. The listing ends at
/// the blank line before `global:`.
fn help_order() -> Vec<String> {
    let out = Command::new(env!("CARGO_BIN_EXE_ank"))
        .arg("help")
        .output()
        .expect("the binary must have been built");
    assert!(out.status.success(), "ank help must succeed");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let mut verbs = Vec::new();
    for line in text.lines().take_while(|l| !l.trim().is_empty()) {
        if let Some(v) = leading_verb(line) {
            push_once(&mut verbs, v);
        }
    }
    assert!(!verbs.is_empty(), "ank help printed no verb");
    verbs
}

/// Verbs §4 specifies and the binary does not dispatch yet.
///
/// **Empty, and that is the state worth keeping.** Every verb §4 specifies now
/// ships. The list stays because it is the only place a future verb may be
/// declared missing, and because the test below fails the moment a declared one
/// starts shipping -- so the declaration cannot outlive the gap it describes.
///
/// `scope`, `graph`, `status` and `edit` were all here in turn, which is the
/// guard working as intended rather than a maintenance chore: shipping each of
/// them (TASK-e717ee625c5c, TASK-253e897d3330, TASK-15336a0012d5,
/// TASK-7ed19b16895e) turned the suite red until this line was edited, in the
/// same commit.
const NOT_YET_DISPATCHED: [&str; 0] = [];

/// A verb the binary answers to and §4 never mentions. `attest`, `init` and
/// `help` were exactly that until TASK-5c868c20472f, and a reader comparing the
/// two documents could not tell which one was wrong.
#[test]
fn every_dispatched_verb_is_listed_in_section_4() {
    let spec = section_4_order();
    for verb in help_order() {
        assert!(
            spec.contains(&verb),
            "`ank {verb}` is dispatched and §4's Commands block does not list \
             it: the specification is the source of truth (ADR-63b59c5c26f7), \
             so the block is what has to change"
        );
    }
}

/// The other direction, and the reason the exemption list stays honest. Every
/// verb §4 lists either ships or is declared unimplemented -- and a declared
/// one that has started shipping fails here until the declaration is removed.
#[test]
fn every_verb_section_4_lists_ships_or_is_declared_unimplemented() {
    let dispatched = help_order();
    for verb in section_4_order() {
        let exempt = NOT_YET_DISPATCHED.contains(&verb.as_str());
        let ships = dispatched.contains(&verb);
        assert!(
            ships || exempt,
            "§4 lists `ank {verb}`, the binary does not dispatch it, and it is \
             not in NOT_YET_DISPATCHED: either implement it or declare it there \
             with its task"
        );
        assert!(
            !(ships && exempt),
            "`ank {verb}` ships and is still declared unimplemented: remove it \
             from NOT_YET_DISPATCHED, in the commit that implemented it"
        );
    }
}

/// **One flat listing, in the order of §4** (ADR-c656cbcc33a9). Until this
/// test the ADR described the output rather than constraining it, and the two
/// orders agreed only while somebody remembered. Fixing the drift of
/// TASK-5c868c20472f introduced a fresh one in the same edit -- `attest` placed
/// after `check` where the binary prints it before -- and only a diff caught it.
#[test]
fn help_prints_section_4s_order() {
    let dispatched = help_order();
    let expected: Vec<String> = section_4_order()
        .into_iter()
        .filter(|v| dispatched.contains(v))
        .collect();
    assert_eq!(
        expected, dispatched,
        "`ank help` must print §4's order, minus what it does not dispatch"
    );
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

// ---------------------------------------------------------------------------
// Which revision is installed (TASK-b495234f192c)
// ---------------------------------------------------------------------------

/// Frontmatter and body, split on the same delimiters the entity format uses.
/// A second rule for a second file is a second thing to get wrong.
///
/// Line endings are unified first: `.gitattributes` covers `.ank/**` and not
/// `skill/`, so a Windows checkout of this file can legitimately be CRLF and
/// the closing delimiter is then `\r\n---\r\n`.
fn split_skill(text: &str) -> (String, String) {
    let lf = text.replace("\r\n", "\n");
    let rest = lf
        .strip_prefix("---\n")
        .expect("SKILL.md must open with frontmatter")
        .to_string();
    let end = rest
        .find("\n---\n")
        .expect("SKILL.md frontmatter must be closed");
    (
        rest[..end].to_string(),
        rest[end + "\n---\n".len()..].to_string(),
    )
}

/// The value the frontmatter declares under `metadata.revision`, unquoted.
fn declared_revision(front: &str) -> Option<String> {
    front.lines().find_map(|l| {
        let v = l.trim().strip_prefix("revision:")?;
        Some(v.trim().trim_matches('"').to_string())
    })
}

/// **A copy in the wild says which revision it is.** Measured on 2026-08-02:
/// the SKILL.md installed at `~/.claude/skills/ank` was byte-identical to the
/// blob at a004ac7, two commits and nine hours behind a tree that had just
/// withdrawn the invitation to read `.ank/` by hand (ADR-01b6dd05f0db). It was
/// not merely old, it instructed against a ratified decision -- and it carried
/// nothing by which its reader or its owner could have noticed.
///
/// The marker is a hash of the body rather than a version anyone keeps by hand,
/// for two reasons. A hand-kept number drifts the first time somebody edits the
/// body and forgets it, whereas this one cannot: the assertion below recomputes
/// it. And a date would not have caught the case above -- a004ac7 at 10:18 and
/// 7429cdd at 19:16 shipped on the same day, so a date-stamped stale copy would
/// have looked current.
///
/// It sits in `metadata`, which the Agent Skills standard defines as an
/// arbitrary map for properties the standard does not itself define, so it is
/// metadata about the file and not part of what the file teaches. That is the
/// whole of the freeze (ADR-c656cbcc33a9), and the three assertions above are
/// the enforcement -- none of them moves because of a fingerprint.
#[test]
fn the_skill_says_which_revision_it_is() {
    let text = skill();
    let (front, body) = split_skill(&text);

    let declared = declared_revision(&front).unwrap_or_else(|| {
        panic!(
            "SKILL.md declares no metadata.revision, so an installed copy \
             identifies itself to nobody"
        )
    });
    let actual = ank_core::freeze_hash_short(&body);

    assert_eq!(
        declared, actual,
        "SKILL.md was edited without its revision: set metadata.revision to \
         \"{actual}\""
    );
}
