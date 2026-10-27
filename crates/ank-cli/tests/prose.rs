//! The prose a person reads, held to the rules of ADR-0dd192525166.
//!
//! The pages are `docs/**/*.md`, `README.md`, `skill/SKILL.md` and
//! `skill/*/SKILL.md`. Three counts run over them:
//!
//! 1. Vale, pinned in mise.toml and configured by `.vale.ini` and the style
//!    under `.vale/ank/`, reports no alert at level error. Vale skips code, so
//!    an output block the binary prints is never read as prose.
//! 2. The contrastive negations of each page stay at or under its row in
//!    [`BUDGET`]. A row moves down in the pull request that rewrites its page,
//!    and nothing moves one up.
//! 3. A number standing before `skill`, `skills` or `SKILL.md` equals the
//!    skills under `skill/`, and one before `sibling` or `siblings` equals that
//!    count minus the contract.
//!
//! The first runs through `mise x`, the route test-ank-cli already takes, so
//! the rules hold in `cargo test` and in every required check with no job of
//! their own.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// One row per page: the count of [`NEGATIONS`] it held when the row was last
/// written. Lower a row when you rewrite its page; never raise one.
const BUDGET: &[(&str, usize)] = &[
    ("README.md", 3),
    ("docs/SUMMARY.md", 0),
    ("docs/alternatives.md", 10),
    ("docs/check.md", 8),
    ("docs/ci-jobs.md", 9),
    ("docs/ci.md", 14),
    ("docs/claims.md", 14),
    ("docs/config-keys.md", 7),
    ("docs/conventions.md", 9),
    ("docs/demo.md", 4),
    ("docs/entity-fields.md", 10),
    ("docs/environment.md", 8),
    ("docs/exit-codes.md", 3),
    ("docs/format.md", 80),
    ("docs/install.md", 2),
    ("docs/integrating.md", 48),
    ("docs/mcp.md", 22),
    ("docs/msrv.md", 5),
    ("docs/multi-agent.md", 10),
    ("docs/peers.md", 18),
    ("docs/proof.md", 13),
    ("docs/quickstart.md", 0),
    ("docs/ratifying.md", 9),
    ("docs/releasing.md", 3),
    ("docs/signing.md", 11),
    ("docs/specification.md", 2),
    ("docs/splitting-verifiers.md", 5),
    ("docs/watch.md", 24),
    ("skill/SKILL.md", 19),
    ("skill/diagnose/SKILL.md", 9),
    ("skill/drift/SKILL.md", 3),
    ("skill/herdr/SKILL.md", 8),
    ("skill/loop/SKILL.md", 11),
    ("skill/plan/SKILL.md", 4),
    ("skill/tdd/SKILL.md", 4),
];

/// `\b(not a|never|rather than|is not|are not|instead of|not the)\b`, the
/// pattern the ADR measured with, tried in this order at each position as the
/// regex alternation is.
const NEGATIONS: &[&str] = &[
    "not a",
    "never",
    "rather than",
    "is not",
    "are not",
    "instead of",
    "not the",
];

const NUMBER_WORDS: &[&str] = &[
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven",
    "twelve",
];

/// Every page the rules read, as a path relative to the root with `/`
/// separators, sorted.
fn pages() -> Vec<String> {
    fn walk(dir: &Path, rel: &str, out: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().into_string().unwrap();
            let path = format!("{rel}/{name}");
            if entry.file_type().unwrap().is_dir() {
                walk(&entry.path(), &path, out);
            } else if name.ends_with(".md") {
                out.push(path);
            }
        }
    }
    let root = root();
    let mut out = vec!["README.md".to_string(), "skill/SKILL.md".to_string()];
    walk(&root.join("docs"), "docs", &mut out);
    out.extend(skill_files());
    out.sort();
    out
}

/// `skill/*/SKILL.md`, the siblings of the contract.
fn skill_files() -> Vec<String> {
    let mut out = Vec::new();
    for entry in fs::read_dir(root().join("skill")).unwrap() {
        let entry = entry.unwrap();
        if entry.path().join("SKILL.md").is_file() {
            let name = entry.file_name().into_string().unwrap();
            out.push(format!("skill/{name}/SKILL.md"));
        }
    }
    out
}

fn read(page: &str) -> String {
    fs::read_to_string(root().join(page)).unwrap()
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn line_of(text: &str, at: usize) -> usize {
    text[..at].matches('\n').count() + 1
}

/// The matches of [`NEGATIONS`] in `text`, leftmost first and never
/// overlapping, each bounded by a word boundary on both sides.
fn negations(text: &str) -> usize {
    let mut count = 0;
    let mut at = 0;
    'scan: while at < text.len() {
        let before = text[..at].chars().next_back();
        if !before.is_some_and(is_word) {
            for phrase in NEGATIONS {
                let end = at + phrase.len();
                if text[at..].starts_with(phrase)
                    && !text[end..].chars().next().is_some_and(is_word)
                {
                    count += 1;
                    at = end;
                    continue 'scan;
                }
            }
        }
        at += text[at..].chars().next().unwrap().len_utf8();
    }
    count
}

/// Every number word or digit run standing before a skill noun, with the noun
/// and the line: `(number, noun, line)`, the noun lowercased.
fn counts(text: &str) -> Vec<(usize, String, usize)> {
    let words: Vec<(usize, &str)> = text
        .match_indices(|c: char| !is_word(c))
        .fold((Vec::new(), 0), |(mut v, start), (i, sep)| {
            if i > start {
                v.push((start, &text[start..i]));
            }
            (v, i + sep.len())
        })
        .0;
    let mut out = Vec::new();
    for pair in words.windows(2) {
        let ((at, number), (next_at, noun)) = (pair[0], pair[1]);
        let gap = &text[at + number.len()..next_at];
        if gap.is_empty() || !gap.chars().all(char::is_whitespace) {
            continue;
        }
        let value = if number.chars().all(|c| c.is_ascii_digit()) {
            number.parse().ok()
        } else {
            NUMBER_WORDS
                .iter()
                .position(|w| w.eq_ignore_ascii_case(number))
                .map(|i| i + 1)
        };
        let noun = noun.to_lowercase();
        if !matches!(noun.as_str(), "skill" | "skills" | "sibling" | "siblings") {
            continue;
        }
        if let Some(value) = value {
            out.push((value, noun, line_of(text, at)));
        }
    }
    out
}

#[test]
fn vale_reports_no_error_on_any_page() {
    let root = root();
    let mise = Command::new("mise").arg("--version").output();
    if !mise.is_ok_and(|o| o.status.success()) {
        eprintln!(
            "SKIPPED: mise is not installed, so the Vale rules of ADR-0dd192525166 did not run: \
             https://mise.jdx.dev/installing-mise.html"
        );
        return;
    }
    // `mise x` runs what is installed and fetches nothing, and ci.yml installs
    // nextest alone, so Vale is installed here, from mise.lock.
    let install = Command::new("mise")
        .args(["install", "--locked", "vale"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        install.status.success(),
        "mise install --locked vale failed:\n{}",
        String::from_utf8_lossy(&install.stderr)
    );
    let pages = pages();
    let out = Command::new("mise")
        .args(["x", "--", "vale", "--output=JSON"])
        .args(&pages)
        .current_dir(&root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Vale exits 1 when it reports an error and 2 when it could not run.
    assert!(
        matches!(out.status.code(), Some(0 | 1)),
        "vale did not run ({:?}):\n{stdout}\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let report: serde_yaml::Mapping = serde_yaml::from_str(&stdout)
        .unwrap_or_else(|e| panic!("vale printed no JSON ({e}):\n{stdout}"));
    let mut errors = Vec::new();
    for (file, alerts) in &report {
        for alert in alerts.as_sequence().unwrap() {
            if alert["Severity"].as_str() == Some("error") {
                errors.push(format!(
                    "{}:{}: {}: {}",
                    file.as_str().unwrap(),
                    alert["Line"].as_u64().unwrap(),
                    alert["Check"].as_str().unwrap(),
                    alert["Message"].as_str().unwrap(),
                ));
            }
        }
    }
    errors.sort();
    assert!(
        errors.is_empty(),
        "{} error(s) under .vale/ank:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

#[test]
fn every_page_keeps_its_negations_within_its_budget() {
    let pages = pages();
    let mut faults = Vec::new();
    for page in &pages {
        let count = negations(&read(page));
        match BUDGET.iter().find(|(p, _)| p == page) {
            None => faults.push(format!(
                "{page} has no row in BUDGET: add (\"{page}\", {count})"
            )),
            Some((_, budget)) if count > *budget => faults.push(format!(
                "{page} holds {count} contrastive negations, over its budget of {budget}: \
                 rewrite the sentences, the row only moves down"
            )),
            Some(_) => {}
        }
    }
    for (page, _) in BUDGET {
        if !pages.iter().any(|p| p == page) {
            faults.push(format!(
                "BUDGET names {page}, which is no page: remove the row"
            ));
        }
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}

#[test]
fn every_skill_count_equals_the_skills_under_skill() {
    let skills = 1 + skill_files().len();
    let mut faults = Vec::new();
    for page in pages() {
        for (number, noun, line) in counts(&read(&page)) {
            let expected = if noun.starts_with("sibling") {
                skills - 1
            } else {
                skills
            };
            if number != expected {
                faults.push(format!(
                    "{page}:{line}: {number} {noun}, where skill/ holds {skills} skills, \
                     the contract and {} siblings",
                    skills - 1
                ));
            }
        }
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}
