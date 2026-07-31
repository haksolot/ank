//! Conformance suite for the Ank format.
//!
//! Any third-party tool that claims to read or write the format can reuse the
//! `tests/golden/` directory: the `valid/` files must round-trip byte for byte
//! once normalised, and the `invalid/` files must be rejected.
//!
//! "Once normalised" carries one file: `TASK-c71f0e5a9b23.md` is in CRLF on
//! purpose and must come back in LF, because the format is read in either and
//! written in one (§3).

use ank_core::*;
use std::fs;
use std::path::PathBuf;

fn golden_dir(sub: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(sub)
}

/// The round-trip is the identity **on canonical form** (§3). Valid but
/// non-canonical input is read correctly and normalised on first rewrite, so
/// the assertion is byte identity against the *normalised* input: for a file
/// already canonical that is the file itself, and the old guarantee is
/// unweakened. For one that is not, demanding it come back unchanged would be
/// demanding that ank write CRLF, which §3 forbids in as many words.
#[test]
fn valid_files_round_trip_byte_identical_once_canonical() {
    let dir = golden_dir("valid");
    let mut checked = 0;
    let mut crlf_seen = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
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
        checked += 1;
    }
    assert!(checked >= 5, "valid golden files are missing");
    // Guards the fixture against .gitattributes: if git ever normalises it on
    // checkout, this test stops covering CRLF and says so instead of passing.
    assert_eq!(
        crlf_seen, 1,
        "the CRLF fixture is missing or was converted to LF on checkout"
    );
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
    let dir = golden_dir("invalid");
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
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
            other => panic!("invalid file not covered: {other}"),
        };
        assert!(ok, "{name}: wrong error: {err:?}");
    }
}

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

    let log = parse_log(&t.body);
    assert_eq!(log.len(), 2);
    assert_eq!(log[0].who, "claude-code@host-3");
    assert!(log[1].message.starts_with("released:"));
}

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
