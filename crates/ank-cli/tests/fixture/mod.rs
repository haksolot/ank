//! Writing and comparing a fixture of `tests/golden-json/`, for the suites that
//! capture one outside `cli.rs` (TASK-49b10f02d209).
//!
//! ADR-6fd69efb629c asks for a golden fixture per document the surface returns,
//! and `cli.rs` captures twenty-six of them. Two cannot be captured there. `read`
//! could be and is not, and `tui` cannot: its document only exists on a terminal
//! (`ank tui --json` refuses at exit 9 into a pipe), so the one place it can be
//! captured is the pseudo-terminal harness in `tui.rs`. An integration test is a
//! crate of its own, so `cli.rs`'s `golden` is not reachable from either suite,
//! and the choice was a second copy or a fixture nothing regenerates.
//!
//! **What is copied is the redaction, and it is copied deliberately.** The two
//! masks -- an instant, and an identifier the binary minted -- are what make a
//! captured document comparable at all, and a fixture written under a different
//! pair of masks would not be the same kind of file as the twenty-six beside it.
//! Folding `cli.rs` onto this module is TASK-7a9d945640e3 and is not done here:
//! that file is a perimeter three other agents are working in, and this task's
//! footprint in it is one number.

use std::path::Path;

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

/// Every instant, and every identifier the binary minted, named away.
///
/// A seeded identifier is deterministic and worth pinning; the zero prefix is
/// what a seeded corpus uses for one. Only what the run itself produced is
/// masked, so what survives is the shape and not the run.
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
            } else if whole && word.len() == 12 && !word.starts_with("0000") {
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

/// `tests/golden-json/<name>.json`, compared against what the process printed,
/// or written when `ANK_BLESS_GOLDEN` is set.
///
/// The same env var `cli.rs` blesses on, so one variable regenerates every
/// fixture in the directory whichever suite captured it.
pub fn pin(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden-json")
        .join(format!("{name}.json"));
    let actual = format!("{}\n", redact(actual.trim_end_matches('\n')));
    if std::env::var_os("ANK_BLESS_GOLDEN").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual.as_bytes()).unwrap();
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
