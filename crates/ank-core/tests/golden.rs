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
        schemas.push(match &entity {
            Entity::Task(t) => t.schema,
            Entity::Adr(a) => a.schema,
        });
        checked += 1;
    }
    assert!(checked >= 8, "valid golden files are missing");
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
            // A `verified` list is optional; an entry in it is not partial.
            "verified-without-at" => {
                matches!(&err, Error::Yaml(_)) && err.to_string().contains("missing field `at`")
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
    assert_eq!(entries.len(), 3);

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

/// The property the move was made to protect, asserted on the file it moved to.
#[test]
fn appending_to_a_log_file_is_a_one_line_diff() {
    let path = golden_dir("valid").join("log").join("TASK-2f8c41ba07d3.md");
    let before = fs::read_to_string(&path).unwrap();

    let entry = LogEntry {
        timestamp: "2026-08-12T10:00Z".into(),
        who: "codex@host-9".into(),
        message: "picked up after expiry".into(),
    };
    let after = append_log_file(&before, &entry);

    assert!(
        after.starts_with(&before),
        "an append rewrites nothing that was already there"
    );
    assert_eq!(
        after.lines().count(),
        before.lines().count() + 1,
        "one log entry, one added line"
    );
    assert_eq!(parse_log_file(&after).unwrap().len(), 4);

    // On an empty file the entry is the whole content: there is no header to
    // create, which is one thing less than the body section needed.
    let created = append_log_file("", &entry);
    assert_eq!(created, format!("{}\n", entry.format_line()));
    assert_eq!(parse_log_file(&created).unwrap().len(), 1);
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

#[test]
fn log_append_is_a_one_line_diff() {
    let input = fs::read_to_string(golden_dir("valid").join("TASK-8f3a91c2d4e7.md")).unwrap();
    let t = parse_task(&input).unwrap();

    let entry = LogEntry {
        timestamp: "2026-07-27T09:00Z".into(),
        who: "codex@host-9".into(),
        message: "picked up after expiry".into(),
    };
    let new_body = append_log(&t.body, &entry);

    let before: Vec<&str> = t.body.lines().collect();
    let after: Vec<&str> = new_body.lines().collect();
    assert_eq!(
        after.len(),
        before.len() + 1,
        "one log entry, one added line"
    );
    assert_eq!(
        &after[..before.len()],
        &before[..],
        "nothing existing moves"
    );
    assert_eq!(parse_log(&new_body).len(), 3);

    // On an empty body, the section is created.
    let created = append_log("", &entry);
    assert!(created.starts_with("## Log\n- "));
    assert_eq!(parse_log(&created).len(), 1);
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
