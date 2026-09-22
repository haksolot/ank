//! The format reference's tables are what the commands print, byte for byte
//! (ADR-2b62b9a1fe67, TASK-46d3a4c56bf4).
//!
//! `docs/entity-fields.md` is the output of the `entity-fields` binary, which
//! renders the kind registry, the model's enums and the schema range;
//! `docs/config-keys.md` is the output of `config-keys`, which renders the
//! config.yml schema. These tests run both binaries, the commands a
//! contributor runs to regenerate the pages, and fail when a committed page
//! differs -- so a field added to the registry and not to the page turns the
//! suite red. The other tests pin what the commands print against values
//! written out here, rather than read back off the tables they render.

use ank_core::model::{AdrStatus, CriteriaBy, ProofType, ProofVia, TaskStatus};
use std::path::{Path, PathBuf};
use std::process::Command;

fn page_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs")
        .join(name)
}

fn run(exe: &str) -> String {
    let out = Command::new(exe).output().expect("the binary runs");
    assert!(out.status.success(), "{exe} exited {:?}", out.status);
    assert!(out.stderr.is_empty(), "{exe} wrote to stderr");
    String::from_utf8(out.stdout).expect("the binary prints UTF-8")
}

fn entity_fields() -> String {
    run(env!("CARGO_BIN_EXE_entity-fields"))
}

fn config_keys() -> String {
    run(env!("CARGO_BIN_EXE_config-keys"))
}

fn assert_page(name: &str, printed: String, bin: &str) {
    let regenerate = format!("cargo run -q -p ank-core --bin {bin} > docs/{name}");
    let page = std::fs::read_to_string(page_path(name))
        .unwrap_or_else(|e| panic!("docs/{name} unreadable ({e}); regenerate: {regenerate}"));
    // Git on Windows may check the page out with CRLF; the bytes that matter
    // are the lines.
    let page = page.replace("\r\n", "\n");
    assert!(
        page == printed,
        "docs/{name} differs from what the command prints; regenerate: {regenerate}"
    );
}

#[test]
fn the_entity_fields_page_is_what_the_command_prints() {
    assert_page("entity-fields.md", entity_fields(), "entity-fields");
}

#[test]
fn the_config_keys_page_is_what_the_command_prints() {
    assert_page("config-keys.md", config_keys(), "config-keys");
}

/// The field column of every table on the page under `heading`, in order.
fn fields_under(page: &str, heading: &str) -> Vec<String> {
    let mut lines = page.lines().skip_while(|l| *l != heading).skip(1);
    let mut fields = Vec::new();
    let mut in_table = false;
    for line in lines.by_ref() {
        if line.starts_with('#') {
            break;
        }
        if line.starts_with("| # ") {
            in_table = true;
            continue;
        }
        if in_table && line.starts_with("| ") {
            let cells: Vec<&str> = line.split(" | ").collect();
            fields.push(cells[1].trim_matches('`').to_string());
        } else if in_table && !line.starts_with("|---") {
            break;
        }
    }
    fields
}

#[test]
fn each_kind_is_printed_with_its_fields_in_canonical_order() {
    let page = entity_fields();
    assert_eq!(
        fields_under(&page, "## Task"),
        [
            "id",
            "type",
            "slug",
            "title",
            "created",
            "author",
            "status",
            "scope",
            "blocked_by",
            "done_criteria",
            "criteria_by",
            "verify",
            "method",
            "proof",
            "verified",
            "schema",
            "version"
        ]
    );
    assert_eq!(
        fields_under(&page, "## ADR"),
        [
            "id",
            "type",
            "slug",
            "title",
            "created",
            "author",
            "status",
            "scope",
            "constraint",
            "see",
            "supersedes",
            "ratified",
            "verified",
            "schema",
            "version"
        ]
    );
    assert_eq!(
        fields_under(&page, "## Spec"),
        [
            "id",
            "type",
            "slug",
            "title",
            "created",
            "author",
            "status",
            "scope",
            "references",
            "supersedes",
            "ratified",
            "verified",
            "schema",
            "version"
        ]
    );
    assert_eq!(
        fields_under(&page, "## Log entry"),
        [
            "id", "type", "slug", "title", "created", "author", "scope", "about", "seq", "records",
            "verified", "schema", "version"
        ]
    );
}

fn row<'a>(page: &'a str, heading: &str, first_cell: &str) -> &'a str {
    let start = page.find(&format!("\n{heading}\n")).expect(heading);
    page[start..]
        .lines()
        .find(|l| l.starts_with(&format!("| {first_cell} |")))
        .unwrap_or_else(|| panic!("no row '{first_cell}' under {heading}"))
}

#[test]
fn the_enums_and_the_schema_range_are_printed() {
    let page = entity_fields();
    assert!(
        page.contains("writes schema **4**, and reads schema **1** through **4**"),
        "{page}"
    );
    assert!(row(&page, "## Task", "7").contains("`open` \\| `in_progress` \\| `done` \\| `closed`"));
    assert!(row(&page, "## ADR", "7").contains("`proposed` \\| `accepted` \\| `superseded`"));
    assert!(row(&page, "## Task", "11").contains("`creator` \\| `claimer`"));
    assert!(row(&page, "## Task", "1").contains("`TASK-<12 hex>`"));
    assert!(row(&page, "## Log entry", "2").contains("always `log`"));
    assert!(row(&page, "## Task", "9").contains("always emitted"));
    assert!(row(&page, "## Task", "3").contains("omitted when absent"));
    for value in ["edit", "create", "method"] {
        row(&page, "## `records`", &format!("`{value}`"));
    }
    for value in ["test", "commit", "human-review", "assertion"] {
        row(&page, "## Proof `type`", &format!("`{value}`"));
    }
    assert!(row(&page, "## Proof `type`", "`assertion`").contains("| weak |"));
    assert!(row(&page, "## Proof `type`", "`test`").contains("| strong |"));
    for value in ["verifier", "attested", "submitted"] {
        row(&page, "## Proof `via`", &format!("`{value}`"));
    }
}

/// A variant added to one of these enums fails to compile here until it is
/// placed, and the page test then says the page has to follow.
#[test]
fn every_variant_is_on_the_page() {
    let page = entity_fields();
    let on_page = |v: &str| page.contains(&format!("`{v}`"));
    for s in [
        TaskStatus::Open,
        TaskStatus::InProgress,
        TaskStatus::Done,
        TaskStatus::Closed,
    ] {
        match s {
            TaskStatus::Open | TaskStatus::InProgress | TaskStatus::Done | TaskStatus::Closed => {
                assert!(on_page(s.as_str()), "{}", s.as_str())
            }
        }
    }
    for s in [
        AdrStatus::Proposed,
        AdrStatus::Accepted,
        AdrStatus::Superseded,
    ] {
        match s {
            AdrStatus::Proposed | AdrStatus::Accepted | AdrStatus::Superseded => {
                assert!(on_page(s.as_str()))
            }
        }
    }
    for c in [CriteriaBy::Creator, CriteriaBy::Claimer] {
        match c {
            CriteriaBy::Creator | CriteriaBy::Claimer => assert!(on_page(c.as_str())),
        }
    }
    for t in [
        ProofType::Test,
        ProofType::Commit,
        ProofType::HumanReview,
        ProofType::Assertion,
    ] {
        match t {
            ProofType::Test | ProofType::Commit | ProofType::HumanReview | ProofType::Assertion => {
                assert!(on_page(t.as_str()))
            }
        }
    }
    for v in [ProofVia::Verifier, ProofVia::Attested, ProofVia::Submitted] {
        match v {
            ProofVia::Verifier | ProofVia::Attested | ProofVia::Submitted => {
                assert!(on_page(v.as_str()))
            }
        }
    }
}

#[test]
fn every_config_key_is_printed_with_its_type_and_default() {
    let page = config_keys();
    let rows: Vec<Vec<String>> = page
        .lines()
        .filter(|l| l.starts_with("| `"))
        .map(|l| {
            l.trim_matches('|')
                .split(" | ")
                .map(|c| c.trim().to_string())
                .collect()
        })
        .collect();
    let key_type_default: Vec<(&str, &str, &str)> = rows
        .iter()
        .map(|r| (r[0].as_str(), r[1].as_str(), r[2].as_str()))
        .collect();
    assert_eq!(
        key_type_default,
        [
            ("`schema`", "integer", "required"),
            ("`context_budget`", "integer", "`8000`"),
            ("`claim_ttl_max`", "duration", "`2h`"),
            ("`claim_ttl_default`", "duration", "`30m`"),
            ("`default_branch`", "string", "none"),
            ("`peers.<name>`", "path", "none"),
            ("`verifiers.<name>.run`", "command", "required"),
            ("`verifiers.<name>.timeout`", "duration", "`10m`"),
            ("`verifiers.<name>.default`", "boolean", "`false`"),
            ("`roles.<name>.can`", "list of strings", "`[]`"),
            ("`roles.<name>.cannot`", "list of strings", "`[]`"),
            ("`identities.<identity>`", "string", "none"),
            ("`weight.hot_files`", "integer", "`3000`"),
            ("`weight.plane_bytes`", "integer", "`4000000`"),
        ]
    );
    assert!(page.contains("`schema: 1`"), "{page}");
}

/// The form a row declares is the form the serializer receives for that field,
/// over every valid golden fixture: a declared form the writer does not use
/// would make the page state an emission rule the files do not follow.
#[test]
fn each_declared_form_is_the_form_the_serializer_writes() {
    use ank_core::registry::{FieldValue, Fields, Form};
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/valid");
    let mut seen = std::collections::BTreeSet::new();
    let mut files: Vec<(PathBuf, String)> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
        .map(|p| {
            let text = std::fs::read_to_string(&p).unwrap();
            (p, text)
        })
        .collect();
    // No golden log entry carries a reading, and a field no file carries is a
    // field this test would say nothing about.
    files.push((
        PathBuf::from("<a log entry read by somebody>"),
        "---\nid: LOG-5a0c3e9d7b21\ntype: log\ntitle: tried the lock\n\
         created: 2026-07-26T17:03:00Z\nscope:\n  - src/**\nabout: TASK-8f3a91c2d4e7\n\
         seq: 0\nverified:\n  - by: human:marie\n    at: 2026-08-12T09:40:00Z\n\
         schema: 4\nversion: 1\n---\n"
            .to_string(),
    ));
    for (path, text) in files {
        let entity = ank_core::parse_entity(&text).unwrap();
        let kind = entity.kind_spec();
        for field in kind.fields {
            let Some(value) = entity.field_value(field.name) else {
                continue;
            };
            let written = match value {
                FieldValue::Bare(s) if s.parse::<u64>().is_ok() => Form::Integer,
                FieldValue::Bare(_) => Form::Bare,
                FieldValue::Scalar(_) => Form::Scalar,
                FieldValue::Block(_) => Form::Block,
                FieldValue::Flow(_) => Form::Flow,
                FieldValue::Seq(_) => Form::Seq,
                FieldValue::Proofs(_) | FieldValue::Readings(_) => Form::Maps,
            };
            assert_eq!(
                written,
                field.form,
                "{}: {} {}",
                path.display(),
                kind.name,
                field.name
            );
            seen.insert((kind.name, field.name));
        }
    }
    // Every field of every kind was measured by at least one fixture, or the
    // loop above proved nothing about it.
    let declared: std::collections::BTreeSet<_> = ank_core::KINDS
        .iter()
        .flat_map(|k| k.fields.iter().map(move |f| (k.name, f.name)))
        .collect();
    let unmeasured: Vec<_> = declared.difference(&seen).collect();
    assert!(unmeasured.is_empty(), "no fixture carries {unmeasured:?}");
}
