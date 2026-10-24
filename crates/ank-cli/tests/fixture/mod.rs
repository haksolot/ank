//! The one redaction every fixture of `tests/golden-json/` is written under,
//! and the comparison that pins it (TASK-7a9d945640e3).
//!
//! ADR-6fd69efb629c asks for a golden fixture per document the surface returns,
//! and three suites capture them: `cli.rs` twenty-six, `schema.rs` `read`, and
//! `tui.rs` `tui`, whose document only exists on a terminal (`ank tui --json`
//! refuses at exit 9 into a pipe) so the pseudo-terminal harness is the one
//! place it can be captured. An integration test is a crate of its own, so a
//! function in `cli.rs` is not reachable from either of the others: this module
//! is compiled into all three, which is what makes one redaction serve them.
//!
//! **The masks are what make a captured document comparable at all**, and they
//! have to be the same masks for every file in the directory: an instant, and
//! an identifier the binary minted. A fixture written under a different pair
//! would not be the same kind of file as the twenty-seven beside it, and until
//! this module became the only copy nothing failed when the two drifted apart:
//! the shape ADR-6fd69efb629c spends its first page on -- four escapers that
//! did not agree -- one rung down, in the suite rather than in the tool.
//!
//! Captured from the process and never from a function. A fixture compared
//! against what an emitter returns proves the emitter, and what §4 promises is
//! what leaves the process.
//!
//! Volatile values are named rather than kept: an instant, a commit, an
//! identifier the binary generated. A golden that changes every run pins
//! nothing, and everything outside those three is compared byte for byte.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// `tests/golden-json/`, the directory every fixture lives in.
pub fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden-json")
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

/// `dddd-dd-ddTdd:dd:ddZ`, the only instant the format writes.
fn timestamp_len_at(b: &[char], i: usize) -> Option<usize> {
    const SHAPE: &str = "nnnn-nn-nnTnn:nn:nnZ";
    if i + SHAPE.len() > b.len() {
        return None;
    }
    for (k, want) in SHAPE.chars().enumerate() {
        let got = b[i + k];
        let ok = match want {
            'n' => got.is_ascii_digit(),
            c => got == c,
        };
        if !ok {
            return None;
        }
    }
    Some(SHAPE.len())
}

/// Every twelve-digit hex word some fixture of the directory names literally:
/// the seeded identifiers, the only ones a golden is written with.
fn literals() -> &'static HashSet<String> {
    static LITERALS: OnceLock<HashSet<String>> = OnceLock::new();
    LITERALS.get_or_init(|| {
        let mut set = HashSet::new();
        for entry in std::fs::read_dir(dir()).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let text = std::fs::read_to_string(path).unwrap();
            for word in text.split(|c: char| !is_word_char(c)) {
                if word.len() == 12 && word.chars().all(|c| c.is_ascii_hexdigit()) {
                    set.insert(word.to_string());
                }
            }
        }
        set
    })
}

/// Whether a twelve-digit word is a seeded identifier rather than a minted one.
///
/// A minted identifier is 48 random bits, so no shape of its digits sets it
/// apart: the zero prefix the seeded corpora use is one a minted id carries
/// once in 65536, and ADR-0000f14329b5 failed a run that way
/// (TASK-7f27dd117cfb). What a minted id cannot do, short of odds of 2^-48 per
/// literal, is equal one the goldens already name, so that is the test. A
/// seeded id no golden names yet is masked when first blessed; write it in by
/// hand and it is kept from then on.
fn is_seeded(word: &str) -> bool {
    literals().contains(word)
}

/// Every instant, and every identifier the binary minted, named away.
///
/// A seeded identifier is deterministic and worth pinning, and is kept; only
/// what the run itself produced is masked, so what survives is the shape and
/// not the run.
fn redact(s: &str) -> String {
    let b: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if let Some(n) = timestamp_len_at(&b, i) {
            out.push_str("<TIME>");
            i += n;
            continue;
        }
        if b[i].is_ascii_hexdigit() {
            let mut j = i;
            while j < b.len() && b[j].is_ascii_hexdigit() {
                j += 1;
            }
            let word: String = b[i..j].iter().collect();
            let whole =
                (i == 0 || !is_word_char(b[i - 1])) && (j == b.len() || !is_word_char(b[j]));
            if whole && word.len() == 40 {
                out.push_str("<SHA>");
            } else if whole && word.len() == 12 && !is_seeded(&word) {
                out.push_str("<HEX>");
            } else {
                out.push_str(&word);
            }
            i = j;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

/// The redaction alone, for a suite comparing against something other than a
/// fixture of this directory: the output blocks of the documentation, replayed
/// under the masks the goldens are written with (TASK-9e80c9a3a5ed).
#[allow(dead_code)]
pub fn redacted(s: &str) -> String {
    redact(s)
}

/// `tests/golden-json/<name>.json`, compared against what the process printed,
/// or written when `ANK_BLESS_GOLDEN` is set.
///
/// One env var regenerates every fixture in the directory, whichever suite
/// captured it, and under the one redaction they are all written with. Read the
/// diff a bless produces before committing it: that diff is the contract
/// changing.
pub fn pin(name: &str, actual: &str) {
    let path = dir().join(format!("{name}.json"));
    let actual = format!("{}\n", redact(actual.trim_end_matches('\n')));
    if std::env::var_os("ANK_BLESS_GOLDEN").is_some() {
        // Whole or not at all: another process reads every fixture for its
        // literals, and a half-written one would cost it a seeded id.
        let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&tmp, actual.as_bytes()).unwrap();
        std::fs::rename(&tmp, &path).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("no golden for {name} at {}: {e}", path.display()))
        .replace("\r\n", "\n");
    assert_eq!(
        actual, expected,
        "the --json document of `{name}` is not what the golden pins.\n\
         If the contract really changed, bless it and read the diff:\n  \
         ANK_BLESS_GOLDEN=1 cargo test --workspace"
    );
}

#[test]
fn a_minted_id_is_masked_whatever_its_digits_and_a_seeded_one_is_kept() {
    assert_eq!(redacted("ADR-0000f14329b5"), "ADR-<HEX>");
    assert_eq!(redacted("ADR-3c1f0e9a77b2"), "ADR-<HEX>");
    assert_eq!(redacted("TASK-000000000001"), "TASK-000000000001");
}
