//! The Proof section of `CLAUDE.md`, and the two counts that hold it in place.
//!
//! `CLAUDE.md` is loaded into every agent's context in this repository before
//! it does anything, which makes its prose an instruction and not a note. The
//! Proof section used to read that no task in this corpus declares a `verify:`
//! list, and then hand over the route that absence forces -- `ank done --proof
//! commit:<sha>`. Every sentence was true and the paragraph was wrong: it
//! stated the thing to fix as a property of the corpus, and taught the close
//! nothing verifies. TASK-54c95c5f2d18 closed green that way while `cargo test
//! --workspace` was failing on three platforms.
//!
//! Two facts keep the repaired paragraph from drifting back, and neither is
//! safe as a habit:
//!
//! 1. `commit:<sha>` appears **once** in the whole file, inside the sentence
//!    naming the case that keeps `--proof` -- the criterion no declared
//!    verifier can settle. A second occurrence is the old route returning.
//! 2. Every verifier the section names is one `.ank/config.yml` declares.
//!    Naming a verifier the repository does not have teaches an `ank new
//!    --verify` that refuses at exit 7, which reads to whoever tries it as the
//!    guide being wrong about the tool.
//!
//! Both are counted here rather than reviewed, because a paragraph is exactly
//! the kind of thing a later edit walks back one sentence at a time.

use std::fs;
use std::path::PathBuf;

fn repo_file(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    let text = fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    // `CLAUDE.md` is not pinned to LF by `.gitattributes`, so a checkout on
    // Windows hands this test CRLF. What is asserted here is the characters,
    // and a line ending git chose is not one of them.
    text.replace("\r\n", "\n")
}

fn guide() -> String {
    repo_file("CLAUDE.md")
}

/// The body of one `##` section of `CLAUDE.md`, heading excluded.
fn section(text: &str, heading: &str) -> String {
    let open = format!("\n## {heading}\n");
    let from = text
        .find(&open)
        .unwrap_or_else(|| panic!("CLAUDE.md has no `## {heading}` section"))
        + open.len();
    let rest = &text[from..];
    let to = rest.find("\n## ").unwrap_or(rest.len());
    rest[..to].to_string()
}

/// The verifier names `.ank/config.yml` declares, read as the keys nested one
/// level under `verifiers:`.
///
/// Hand-scanned rather than parsed with a yaml crate: `ank-cli` has no yaml
/// dependency for its tests, and the shape being read is two levels of fixed
/// indentation that `ank config` writes and `check` validates.
fn declared_verifiers() -> Vec<String> {
    let text = repo_file(".ank/config.yml");
    let mut inside = false;
    let mut names = Vec::new();
    for line in text.lines() {
        if !line.starts_with(char::is_whitespace) {
            inside = line.trim_end() == "verifiers:";
            continue;
        }
        if !inside {
            continue;
        }
        // A key of the block itself is indented exactly two spaces; anything
        // deeper is `run:` or `timeout:` belonging to the verifier above it.
        let Some(rest) = line.strip_prefix("  ") else {
            continue;
        };
        if rest.starts_with(' ') {
            continue;
        }
        if let Some(name) = rest.split(':').next() {
            let name = name.trim();
            if !name.is_empty() {
                names.push(name.to_string());
            }
        }
    }
    assert!(
        !names.is_empty(),
        ".ank/config.yml declares no verifiers, so nothing below is being checked"
    );
    names
}

/// The whitespace-separated tokens of every backticked span in `text`.
///
/// A span is read token by token because the section's spans are commands:
/// `ank new task --verify cargo-test --verify fmt-check` carries two verifier
/// names inside one pair of backticks.
fn code_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('`') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('`') else { break };
        for token in rest[..close].split_whitespace() {
            tokens.push(token.to_string());
        }
        rest = &rest[close + 1..];
    }
    tokens
}

/// Whether a token is shaped like a verifier name: lowercase kebab-case, at
/// least one hyphen, and nothing that makes it a path, a flag or a reference.
///
/// This is what makes the second count possible at all. There is no marker in
/// the prose saying "this word is a verifier", so the shape has to do it, and
/// the shape is the one every name in `config.yml` has and no other backticked
/// token in the section does: `--proof` is a flag, `commit:<sha>` a reference,
/// `.ank/config.yml` a path, `ank done` two bare words.
fn looks_like_a_verifier(token: &str) -> bool {
    token.contains('-')
        && token.starts_with(|c: char| c.is_ascii_lowercase())
        && token
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

// ---------------------------------------------------------------------------
// Count one: `commit:<sha>` occurs once, where the remaining case is named
// ---------------------------------------------------------------------------

#[test]
fn commit_sha_occurs_exactly_once_in_the_guide() {
    let text = guide();
    let count = text.matches("commit:<sha>").count();
    assert_eq!(
        count, 1,
        "CLAUDE.md names `commit:<sha>` {count} time(s); it belongs in exactly one \
         sentence, the one describing the criterion no declared verifier can settle"
    );
}

#[test]
fn the_one_commit_sha_sits_in_the_sentence_that_keeps_proof() {
    let text = guide();
    let proof = section(&text, "Proof");
    assert!(
        proof.contains("commit:<sha>"),
        "the single `commit:<sha>` has left the Proof section"
    );

    // The sentence around it, taken between the full stops that bound it. Line
    // breaks are the paragraph's wrapping and not sentence boundaries, so the
    // section is flattened first.
    let flat = proof.replace('\n', " ");
    let at = flat.find("commit:<sha>").unwrap();
    let start = flat[..at].rfind(". ").map_or(0, |i| i + 2);
    let end = flat[at..].find(". ").map_or(flat.len(), |i| at + i + 1);
    let sentence = &flat[start..end];

    assert!(
        sentence.contains("--proof"),
        "`commit:<sha>` is no longer in a sentence about `--proof`: {sentence:?}"
    );
    assert!(
        sentence.contains("no declared verifier can settle"),
        "the sentence carrying `commit:<sha>` no longer names the case that keeps \
         `--proof` -- the criterion no declared verifier can settle: {sentence:?}"
    );
}

#[test]
fn the_guide_never_spells_the_close_it_stopped_teaching() {
    let text = guide();
    assert!(
        !text.contains("--proof commit:"),
        "CLAUDE.md hands `--proof commit:` back as the route a task closes by; \
         the route is `ank done` running the verifiers the task declares"
    );
    assert!(
        !text.contains("No task in this corpus declares"),
        "CLAUDE.md states the absence of `verify:` as a property of the corpus \
         again; it is the thing to fix, not a fact to work around"
    );
}

// ---------------------------------------------------------------------------
// Count two: every verifier the section names is declared
// ---------------------------------------------------------------------------

#[test]
fn every_verifier_the_proof_section_names_is_declared_in_config() {
    let declared = declared_verifiers();
    let proof = section(&guide(), "Proof");

    let named: Vec<String> = code_tokens(&proof)
        .into_iter()
        .filter(|t| looks_like_a_verifier(t))
        .collect();

    assert!(
        !named.is_empty(),
        "the Proof section names no verifier at all, so an agent reading it has \
         nothing to put in `verify:`"
    );

    let undeclared: Vec<&String> = named.iter().filter(|n| !declared.contains(n)).collect();
    assert_eq!(
        undeclared.len(),
        0,
        "the Proof section names {} verifier(s) `.ank/config.yml` does not declare: \
         {undeclared:?}; declared are {declared:?}",
        undeclared.len()
    );
}

#[test]
fn the_proof_section_teaches_declaring_verifiers_at_creation() {
    let proof = section(&guide(), "Proof");
    assert!(
        proof.contains("ank new task --verify"),
        "the Proof section no longer names the command that declares a task's \
         verifiers when it is written"
    );
    assert!(
        proof.contains("ank done"),
        "the Proof section no longer names the command that runs them"
    );
}
