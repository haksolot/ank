//! Conformance suite for the Ank format.
//!
//! Any third-party tool that claims to read or write the format can reuse the
//! `tests/golden/` directory. Its shape mirrors `.ank/` itself, so that porting
//! it is reading a directory rather than learning a convention:
//!
//! ```text
//! golden/
//!   valid/<ID>.md         entity files: must parse, and round-trip byte for
//!                         byte once normalised
//!   valid/log/<ID>.md     the log of the entity with the same id: every line
//!                         must match the grammar
//!   invalid/<name>.md     entity files: each rejected with its own error
//!   invalid/log/<name>.md log files: each rejected with its own error
//! ```
//!
//! A log is paired with its entity by **id and nothing else** — the same rule
//! the store applies between `.ank/entities/<ID>.md` and `.ank/log/<ID>.md`
//! (§6). An entity with no log file has an empty log, never an error.
//!
//! "Once normalised" carries one file: `TASK-c71f0e5a9b23.md` is in CRLF on
//! purpose and must come back in LF, because the format is read in either and
//! written in one (§3).
//!
//! There is deliberately **no invalid fixture for a malformed actor**. The
//! convention of §3 (`human:<id>`, `<producer>/<version>`, `process:<id>`) is a
//! `check` finding and never a parse error (ADR-3877fef1d662): enforcing it
//! here would lock every file written before it out of its own format. What
//! guards that decision is a positive assertion instead, below.

use ank_core::log::MESSAGE_LINE_MAX;
use ank_core::*;
use std::fs;
use std::path::PathBuf;

fn golden_dir(sub: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(sub)
}

/// Entity fixtures only: `log/` is a directory of a different kind of file and
/// is walked by its own tests.
fn entity_fixtures(sub: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(golden_dir(sub))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    paths
}

fn log_fixtures(sub: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(golden_dir(sub).join("log"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    paths
}

/// The round-trip is the identity **on canonical form** (§3). Valid but
/// non-canonical input is read correctly and normalised on first rewrite, so
/// the assertion is byte identity against the *normalised* input: for a file
/// already canonical that is the file itself, and the old guarantee is
/// unweakened. For one that is not, demanding it come back unchanged would be
/// demanding that ank write CRLF, which §3 forbids in as many words.
///
/// Schema 1 and schema 2 fixtures are in here to stay. A file written before a
/// field existed must survive a rewrite unchanged; if one of them ever moves,
/// the version bump has silently become a migration.
#[test]
fn valid_files_round_trip_byte_identical_once_canonical() {
    let mut checked = 0;
    let mut crlf_seen = 0;
    let mut schemas = Vec::new();
    for path in entity_fixtures("valid") {
        let input = fs::read_to_string(&path).unwrap();
        let entity = parse_entity(&input)
            .unwrap_or_else(|e| panic!("{} should be valid: {e}", path.display()));
        let output = serialize_entity(&entity);

        if has_crlf(&input) {
            crlf_seen += 1;
            assert!(
                !has_crlf(&output),
                "{} came back with CRLF: read, never written",
                path.display()
            );
        }
        assert_eq!(
            normalise_line_endings(&input),
            output,
            "round-trip differs for {}",
            path.display()
        );
        schemas.push(entity.schema());
        checked += 1;
    }
    assert!(checked >= 10, "valid golden files are missing");
    // Guards the fixture against .gitattributes: if git ever normalises it on
    // checkout, this test stops covering CRLF and says so instead of passing.
    assert_eq!(
        crlf_seen, 1,
        "the CRLF fixture is missing or was converted to LF on checkout"
    );
    // Every version in the reader range is represented, so that no bump can be
    // shipped without a fixture proving the older ones still round-trip.
    for v in model::MIN_SCHEMA..=SCHEMA_VERSION {
        assert!(
            schemas.contains(&v),
            "no valid fixture at schema {v}: the reader range is not covered"
        );
    }
}

/// The diagnostic a CRLF file earns is the one that names line endings, never
/// "missing frontmatter". That substitution is the whole reason this exists:
/// `---\r\n` reported as missing frontmatter sends the reader to look for a
/// delimiter that is right there.
#[test]
fn crlf_is_read_and_never_diagnosed_as_missing_frontmatter() {
    let path = golden_dir("valid").join("TASK-c71f0e5a9b23.md");
    let input = fs::read_to_string(&path).unwrap();
    assert!(
        has_crlf(&input),
        "the fixture must carry CRLF to mean anything"
    );

    // It parses. Before this task it did not, and the error was the wrong one.
    let entity = parse_entity(&input).expect("CRLF must be read, not rejected");

    // The body crossed intact apart from its line endings.
    let out = serialize_entity(&entity);
    assert!(!has_crlf(&out));
    assert!(out.contains("## Log\n- 2026-07-28T00:22:06Z"));

    // And the diagnostic that describes it names the cause, with the command:
    // the file cannot be fixed by editing it while git converts on checkout.
    let d = Error::CrlfLineEndings.to_string();
    assert!(d.contains("CRLF"), "{d}");
    assert!(d.contains("git config core.autocrlf input"), "{d}");
    assert!(!d.contains("missing frontmatter"), "{d}");
}

#[test]
fn invalid_files_are_rejected_with_the_right_error() {
    for path in entity_fixtures("invalid") {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let input = fs::read_to_string(&path).unwrap();
        let err = parse_entity(&input)
            .err()
            .unwrap_or_else(|| panic!("{name} should be rejected"));
        let ok = match name.as_str() {
            "missing-scope" => matches!(err, Error::EmptyScope),
            "bad-schema" => matches!(err, Error::UnknownSchema { found: 99, .. }),
            "bad-status" => matches!(err, Error::Yaml(_)),
            "bad-id" => matches!(err, Error::InvalidId(_)),
            "type-mismatch" => matches!(err, Error::TypeMismatch { .. }),
            "unknown-field" => matches!(err, Error::Yaml(_)),
            "no-frontmatter" => matches!(err, Error::MissingFrontmatter),
            "criteria-by-without-criteria" => matches!(err, Error::CriteriaByWithoutCriteria),
            "bad-glob" => matches!(err, Error::InvalidGlob(_)),
            // The kind is named, not the id prefix and not the first field the
            // kind happens to carry. A reader told "invalid identifier" goes
            // hunting for a typo in the hex.
            "unknown-kind" => matches!(&err, Error::UnknownKind { kind } if kind == "epic"),
            // `via` is a closed set. The absent field is the only way to say
            // "no route recorded", so a value outside the set is a defect and
            // never a route this reader has not heard of.
            "bad-proof-via" => matches!(&err, Error::Yaml(_)) && err.to_string().contains("via"),
            // A `verified` list is optional; an entry in it is not partial.
            "verified-without-at" => {
                matches!(&err, Error::Yaml(_)) && err.to_string().contains("missing field `at`")
            }
            // The absence that justifies the kind, refused by name. A spec
            // describes and an ADR binds, so a `constraint` inside a spec is
            // not a field this kind has yet to learn: it is the one field the
            // kind exists in order not to carry, and the diagnostic says which
            // field rather than which kind.
            "spec-with-constraint" => {
                matches!(&err, Error::Yaml(_)) && err.to_string().contains("constraint")
            }
            // A log entry with no subject. Since the move, the entity an entry
            // is about is a field and no longer arithmetic on the id, so the
            // absent field is the association missing entirely — named, rather
            // than defaulted to the entity that happens to be nearby.
            "log-without-about" => {
                matches!(&err, Error::Yaml(_)) && err.to_string().contains("missing field `about`")
            }
            // A log entry with no rank. The order of an entity's entries is
            // `created`, then `seq`, then the identifier, and a timestamp alone
            // is not an order -- several entries in one second is the ordinary
            // case. An absent `seq` is therefore the order missing, not a
            // default to invent, and it is refused by name.
            "log-without-seq" => {
                matches!(&err, Error::Yaml(_)) && err.to_string().contains("missing field `seq`")
            }
            other => panic!("invalid file not covered: {other}"),
        };
        assert!(ok, "{name}: wrong error: {err:?}");
    }
}

/// Schema 3 is two things and this fixture carries both: the actors are typed,
/// and the body holds no log because the log is a file.
#[test]
fn schema_three_carries_typed_actors_and_a_reading() {
    let input = fs::read_to_string(golden_dir("valid").join("TASK-2f8c41ba07d3.md")).unwrap();
    let t = parse_task(&input).unwrap();
    assert_eq!(t.schema, 3);
    assert_eq!(t.author.as_deref(), Some("claude-code/1.4.2"));

    assert_eq!(t.verified.len(), 2);
    assert_eq!(t.verified[0].by, "human:marie");
    assert_eq!(t.verified[0].at, "2026-08-12T09:40:00Z");
    assert_eq!(t.verified[1].by, "process:ci");

    // The three forms of the convention, in one corpus.
    assert!(t.author.as_deref().unwrap().contains('/'));
    assert!(t.verified[0].by.starts_with("human:"));
    assert!(t.verified[1].by.starts_with("process:"));

    // No log in the body, and none inferred from its absence. The test is on a
    // *line* being the heading, not on the text appearing anywhere: a body is
    // free-form markdown and may perfectly well discuss `## Log` in prose, as
    // this one does.
    assert!(
        !t.body.lines().any(|l| l.trim_end() == "## Log"),
        "the log left the body at schema 3"
    );
    assert!(parse_log(&t.body).is_empty());
}

#[test]
fn an_adr_carries_a_reading_too() {
    let input = fs::read_to_string(golden_dir("valid").join("ADR-5e1a9d7c30b4.md")).unwrap();
    let a = parse_adr(&input).unwrap();
    assert_eq!(a.schema, 3);
    assert_eq!(a.status, AdrStatus::Accepted);
    assert_eq!(a.author.as_deref(), Some("human:marie"));
    assert_eq!(a.verified.len(), 1);
    assert_eq!(a.verified[0].by, "human:marie");
    // `verified` sits between `ratified` and `schema`, and the round-trip test
    // is what proves the position rather than this one.
    assert_eq!(a.ratified.as_deref(), Some("9f2b41c70de8"));
}

/// Goldens are the specification made executable, and this is the assertion
/// that keeps them so: a row added to the registry with no fixture beside it
/// fails here rather than shipping a kind whose round trip nobody has run.
#[test]
fn every_kind_in_the_registry_has_a_valid_fixture() {
    let mut covered: Vec<&str> = Vec::new();
    for path in entity_fixtures("valid") {
        let input = fs::read_to_string(&path).unwrap();
        let entity = parse_entity(&input).unwrap();
        let name = entity.kind_spec().name;
        if !covered.contains(&name) {
            covered.push(name);
        }
    }
    for kind in KINDS {
        assert!(
            covered.contains(&kind.name),
            "no valid golden fixture of kind {}: the registry declares a kind the suite never reads",
            kind.name
        );
    }
}

/// A spec declares no `constraint`, and the whole justification for the kind is
/// that absence (§3): a spec describes, an ADR binds. The positive half is that
/// it carries the rest of a decision's lifecycle — a status, a supersession, a
/// ratification anchor — so the document moves the way every other decision
/// does.
#[test]
fn a_spec_carries_a_lifecycle_and_declares_no_constraint() {
    let input = fs::read_to_string(golden_dir("valid").join("SPEC-19c4f0a83b2e.md")).unwrap();
    let s = parse_spec(&input).unwrap();
    assert_eq!(s.schema, 3);
    assert_eq!(s.status, SpecStatus::Accepted);
    assert_eq!(
        s.supersedes.as_ref().unwrap().to_string(),
        "SPEC-c07d1b4a92e5"
    );
    assert_eq!(s.ratified.as_deref(), Some("9f2c81b0"));
    assert_eq!(s.verified.len(), 1);

    // The absence, stated on the table rather than on this one file: nothing in
    // the kind's row can carry a constraint, so no spec can.
    let row = s.kind_spec();
    assert!(!row.fields.iter().any(|f| f.name == "constraint"));
    // On the frontmatter and not on the file: a body is free-form markdown and
    // this one discusses the absence in prose, exactly as the schema 3 task
    // fixture discusses `## Log`.
    let out = serialize_spec(&s);
    let frontmatter = out.split("\n---\n").next().unwrap();
    assert!(!frontmatter
        .lines()
        .any(|l| l.starts_with("constraint:") || l.starts_with("see:")));

    // And the document is the body, which is what `context` names and never
    // quotes: there is no field for it to serve.
    assert!(s.body.contains("The document itself."));
}

/// A log entry names the entity it is about, in a field. That is what the
/// previous shape computed from the id instead, and the trade — an address
/// becomes a query — is the cost the kind was accepted with (ADR-25f977377fa0).
#[test]
fn a_log_entry_names_the_entity_it_is_about() {
    let input = fs::read_to_string(golden_dir("valid").join("LOG-6b0f39d7a4c1.md")).unwrap();
    let l = parse_log_entity(&input).unwrap();
    assert_eq!(l.about.to_string(), "TASK-8f3a91c2d4e7");
    assert_eq!(l.about.kind(), EntityKind::Task);

    // The four things the rendered line prints are four fields, and the message
    // is the title rather than a field of its own.
    assert_eq!(l.created, "2026-07-26T14:02:00Z");
    assert_eq!(l.author.as_deref(), Some("claude-code/1.4.2"));
    assert_eq!(l.title, "jwt.verify removed from session.ts");

    // No status on the row, because an entry has nothing to transition to.
    assert!(!l.kind_spec().fields.iter().any(|f| f.name == "status"));
    assert!(!serialize_log_entity(&l).contains("status:"));

    // The subject exists in the corpus the suite reads, which is what makes the
    // association a query that resolves rather than a string.
    assert!(golden_dir("valid")
        .join(format!("{}.md", l.about))
        .is_file());
}

/// A proof entry says which route it arrived by, and the entries that predate
/// the field say nothing rather than something wrong (ADR-b6b69053a47b).
///
/// The negative half is the one that matters, and it is asserted on the schema
/// 2 fixture: its proof entries were written before `via` existed, they hold
/// `None`, and the round-trip test above is what proves they do not acquire one
/// on a rewrite. A reader that filled the absence in with a default would be
/// reinterpreting a corpus it postdates — which is the same reading §3 gives to
/// pre-convention actors and to entities predating `author`.
#[test]
fn a_proof_records_its_route_and_an_older_one_records_none() {
    let input = fs::read_to_string(golden_dir("valid").join("TASK-2f8c41ba07d3.md")).unwrap();
    let t = parse_task(&input).unwrap();
    assert_eq!(t.proof.len(), 2);
    assert_eq!(t.proof[0].via, Some(ProofVia::Verifier));
    assert_eq!(t.proof[1].via, Some(ProofVia::Submitted));
    // Route and type are orthogonal: the verifier entry is what anchors, the
    // submitted `commit` is not a `test` at all.
    assert!(t.proof[0].anchors_externally());
    assert!(!t.proof[1].anchors_externally());

    let older = fs::read_to_string(golden_dir("valid").join("TASK-8f3a91c2d4e7.md")).unwrap();
    let older = parse_task(&older).unwrap();
    assert_eq!(older.schema, 2);
    assert!(
        older.proof.iter().all(|p| p.via.is_none()),
        "an entry written before the field must carry no route"
    );
    // And it counts as it always counted. Absent is not `submitted`.
    assert!(older.proof.iter().all(|p| p.anchors_externally()));
    assert!(!serialize_entity(&Entity::Task(older)).contains("via:"));
}

/// `verified` is optional on every kind and its absence is never a fault. The
/// schema 1 and schema 2 fixtures carry none, and must not acquire one on a
/// rewrite — which is the round-trip test above, stated here as intent.
#[test]
fn a_reading_is_optional_and_absence_is_not_a_fault() {
    let input = fs::read_to_string(golden_dir("valid").join("TASK-51c2a7f0b3d9.md")).unwrap();
    let t = parse_task(&input).unwrap();
    assert_eq!(t.schema, 1);
    assert!(t.verified.is_empty());
    assert!(!serialize_entity(&Entity::Task(t)).contains("verified"));
}

/// The convention of §3 is a signal and not a wall. A value that does not match
/// it parses, and `check` is what reports it: the corpus is not migrated by a
/// rule it predates, and 96 files in this repository predate this one.
#[test]
fn a_pre_convention_actor_is_read_and_never_refused() {
    let input = fs::read_to_string(golden_dir("valid").join("TASK-8f3a91c2d4e7.md")).unwrap();
    let t = parse_task(&input).unwrap();
    assert_eq!(t.author.as_deref(), Some("claude-code@host-3"));

    let a = fs::read_to_string(golden_dir("valid").join("ADR-3c7e0b9142af.md")).unwrap();
    assert_eq!(
        parse_adr(&a).unwrap().author.as_deref(),
        Some("marie@laptop")
    );
}

// ---------------------------------------------------------------------------
// The log, now a file of its own
// ---------------------------------------------------------------------------

/// Every valid log fixture parses, and its name is the id of an entity that
/// exists. The pairing is the whole addressing scheme: no lookup, no index,
/// the same id on both sides.
#[test]
fn a_log_file_is_paired_with_its_entity_by_id_alone() {
    let mut checked = 0;
    for path in log_fixtures("valid") {
        let name = path.file_name().unwrap();
        assert!(
            golden_dir("valid").join(name).is_file(),
            "{} names no entity: a log is keyed by the id of the entity it belongs to",
            path.display()
        );
        let input = fs::read_to_string(&path).unwrap();
        let entries = parse_log_file(&input)
            .unwrap_or_else(|e| panic!("{} should be a valid log: {e}", path.display()));
        assert!(!entries.is_empty(), "{} is empty", path.display());
        checked += 1;
    }
    assert!(checked >= 1, "the log fixture is missing");
}

/// The line grammar does not change: a dash, the timestamp, the identity, an em
/// dash, the message. Nothing about an entry written before the move is
/// reinterpreted — only its address moved.
#[test]
fn the_log_line_grammar_is_unchanged() {
    let path = golden_dir("valid").join("log").join("TASK-2f8c41ba07d3.md");
    let entries = parse_log_file(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(entries.len(), 4);

    assert_eq!(entries[0].timestamp, "2026-08-12T09:14Z");
    assert_eq!(entries[0].who, "claude-code/1.4.2");
    assert_eq!(
        entries[0].message,
        "rotation job scheduled, previous key retained"
    );
    // A typed actor is carried verbatim, exactly as an untyped one is.
    assert_eq!(entries[1].who, "human:marie");
    assert_eq!(entries[2].who, "process:ci");

    // And the line comes back as it went in.
    assert_eq!(
        entries[0].format_line(),
        "- 2026-08-12T09:14Z claude-code/1.4.2 — rotation job scheduled, previous key retained"
    );
}

/// A discrepancy recorded against a frozen criterion is a log entry and nothing
/// else (§3): the record is the opening of the message, the line grammar does
/// not move, and the file carrying the frozen field is never opened to write it.
#[test]
fn a_discrepancy_is_a_log_message_and_never_a_field() {
    let entity_path = golden_dir("valid").join("TASK-2f8c41ba07d3.md");
    let entity_before = fs::read_to_string(&entity_path).unwrap();
    let t = parse_task(&entity_before).unwrap();
    let criteria = t.done_criteria.clone().unwrap();
    let anchor = freeze_hash_short(&criteria);

    let log_path = golden_dir("valid").join("log").join("TASK-2f8c41ba07d3.md");
    let log = fs::read_to_string(&log_path).unwrap();
    let entries = parse_log_file(&log).unwrap();

    // The opening is the whole of the recognition, and it recognises one entry
    // of the four: an ordinary line is an ordinary line.
    let recorded: Vec<&str> = entries.iter().filter_map(|e| e.discrepancy()).collect();
    assert_eq!(recorded.len(), 1, "one record among ordinary entries");
    assert!(
        recorded[0].starts_with("the criterion assumes"),
        "what follows the opening is returned verbatim: {}",
        recorded[0]
    );
    assert!(entries[..3].iter().all(|e| e.discrepancy().is_none()));

    // Nothing about the line moved. The convention lives in the message,
    // exactly where `released:` lives, so the entry round-trips as any other.
    assert!(entries[3].message.starts_with(ank_core::log::DISCREPANCY));
    assert_eq!(entries[3].who, "claude-code/1.4.2");
    assert_eq!(entries[3].format_line(), log.lines().last().unwrap());

    // And the freeze is untouched by it. Recording another one is one more
    // entity; the file carrying `done_criteria` is not written at all, so the
    // hash a claim anchored still verifies the criterion it froze. The
    // convention survives the move because it lives in the message, and the
    // message is the title of the entry that carries it.
    let recorded = format!(
        "{} the incident runbook says the same thing",
        ank_core::log::DISCREPANCY
    );
    let (title, body) = message_fields(&recorded);
    let entry = LogEntry {
        timestamp: "2026-08-12T09:50Z".into(),
        who: "human:marie".into(),
        message: message_of(&title, &body),
    };
    assert_eq!(
        entry.discrepancy(),
        Some("the incident runbook says the same thing")
    );
    assert_eq!(fs::read_to_string(&entity_path).unwrap(), entity_before);
    assert!(verify_frozen(&criteria, &anchor));

    // The other design, stated as the absence it is: the entity the record is
    // about carries no trace of it, because the format has no field to carry.
    assert!(!entity_before.contains("discrepancy"));
}

/// A missing log file is an empty log, never an error. The store answers the
/// missing-file half (§6); the parser answers the empty-content half.
#[test]
fn a_missing_log_is_an_empty_log_and_never_an_error() {
    assert!(
        !golden_dir("valid")
            .join("log")
            .join("TASK-51c2a7f0b3d9.md")
            .exists(),
        "this fixture must stay logless for the assertion to mean anything"
    );
    assert!(parse_log_file("").unwrap().is_empty());
}

#[test]
fn invalid_log_files_are_rejected_with_the_right_error() {
    for path in log_fixtures("invalid") {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let input = fs::read_to_string(&path).unwrap();
        let err = parse_log_file(&input)
            .err()
            .unwrap_or_else(|| panic!("{name} should be rejected"));
        let ok = match name.as_str() {
            // A file whose entire content is the log has no room for prose: a
            // line that is not an entry is a defect, and the diagnostic names
            // which line rather than which file.
            "bad-log-line" => matches!(err, Error::MalformedLogLine { line: 2 }),
            other => panic!("invalid log file not covered: {other}"),
        };
        assert!(ok, "{name}: wrong error: {err:?}");
    }
}

/// The property the move was made to protect, asserted on what replaced the
/// append: **the message survives the two fields it is stored in**, whatever
/// its length, and comes back byte for byte (ADR-25f977377fa0).
///
/// Read off the previous layout's own fixture, so the corpus under test is the
/// one the migration has to move rather than a message invented here.
#[test]
fn every_message_of_the_previous_layout_survives_becoming_an_entity() {
    for path in log_fixtures("valid") {
        let text = fs::read_to_string(&path).unwrap();
        for entry in parse_log_file(&text).unwrap() {
            let (title, body) = message_fields(&entry.message);
            assert_eq!(
                message_of(&title, &body),
                entry.message,
                "{}: a message altered on the way into an entity",
                path.display()
            );
            assert!(!title.contains('\n'), "{}: a title is one line", title);
        }
    }

    // The fixture that carries a split message reads it back whole, and the
    // two halves are the two fields rather than a rule stated only in prose.
    let split = parse_log_entity(
        &fs::read_to_string(golden_dir("valid").join("LOG-04c8e2f1a7b3.md")).unwrap(),
    )
    .unwrap();
    let whole = split.message();
    assert!(whole.chars().count() > MESSAGE_LINE_MAX, "{whole}");
    assert_eq!(
        message_fields(&whole),
        (split.title.clone(), split.body.clone())
    );
    assert_eq!(
        split.title.chars().count() <= MESSAGE_LINE_MAX,
        true,
        "the title is bounded: {}",
        split.title
    );
    let rendered = LogEntry::of(&split);
    let recorded = rendered
        .discrepancy()
        .expect("a convention on the message survives the split, because it is one message");
    assert_eq!(
        recorded,
        &whole["discrepancy: ".len()..],
        "the opening is the whole of the recognition, and it spans both fields"
    );

    // And a message built here, longer still, reads back whole through the
    // model rather than through the two halves.
    let long = format!("discrepancy: {}", "measured and recorded ".repeat(40));
    let (title, body) = message_fields(&long);
    let mut l = parse_log_entity(
        &fs::read_to_string(golden_dir("valid").join("LOG-6b0f39d7a4c1.md")).unwrap(),
    )
    .unwrap();
    l.title = title;
    l.body = body;
    assert_eq!(l.message(), long);
    assert_eq!(parse_log_entity(&serialize_log_entity(&l)).unwrap(), l);
}

// ---------------------------------------------------------------------------
// The previous layout, read for one window
// ---------------------------------------------------------------------------

#[test]
fn parsed_task_carries_the_model() {
    let input = fs::read_to_string(golden_dir("valid").join("TASK-8f3a91c2d4e7.md")).unwrap();
    let t = parse_task(&input).unwrap();
    assert_eq!(t.id.to_string(), "TASK-8f3a91c2d4e7");
    assert_eq!(t.status, TaskStatus::InProgress);
    assert_eq!(t.scope.len(), 2);
    assert_eq!(t.blocked_by.len(), 1);
    assert_eq!(t.verify, vec!["auth-tests", "no-jwt"]);
    // One proof entry per verifier that ran (`verify` is a list).
    assert_eq!(t.proof.len(), t.verify.len());
    assert_eq!(t.proof[0].proof_type, ProofType::Test);
    assert_eq!(t.proof[1].verifier.as_deref(), Some("no-jwt@9ab0c1d2"));
    assert!(!t.proof[0].proof_type.is_weak());
    assert_eq!(t.criteria_by, Some(CriteriaBy::Creator));
    assert_eq!(t.created, "2026-07-25T09:14:00Z");
    assert_eq!(t.proof[0].verifier.as_deref(), Some("auth-tests@1f2e3d4c"));
    assert_eq!(t.version, 7);

    // A schema 2 task keeps its log in the body, and it is still read there.
    let log = parse_log(&t.body);
    assert_eq!(log.len(), 2);
    assert_eq!(log[0].who, "claude-code@host-3");
    assert!(log[1].message.starts_with("released:"));
}

/// The body section is legacy and stays tolerant: a markdown body may hold
/// anything, so a line under `## Log` that is not an entry is skipped and
/// reported by `check`. The strictness lives in the file form, where the whole
/// content is the log and there is nothing else a line could be.
#[test]
fn the_body_log_stays_tolerant_where_the_file_log_is_strict() {
    let body = "## Log\n- 2026-07-26T14:02Z marie@laptop — an entry\nprose under the heading\n";
    assert_eq!(parse_log(body).len(), 1);
    assert!(parse_log_file(body).is_err());
}

/// Both previous layouts are read and **neither is written**: the crate
/// exposes no way to append to either, which is what makes append-only a
/// property of the storage rather than a convention the format asks for
/// (ADR-25f977377fa0).
///
/// Asserted on the surface, because that is where the guarantee lives. The two
/// functions that used to do it are gone, and a reader of this suite should be
/// told so here rather than discover it from a compile error.
#[test]
fn the_previous_layouts_are_read_and_never_written() {
    let input = fs::read_to_string(golden_dir("valid").join("TASK-8f3a91c2d4e7.md")).unwrap();
    let t = parse_task(&input).unwrap();
    assert_eq!(parse_log(&t.body).len(), 2, "the body section still reads");

    let path = golden_dir("valid").join("log").join("TASK-2f8c41ba07d3.md");
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(parse_log_file(&text).unwrap().len(), 4, "so does the file");

    // Every entry either layout holds becomes an entity of its own, and the
    // rendering is a projection of that entity's fields.
    let l = parse_log_entity(
        &fs::read_to_string(golden_dir("valid").join("LOG-6b0f39d7a4c1.md")).unwrap(),
    )
    .unwrap();
    let rendered = LogEntry::of(&l);
    assert_eq!(rendered.timestamp, l.created);
    assert_eq!(rendered.who, l.author.clone().unwrap());
    assert_eq!(rendered.message, l.message());
}

// ---------------------------------------------------------------------------
// Identity, freezes, scopes
// ---------------------------------------------------------------------------

#[test]
fn prefix_resolution_never_guesses() {
    let ids = vec![
        EntityId::parse("TASK-8f3a91c2d4e7").unwrap(),
        EntityId::parse("TASK-8f3a00000000").unwrap(),
        EntityId::parse("ADR-3c7e0b9142af").unwrap(),
    ];

    // Unique: resolved.
    let r = resolve_prefix("3c7e", &ids).unwrap();
    assert_eq!(r.to_string(), "ADR-3c7e0b9142af");

    // Ambiguous: an error listing the candidates, never a silent choice.
    match resolve_prefix("8f3a", &ids) {
        Err(Error::AmbiguousPrefix { candidates, .. }) => assert_eq!(candidates.len(), 2),
        other => panic!("expected AmbiguousPrefix, got {other:?}"),
    }

    // The type prefix disambiguates the kind, not the hex.
    let r = resolve_prefix("TASK-8f3a9", &ids).unwrap();
    assert_eq!(r.to_string(), "TASK-8f3a91c2d4e7");

    // Not found and too short are distinct errors.
    assert!(matches!(
        resolve_prefix("ffff", &ids),
        Err(Error::NotFound(_))
    ));
    assert!(matches!(
        resolve_prefix("8f", &ids),
        Err(Error::PrefixTooShort(_))
    ));
}

#[test]
fn id_generation_is_deterministic_for_a_given_creation_act() {
    let a = EntityId::generate(
        EntityKind::Task,
        "2026-07-27T10:00:00Z",
        "pi@host",
        "title",
        b"seed",
    );
    let b = EntityId::generate(
        EntityKind::Task,
        "2026-07-27T10:00:00Z",
        "pi@host",
        "title",
        b"seed",
    );
    let c = EntityId::generate(
        EntityKind::Task,
        "2026-07-27T10:00:00Z",
        "pi@host",
        "title",
        b"other",
    );
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.hex().len(), 12);
    EntityId::parse(&a.to_string()).unwrap();
}

#[test]
fn freeze_hash_ignores_editing_noise_but_not_meaning() {
    let original = "The tests pass, and no reference to\njwt.verify remains\n";
    let noisy = "The tests pass, and no reference to  \r\njwt.verify remains\n\n\n";
    let weakened = "The tests pass\n";

    assert_eq!(freeze_hash(original), freeze_hash(noisy));
    assert_ne!(freeze_hash(original), freeze_hash(weakened));

    let anchor = freeze_hash_short(original);
    assert_eq!(anchor.len(), 12);
    assert!(verify_frozen(noisy, &anchor));
    assert!(!verify_frozen(weakened, &anchor));
}

#[test]
fn task_state_machine_matches_the_spec() {
    use TaskStatus::*;
    // Legal: claim, done, release, pickup after TTL, ratified abandonment.
    assert!(Open.transition_allowed(InProgress));
    assert!(InProgress.transition_allowed(Done));
    assert!(InProgress.transition_allowed(Open));
    assert!(InProgress.transition_allowed(InProgress));
    assert!(Open.transition_allowed(Closed));
    assert!(InProgress.transition_allowed(Closed));
    // Illegal: done and closed are terminal, open never jumps to done.
    assert!(!Open.transition_allowed(Done));
    assert!(!Done.transition_allowed(Open));
    assert!(!Done.transition_allowed(InProgress));
    assert!(!Closed.transition_allowed(Open));
    assert!(!Closed.transition_allowed(InProgress));
    assert!(!Done.transition_allowed(Closed));
    assert!(matches!(
        Done.check_transition(Open),
        Err(Error::IllegalTransition { .. })
    ));

    use AdrStatus::*;
    assert!(Proposed.transition_allowed(Accepted));
    assert!(Accepted.transition_allowed(Superseded));
    assert!(!Proposed.transition_allowed(Superseded));
    assert!(!Superseded.transition_allowed(Accepted));
    assert!(!Accepted.transition_allowed(Proposed));
}

#[test]
fn blocked_is_derived_never_declared() {
    let input = fs::read_to_string(golden_dir("valid").join("TASK-8f3a91c2d4e7.md")).unwrap();
    let t = parse_task(&input).unwrap();
    let blocker = &t.blocked_by[0];

    // Unfinished blocker: the task is blocked.
    let active = t.active_blockers(|_| Some(TaskStatus::Open)).unwrap();
    assert_eq!(active.len(), 1);

    // Blocker done: unblocked, not finished.
    let active = t.active_blockers(|_| Some(TaskStatus::Done)).unwrap();
    assert!(active.is_empty());

    // Unknown reference: an error, never a silent unblocking.
    let err = t.active_blockers(|_| None).unwrap_err();
    assert!(matches!(err, Error::UnknownReference(ref s) if s == &blocker.to_string()));
}

#[test]
fn scope_matching_is_verifiable() {
    let globs = vec![
        "src/auth/**".to_string(),
        "src/middleware/session.ts".to_string(),
    ];
    let set = ScopeSet::new(&globs).unwrap();
    assert!(set.matches("src/auth/session.rs"));
    assert!(set.matches("src/auth/deep/nested.rs"));
    assert!(set.matches("src/middleware/session.ts"));
    assert!(!set.matches("src/billing/invoice.rs"));
    assert!(set.overlaps_dir("src/auth", &globs));
    assert!(!set.overlaps_dir("docs", &globs));
}

#[test]
fn closed_blocker_does_not_unblock() {
    // `closed` is not `done`: the work was not carried out, so dependents stay
    // blocked and check reports the case.
    let input = fs::read_to_string(golden_dir("valid").join("TASK-8f3a91c2d4e7.md")).unwrap();
    let t = parse_task(&input).unwrap();
    let active = t.active_blockers(|_| Some(TaskStatus::Closed)).unwrap();
    assert_eq!(active.len(), 1);
}
