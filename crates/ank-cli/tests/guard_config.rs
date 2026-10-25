//! `guard.corpus` and `guard.edit` in `.ank/config.yml`, read and written by
//! `ank config` (ADR-43cbb20dcbf1, TASK-ebf32feffa23).
//!
//! **Through the binary.** The hook that applies the guards reads them from the
//! file, so what is pinned here is what `ank config` leaves on disk and what it
//! prints: a default printed without being written, a value outside the closed
//! set refused before the file is touched, and an unset that takes out one key
//! and nothing beside it.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

fn ank(dir: &Path, home: &Path, args: &[&str]) -> (i32, String, String) {
    let home = home.to_string_lossy().into_owned();
    let out = Command::new(ANK)
        .args(args)
        .current_dir(dir)
        .env("XDG_CONFIG_HOME", &home)
        .env("APPDATA", &home)
        .env("HOME", "")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("ANK_AGENT", "guard-config@fixture")
        .output()
        .expect("the binary under test must run");
    (
        out.status.code().expect("the binary must not be signalled"),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A corpus `ank init` made, and the path of its `config.yml`.
fn corpus(what: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = scratch::dir(what);
    let home = root.join("config");
    fs::create_dir_all(home.join("ank")).unwrap();
    fs::write(
        home.join("ank").join("corpora.yml"),
        "schema: 1\ncorpora: {}\n",
    )
    .unwrap();
    let repo = root.join("repo");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q", "."]);
    git(&repo, &["config", "commit.gpgsign", "false"]);
    let (code, _, err) = ank(&repo, &home, &["init"]);
    assert_eq!(code, 0, "ank init: {err}");
    let cfg = repo.join(".ank").join("config.yml");
    (repo, home, cfg)
}

#[test]
fn an_absent_guard_prints_its_default_and_leaves_the_file_as_it_was() {
    let (repo, home, cfg) = corpus("guard-defaults");
    let before = fs::read(&cfg).unwrap();
    assert!(!String::from_utf8_lossy(&before).contains("guard"));

    // Printed the way every absent key is, the value and then where it came
    // from; the document carries the value alone.
    let (code, out, err) = ank(&repo, &home, &["config", "guard.edit"]);
    assert_eq!((code, out.trim()), (0, "warn (default)"), "{err}");
    let (code, out, err) = ank(&repo, &home, &["config", "guard.corpus"]);
    assert_eq!((code, out.trim()), (0, "deny (default)"), "{err}");
    let (code, out, err) = ank(&repo, &home, &["config", "guard.edit", "--json"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains(r#""value":"warn""#), "{out}");
    assert!(out.contains(r#""source":"default""#), "{out}");

    assert_eq!(fs::read(&cfg).unwrap(), before);
}

#[test]
fn a_guard_is_written_read_back_and_unset_alone() {
    let (repo, home, cfg) = corpus("guard-write");
    let before = fs::read_to_string(&cfg).unwrap();

    let (code, _, err) = ank(&repo, &home, &["config", "guard.corpus", "off"]);
    assert_eq!(code, 0, "{err}");
    let (code, _, err) = ank(&repo, &home, &["config", "guard.edit", "claim"]);
    assert_eq!(code, 0, "{err}");
    let written = fs::read_to_string(&cfg).unwrap();
    assert!(written.contains("guard:\n"), "{written}");
    assert!(written.contains("  edit: claim\n"), "{written}");

    let (code, out, err) = ank(&repo, &home, &["config", "guard.edit"]);
    assert_eq!((code, out.trim()), (0, "claim"), "{err}");
    let (code, out, err) = ank(&repo, &home, &["config", "guard.corpus"]);
    assert_eq!((code, out.trim()), (0, "off"), "{err}");

    let (code, _, err) = ank(&repo, &home, &["config", "--unset", "guard.edit"]);
    assert_eq!(code, 0, "{err}");
    let after = fs::read_to_string(&cfg).unwrap();
    assert!(!after.contains("edit:"), "{after}");
    // `off` is quoted, because written plain YAML 1.1 reads it as a boolean.
    assert!(after.contains("  corpus: \"off\"\n"), "{after}");
    // Only that key: the file is what it was with `guard.corpus` written alone.
    let (code, out, _) = ank(&repo, &home, &["config", "guard.edit"]);
    assert_eq!((code, out.trim()), (0, "warn (default)"));

    let (code, _, err) = ank(&repo, &home, &["config", "--unset", "guard.corpus"]);
    assert_eq!(code, 0, "{err}");
    let unset = fs::read_to_string(&cfg).unwrap();
    assert!(!unset.contains("corpus:"), "{unset}");
    assert_eq!(
        unset.replace("guard: {}\n", "").replace("guard:\n", ""),
        before
    );
}

#[test]
fn a_value_outside_the_closed_set_is_refused_naming_the_accepted_ones() {
    let (repo, home, cfg) = corpus("guard-refused");
    let before = fs::read(&cfg).unwrap();

    let (code, out, err) = ank(&repo, &home, &["config", "guard.edit", "strict"]);
    assert_ne!(code, 0, "{out}");
    for v in ["off", "warn", "claim"] {
        assert!(err.contains(v), "guard.edit refusal names {v}: {err}");
    }
    assert_eq!(fs::read(&cfg).unwrap(), before);

    let (code, out, err) = ank(&repo, &home, &["config", "guard.corpus", "warn"]);
    assert_ne!(code, 0, "{out}");
    for v in ["deny", "off"] {
        assert!(err.contains(v), "guard.corpus refusal names {v}: {err}");
    }
    assert_eq!(fs::read(&cfg).unwrap(), before);

    let (code, _, err) = ank(&repo, &home, &["config", "guard.shell", "off"]);
    assert_ne!(code, 0);
    assert!(err.contains("guard.edit"), "{err}");
    assert_eq!(fs::read(&cfg).unwrap(), before);
}
