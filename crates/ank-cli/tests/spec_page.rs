//! `docs/specification.md` lists every accepted spec of this repository's own
//! corpus, and nothing else (TASK-6113526927b2).
//!
//! The site links the specification rather than rendering it
//! (ADR-33970fcdb6e8 leaves that choice to the restructuring task), so the page
//! carries one line per spec: its id, a link to its file, and its title. A
//! typed list of that kind is exactly what drifts, so it is held to the binary:
//! `ank find --type spec --json`, run on the workspace corpus, is the list, and
//! the page has to say the same ids with the same titles.

use std::path::{Path, PathBuf};
use std::process::Command;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const PAGE: &str = "docs/specification.md";
const BLOB: &str = "https://github.com/haksolot/ank/blob/main/.ank/entities/";

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The string value of `"key":"..."` in one JSON object, unescaped for the
/// two escapes a title can carry.
fn field(object: &str, key: &str) -> Option<String> {
    let start = object.find(&format!("\"{key}\":\""))? + key.len() + 4;
    let mut value = String::new();
    let mut chars = object[start..].chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => value.push(chars.next()?),
            '"' => return Some(value),
            c => value.push(c),
        }
    }
    None
}

/// `(id, title)` of every accepted spec, as the binary lists them.
fn accepted_specs() -> Vec<(String, String)> {
    let out = Command::new(ANK)
        .args(["find", "--type", "spec", "--json"])
        .current_dir(workspace())
        .env_remove("ANK_AGENT")
        .output()
        .expect("ank runs");
    assert!(
        out.status.success(),
        "ank find --type spec exited {:?}: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let doc = String::from_utf8(out.stdout).unwrap();
    assert!(
        doc.contains("\"hidden\":0"),
        "find hid some specs, so the list below is not the whole corpus: {doc}"
    );
    let mut specs: Vec<(String, String)> = doc
        .split("{\"id\":")
        .skip(1)
        .map(|o| format!("{{\"id\":{o}"))
        .filter(|o| field(o, "status").as_deref() == Some("accepted"))
        .map(|o| (field(&o, "id").unwrap(), field(&o, "title").unwrap()))
        .collect();
    specs.sort();
    specs
}

/// `(id, title)` of every line of the page that links a spec, with the link
/// checked against the id it shows.
fn listed_specs() -> Vec<(String, String)> {
    let text = std::fs::read_to_string(workspace().join(PAGE)).unwrap();
    let mut specs = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("- [SPEC-") else {
            continue;
        };
        let (id, rest) = rest.split_once("](").expect("a link after the id");
        let id = format!("SPEC-{id}");
        let (url, title) = rest.split_once(") ").expect("a title after the link");
        assert_eq!(
            url,
            format!("{BLOB}{id}.md"),
            "{PAGE}: the link for {id} opens another file"
        );
        specs.push((id, title.to_string()));
    }
    specs.sort();
    specs
}

#[test]
fn the_specification_page_lists_every_accepted_spec_and_nothing_else() {
    let corpus = accepted_specs();
    assert!(
        !corpus.is_empty(),
        "the corpus holds no accepted spec, so this test would compare nothing"
    );
    assert_eq!(
        listed_specs(),
        corpus,
        "{PAGE} and `ank find --type spec` disagree: list each accepted spec as \
         `- [<id>]({BLOB}<id>.md) <title>`"
    );
}
