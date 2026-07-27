//! Suite de conformite du format Ankor.
//!
//! Tout outil tiers qui pretend lire ou ecrire le format peut reutiliser
//! le dossier `tests/golden/` : les fichiers `valid/` doivent round-tripper
//! a l'octet pres, les fichiers `invalid/` doivent etre refuses.

use ankor_core::*;
use std::fs;
use std::path::PathBuf;

fn golden_dir(sub: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(sub)
}

#[test]
fn valid_files_round_trip_byte_identical() {
    let dir = golden_dir("valid");
    let mut checked = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let input = fs::read_to_string(&path).unwrap();
        let entity = parse_entity(&input)
            .unwrap_or_else(|e| panic!("{} devrait etre valide : {e}", path.display()));
        let output = serialize_entity(&entity);
        assert_eq!(
            input,
            output,
            "round-trip non identique pour {}",
            path.display()
        );
        checked += 1;
    }
    assert!(checked >= 5, "fichiers golden valides manquants");
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
            .unwrap_or_else(|| panic!("{name} devrait etre refuse"));
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
            other => panic!("fichier invalide non couvert : {other}"),
        };
        assert!(ok, "{name} : mauvaise erreur : {err:?}");
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
    // Une entree de preuve par verificateur execute (verify en liste).
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

    // Unique : resolu.
    let r = resolve_prefix("3c7e", &ids).unwrap();
    assert_eq!(r.to_string(), "ADR-3c7e0b9142af");

    // Ambigu : erreur avec les candidats, jamais un choix silencieux.
    match resolve_prefix("8f3a", &ids) {
        Err(Error::AmbiguousPrefix { candidates, .. }) => assert_eq!(candidates.len(), 2),
        other => panic!("attendu AmbiguousPrefix, obtenu {other:?}"),
    }

    // Le prefixe de type desambiguise le kind, pas le hex.
    let r = resolve_prefix("TASK-8f3a9", &ids).unwrap();
    assert_eq!(r.to_string(), "TASK-8f3a91c2d4e7");

    // Introuvable et trop court sont des erreurs distinctes.
    assert!(matches!(resolve_prefix("ffff", &ids), Err(Error::NotFound(_))));
    assert!(matches!(resolve_prefix("8f", &ids), Err(Error::PrefixTooShort(_))));
}

#[test]
fn id_generation_is_deterministic_for_a_given_creation_act() {
    let a = EntityId::generate(EntityKind::Task, "2026-07-27T10:00:00Z", "pi@host", "titre", b"seed");
    let b = EntityId::generate(EntityKind::Task, "2026-07-27T10:00:00Z", "pi@host", "titre", b"seed");
    let c = EntityId::generate(EntityKind::Task, "2026-07-27T10:00:00Z", "pi@host", "titre", b"autre");
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.hex().len(), 12);
    EntityId::parse(&a.to_string()).unwrap();
}

#[test]
fn freeze_hash_ignores_editing_noise_but_not_meaning() {
    let original = "Les tests passent, et plus aucune\nréférence à jwt.verify\n";
    let noisy = "Les tests passent, et plus aucune  \r\nréférence à jwt.verify\n\n\n";
    let weakened = "Les tests passent\n";

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
        message: "reprise après expiration".into(),
    };
    let new_body = append_log(&t.body, &entry);

    let before: Vec<&str> = t.body.lines().collect();
    let after: Vec<&str> = new_body.lines().collect();
    assert_eq!(after.len(), before.len() + 1, "un log = une ligne ajoutee");
    assert_eq!(&after[..before.len()], &before[..], "rien d'existant ne bouge");
    assert_eq!(parse_log(&new_body).len(), 3);

    // Sur un corps vide, la section est creee.
    let created = append_log("", &entry);
    assert!(created.starts_with("## Log\n- "));
    assert_eq!(parse_log(&created).len(), 1);
}

#[test]
fn task_state_machine_matches_the_spec() {
    use TaskStatus::*;
    // Legales : claim, done, release, reprise apres TTL, abandon ratifie.
    assert!(Open.transition_allowed(InProgress));
    assert!(InProgress.transition_allowed(Done));
    assert!(InProgress.transition_allowed(Open));
    assert!(InProgress.transition_allowed(InProgress));
    assert!(Open.transition_allowed(Closed));
    assert!(InProgress.transition_allowed(Closed));
    // Illegales : done et closed sont terminaux, open ne saute jamais a done.
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

    // Bloqueur non termine : la tache est bloquee.
    let active = t.active_blockers(|_| Some(TaskStatus::Open)).unwrap();
    assert_eq!(active.len(), 1);

    // Bloqueur done : debloquee, pas terminee.
    let active = t.active_blockers(|_| Some(TaskStatus::Done)).unwrap();
    assert!(active.is_empty());

    // Reference inconnue : erreur, jamais un deblocage silencieux.
    let err = t.active_blockers(|_| None).unwrap_err();
    assert!(matches!(err, Error::UnknownReference(ref s) if s == &blocker.to_string()));
}

#[test]
fn scope_matching_is_verifiable() {
    let globs = vec!["src/auth/**".to_string(), "src/middleware/session.ts".to_string()];
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
    // `closed` n'est pas `done` : le travail n'a pas ete fait,
    // les dependantes restent bloquees et check remonte le cas.
    let input = fs::read_to_string(golden_dir("valid").join("TASK-8f3a91c2d4e7.md")).unwrap();
    let t = parse_task(&input).unwrap();
    let active = t.active_blockers(|_| Some(TaskStatus::Closed)).unwrap();
    assert_eq!(active.len(), 1);
}
